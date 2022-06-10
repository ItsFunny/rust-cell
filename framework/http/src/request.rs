use std::any::Any;
use cell_core::request::ServerRequestTrait;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

pub struct HttpRequest {
  pub  request: Request<Body>
}

impl HttpRequest {
    pub fn new(request: Request<Body>) -> Self {
        Self { request }
    }
}


impl ServerRequestTrait for HttpRequest {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
