use std::any::Any;
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use cell_core::cerror::CellResult;
use cell_core::request::ServerResponseTrait;

pub struct HttpResponse {}

impl HttpResponse {
    pub fn new() -> Self {
        Self {}
    }
}

impl ServerResponseTrait for HttpResponse {
    fn add_header(&mut self, key: HeaderName, value: HeaderValue) {
        todo!()
    }

    fn fire_result(&mut self, result: Response<Body>) -> CellResult<()> {
        todo!()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}