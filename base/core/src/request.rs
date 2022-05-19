use std::fmt::Error;
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use tokio::sync::oneshot::Sender;
use crate::header::name::CellHeaderName;
use crate::header::value::CellHeaderValue;

pub trait ServerRequestTrait: Send + Sync {}

pub trait ServerResponseTrait: Send + Sync {
     fn add_header(&mut self, key: HeaderName, value: HeaderValue);
     fn fire_result(&mut self, result: Response<Body>) -> Result<(), Error>;
}


// mock
pub struct MockRequest {}


impl MockRequest {
    pub fn new() -> Self {
        MockRequest {}
    }
}


impl ServerRequestTrait for MockRequest {}