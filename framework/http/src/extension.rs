use std::any::Any;
use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::mem;
use std::sync::{Arc, Mutex};
use cell_core::cerror::CellResult;
use cell_core::dispatcher::DefaultDispatcher;
use cell_core::extension::{ExtensionFactory, NodeContext, NodeExtension};
use cell_core::selector::{CommandSelector, SelectorStrategy};
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use shaku::{module, Component, Interface, HasComponent};
use cell_core::command::Command;
use logsdk::cinfo;
use crate::channel::HttpChannel;
use crate::dispatcher::HttpDispatcher;
use crate::selector::HttpSelector;
use crate::server::{HttpServer, HttpServerBuilder};

pub struct HttpExtensionFactory {}

impl ExtensionFactory for HttpExtensionFactory {
    fn build_extension(&self, compoents: Vec<Arc<Box<dyn Any>>>) -> Option<Arc<RefCell<dyn NodeExtension>>> {
        let ext = HttpExtensionBuilder::default().build();
        Some(Arc::new(RefCell::new(ext)))
    }
}

pub struct HttpExtensionBuilder {
    server_builder: HttpServerBuilder,
}

impl HttpExtensionBuilder {
    pub fn with_selector(mut self, se: Box<dyn CommandSelector<'static>>) -> Self {
        self.server_builder = self.server_builder.with_selector(se);
        self
    }
}

impl Default for HttpExtensionBuilder {
    fn default() -> Self {
        HttpExtensionBuilder {
            server_builder: Default::default()
        }
    }
}

impl HttpExtensionBuilder {
    pub fn build(self) -> HttpExtension {
        let server = self.server_builder.build();
        HttpExtension::new(Arc::new(RefCell::new(server)))
    }
}

pub struct HttpExtension {
    // TODO?  may have a another better idea about how to inject with component  rather than wrapped by mutex
    // but it does not matter , right ?
    server: Arc<RefCell<HttpServer>>,
}

impl HttpExtension {
    pub fn new(server: Arc<RefCell<HttpServer>>) -> Self {
        Self { server }
    }
}


pub const HttpModule: CellModule = CellModule::new(1, "HTTP_EXTENSION", &LogLevel::Info);

pub trait HttpNodeExtension: NodeExtension + Interface {}

impl HttpNodeExtension for HttpExtension {}


unsafe impl Sync for HttpExtension {}

unsafe impl Send for HttpExtension {}

impl NodeExtension for HttpExtension {
    fn on_init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let mut s = self.server.clone().take();
        s.init(ctx);
        let v = self.server.replace(s);
        Ok(())
    }
    fn module(&self) -> CellModule {
        HttpModule
    }
    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let s = self.server.clone().take();
        ctx.borrow().tokio_runtime.spawn(s.start());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use cell_core::application::CellApplication;
    use cell_core::command::Command;
    use cell_core::core::{ProtocolID, runTypeHttp};
    use cell_core::extension::{ExtensionFactory, NodeContext, NodeExtension};
    use cell_core::selector::MockDefaultPureSelector;
    use crate::extension::{HttpExtension, HttpExtensionBuilder, HttpExtensionFactory};

    #[test]
    fn test_extension() {
        let mock_select1 = MockDefaultPureSelector::new();
        let mut ex = HttpExtensionBuilder::default().with_selector(Box::new(mock_select1)).build();
        let ctx = NodeContext::default();
        let arcc = Arc::new(RefCell::new(ctx));
        ex.start(arcc.clone()).unwrap();
        let a = async {
            thread::sleep(Duration::from_secs(1000000))
        };
        arcc.clone().borrow().tokio_runtime.block_on(a);
    }

    pub struct DemoExtensionFactory {}

    impl ExtensionFactory for DemoExtensionFactory {
        fn commands(&self) -> Option<Vec<Command<'static>>> {
            let mut ret: Vec<Command<'static>> = Vec::new();
            let mut c1 = Command::default();
            c1 = c1.with_protocol_id("asd" as ProtocolID)
                .with_run_type(runTypeHttp);
            ret.push(c1);

            return Some(ret);
        }
    }

    #[test]
    fn test_commands() {
        let mut factories: Vec<Box<dyn ExtensionFactory>> = Vec::new();
        factories.push(Box::new(HttpExtensionFactory {}));
        factories.push(Box::new(DemoExtensionFactory {}));
        let mut app = CellApplication::new(factories);
        app.run(vec![]);
    }
}