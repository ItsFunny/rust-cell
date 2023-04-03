use cell_core::cerror::CellResult;
use cell_core::request::ServerResponseTrait;
use futures::*;
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use std::any::Any;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use tokio::sync::oneshot;

pub struct HttpResponse {
    // tx: oneshot::Sender<Response<Body>>,
    txx: Sender<Response<Body>>,
}

unsafe impl Send for HttpResponse {}

unsafe impl Sync for HttpResponse {}

impl HttpResponse {
    pub fn new(txxxx: Sender<Response<Body>>) -> Self {
        Self { txx: txxxx }
    }
}

impl ServerResponseTrait for HttpResponse {
    fn add_header(&mut self, key: HeaderName, value: HeaderValue) {}

    fn fire_result(&mut self, result: Response<Body>) -> CellResult<()> {
        let (tx, rx) = std::sync::mpsc::channel::<Response<Body>>();
        self.txx.send(result);
        // TODO ,use another channel ,because of the ownship
        // self.tx.send(result);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
