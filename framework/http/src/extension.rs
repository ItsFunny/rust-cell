use cell_core::extension::NodeExtension;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;

pub struct HttpExtension {}


pub const HttpModule: CellModule = CellModule::new(1, "HTTP_EXTENSION", &LogLevel::Info);

impl Default for HttpExtension {
    fn default() -> Self {
        HttpExtension {}
    }
}

impl NodeExtension for HttpExtension {
    fn module(&self) -> CellModule {
        HttpModule
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use cell_core::extension::{NodeContext, NodeExtension};
    use crate::extension::HttpExtension;

    #[test]
    fn test_extension() {
        let mut ex = HttpExtension::default();
        let ctx = NodeContext::default();
        ex.start(Arc::new(ctx)).unwrap();
    }
}