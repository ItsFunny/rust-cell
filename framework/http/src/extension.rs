// use cell_core::extension::NodeExtension;
// use logsdk::common::LogLevel;
// use logsdk::module::CellModule;
// use crate::server::HttpServer;
//
// pub struct HttpExtensionBuilder {}
//
// impl Default for HttpExtensionBuilder {
//     fn default() -> Self {
//         HttpExtensionBuilder {}
//     }
// }
//
// impl HttpExtensionBuilder {
//     pub fn build(self) -> HttpExtension {
//         HttpExtension::new()
//     }
// }
//
// pub struct HttpExtension {
//     server: HttpServer,
// }
//
// impl HttpExtension {
//     pub fn new(server: HttpServer) -> Self {
//         Self { server }
//     }
// }
//
//
// pub const HttpModule: CellModule = CellModule::new(1, "HTTP_EXTENSION", &LogLevel::Info);
//
//
// impl NodeExtension for HttpExtension {
//     fn module(&self) -> CellModule {
//         HttpModule
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use std::sync::Arc;
//     use cell_core::extension::{NodeContext, NodeExtension};
//     use crate::extension::{HttpExtension, HttpExtensionBuilder};
//
//     #[test]
//     fn test_extension() {
//         let mut ex = HttpExtensionBuilder::default().build();
//         let ctx = NodeContext::default();
//         ex.start(Arc::new(ctx)).unwrap();
//     }
// }