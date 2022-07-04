use std::cell::RefCell;
use std::mem;
use std::sync::{Arc, Mutex};
use cell_core::cerror::CellResult;
use cell_core::dispatcher::DefaultDispatcher;
use cell_core::extension::{NodeContext, NodeExtension};
use cell_core::selector::{CommandSelector, SelectorStrategy};
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use shaku::{module, Component, Interface, HasComponent};
use crate::channel::HttpChannel;
use crate::dispatcher::HttpDispatcher;
use crate::selector::HttpSelector;
use crate::server::HttpServer;

pub struct HttpExtensionBuilder {
    selector: Option<Box<dyn CommandSelector<'static>>>,
}

impl HttpExtensionBuilder {
    pub fn with_selector(mut self, se: Box<dyn CommandSelector<'static>>) -> Self {
        self.selector = Some(se);
        self
    }
}

impl Default for HttpExtensionBuilder {
    fn default() -> Self {
        HttpExtensionBuilder {
            selector: None
        }
    }
}

impl HttpExtensionBuilder {
    pub fn build(self) -> HttpExtension {
        let mut default_http_selector = Box::new(HttpSelector::default());
        let mut executors: Vec<Box<dyn CommandSelector>> = Vec::new();
        if let Some(v) = self.selector {
            executors.push(v);
        }
        executors.push(default_http_selector);
        let mut selector_strategy = SelectorStrategy::new(executors);
        let channel = HttpChannel::default();
        let http_dispatch = HttpDispatcher::new();
        let default_dispatcher = DefaultDispatcher::new(
            Box::new(channel),
            selector_strategy,
            Box::new(http_dispatch));
        let server = HttpServer::new(default_dispatcher);
        HttpExtension::new(Arc::new(Mutex::new(RefCell::new(server))))
    }
}

#[derive(Component)]
#[shaku(interface = HttpNodeExtension)]
pub struct HttpExtension {
    // TODO?  may have a another better idea about how to inject with component  rather than wrapped by mutex
    // but it does not matter , right ?
    server: Arc<Mutex<RefCell<HttpServer>>>,
}

impl HttpExtension {
    pub fn new(server: Arc<Mutex<RefCell<HttpServer>>>) -> Self {
        Self { server }
    }
}


pub const HttpModule: CellModule = CellModule::new(1, "HTTP_EXTENSION", &LogLevel::Info);

pub trait HttpNodeExtension: NodeExtension + Interface {}

impl HttpNodeExtension for HttpExtension {}


unsafe impl Sync for HttpExtension {}

unsafe impl Send for HttpExtension {}

impl NodeExtension for HttpExtension {
    fn module(&self) -> CellModule {
        HttpModule
    }
    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let s = self.server.lock().unwrap().take();
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
    use cell_core::extension::{NodeContext, NodeExtension};
    use cell_core::selector::MockDefaultPureSelector;
    use crate::extension::{HttpExtension, HttpExtensionBuilder};

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
}