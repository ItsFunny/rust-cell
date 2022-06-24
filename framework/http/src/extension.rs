use std::sync::Arc;
use cell_core::cerror::CellResult;
use cell_core::dispatcher::DefaultDispatcher;
use cell_core::extension::{NodeContext, NodeExtension};
use cell_core::selector::{CommandSelector, SelectorStrategy};
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::channel::HttpChannel;
use crate::dispatcher::HttpDispatcher;
use crate::selector::HttpSelector;
use crate::server::HttpServer;

pub struct HttpExtensionBuilder {
    selector: Option<Box<dyn CommandSelector<'static>>>,
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
        HttpExtension::new(server)
    }
}

pub struct HttpExtension {
    server: HttpServer,
}

impl HttpExtension {
    pub fn new(server: HttpServer) -> Self {
        Self { server }
    }
}


pub const HttpModule: CellModule = CellModule::new(1, "HTTP_EXTENSION", &LogLevel::Info);


impl NodeExtension for HttpExtension {
    fn module(&self) -> CellModule {
        HttpModule
    }
    fn on_start(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        // TODO
        ctx.tokio_runtime.spawn(self.server.start());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use cell_core::extension::{NodeContext, NodeExtension};
    use crate::extension::{HttpExtension, HttpExtensionBuilder};

    #[test]
    fn test_extension() {
        let mut ex = HttpExtensionBuilder::default().build();
        let ctx = NodeContext::default();
        ex.start(Arc::new(ctx)).unwrap();
        thread::sleep(Duration::from_secs(1000000))
    }
}