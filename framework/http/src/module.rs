use crate::extension::HttpExtension;
use shaku::{module, Component, HasComponent, Interface};

#[cfg(test)]
mod tests {
    use crate::extension::{HttpExtension, HttpExtensionBuilder};
    use crate::server::{HttpServer, HttpServerBuilder};
    use std::cell::RefCell;
    use std::sync::{Arc, Mutex};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // #[test]
    // fn test_http_module() {
    //     let builder: HttpExtensionBuilder = HttpExtensionBuilder::default();
    //     let server: Arc<Mutex<RefCell<HttpServer>>> = Arc::new(Mutex::new(RefCell::new(HttpServerBuilder::default().build())));
    //     let module = HttpModule::builder()
    //         .with_component_parameters::<HttpExtension>(HttpExtensionParameters {
    //             server: server,
    //         })
    //         .build();
    // }
}
