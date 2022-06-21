use std::any::Any;
use std::net::SocketAddr;
use cell_core::request::ServerRequestTrait;
use hyper::{Body, Method, Request, Response, Server, StatusCode};

pub struct HttpRequest {
  pub  request: Request<Body>,
  pub remote_addr:String,
}

unsafe impl Send for HttpRequest {

}
unsafe impl Sync for HttpRequest {

}

impl HttpRequest {
    pub fn new(request: Request<Body>, remote_addr: String) -> Self {
        Self { request, remote_addr }
    }
}


impl ServerRequestTrait for HttpRequest {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_ip(&self)->String{
        self.remote_addr.clone()
    }
}

