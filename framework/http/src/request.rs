use std::any::Any;
use std::net::SocketAddr;
use cell_core::request::ServerRequestTrait;
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use cell_core::core::ProtocolID;

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
    fn get_string_protocol(&self)->String{
        self.request.uri().to_string()
    }
}

