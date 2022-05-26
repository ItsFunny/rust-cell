use std::any::Any;
use std::fmt::Error;
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use tokio::sync::oneshot::Sender;
use crate::cerror::CellResult;
use crate::core::ProtocolID;
use crate::header::name::CellHeaderName;
use crate::header::value::CellHeaderValue;

pub trait ServerRequestTrait: Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

pub trait ServerResponseTrait: Send + Sync {
    fn add_header(&mut self, key: HeaderName, value: HeaderValue);
    fn fire_result(&mut self, result: Response<Body>) -> CellResult<()>;
    fn as_any(&self) -> &dyn Any;
}


// mock
pub struct MockRequest {
    pub protocol: ProtocolID,
}


impl MockRequest {
    pub fn new() -> Self {
        MockRequest { protocol: "protocol" }
    }
}


impl ServerRequestTrait for MockRequest {
    fn as_any(&self) -> &dyn Any {
        self
    }
}