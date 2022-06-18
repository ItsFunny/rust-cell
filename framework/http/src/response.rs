use std::any::Any;
use std::sync::mpsc::Sender;
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use futures::*;
use tokio::sync::oneshot;
use cell_core::cerror::CellResult;
use cell_core::request::ServerResponseTrait;

pub struct HttpResponse {
    tx:oneshot::Sender<Response<Body>>,
}

unsafe impl Send for HttpResponse {

}
unsafe impl Sync for HttpResponse {

}

impl HttpResponse {
    pub fn new(tx: oneshot::Sender<Response<Body>>) -> Self {
        Self { tx }
    }
}


impl ServerResponseTrait for HttpResponse {
    fn add_header(&mut self, key: HeaderName, value: HeaderValue) {
        todo!()
    }

    fn fire_result(&mut self, result: Response<Body>) -> CellResult<()> {
        // self.txx.send(result)
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}