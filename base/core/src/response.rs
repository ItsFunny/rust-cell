use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};
use crate::request::ServerResponseTrait;
use futures::*;
use http::header::HeaderName;
use http::{HeaderValue, Response};
use hyper::Body;
use rocket::figment::map;
use serde::ser::Error;
use std::any::Any;
use std::fmt::Error as std_error;
use std::rc::Rc;
use std::sync::mpsc::Sender;

//////////
// pub struct ServerResponseWrapper {
//     tx: Sender<Response<Body>>,
// }
//
// impl ServerResponseTrait for ServerResponseWrapper {
//     fn add_header(&mut self, key: HeaderName, value: HeaderValue) {
//         todo!()
//     }
//
//     fn fire_result(&self, result: Response<Body>) {
//         // let mut hyp_res = hyper::Response::builder();
//         // let (mut sender, hyp_body) = hyper::Body::channel();
//         // let hyp_response = hyp_res.body(hyp_body)
//         //     .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
//     }
// }

pub struct MockResponse {
    tx: Sender<Response<Body>>,
}

impl MockResponse {
    pub fn new(tx: Sender<Response<Body>>) -> Self {
        MockResponse { tx }
    }
}

unsafe impl Sync for MockResponse {}

impl ServerResponseTrait for MockResponse {
    fn add_header(&mut self, key: HeaderName, value: HeaderValue) {
        println!("add header,{},{:?}", key, value);
    }

    // TODO
    fn fire_result(&mut self, result: Response<Body>) -> CellResult<()> {
        self.tx
            .send(result)
            .and_then(|_| Ok(()))
            .map_err(|e| CellError::from(ErrorEnumsStruct::RESPONSE_FAILED).with_error(Box::new(e)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::mpsc;
    use futures::future::ok;
    use futures::*;
    use std::task::{Context, RawWaker, Waker};
    use std::thread;

    #[test]
    fn test_future() {
        let (t, mut r) = tokio::sync::mpsc::channel::<i32>(1);
        let v = async {
            t.send(1).await;
        };
        executor::block_on(v);
        let ret = r.blocking_recv();
        match ret {
            Some(vv) => {
                println!("asdd:{}", vv)
            }
            None => {
                println!("none")
            }
        }
        let rett = r.try_recv();
        match rett {
            Ok(v) => {
                println!("again {}", v)
            }
            Err(e) => {
                println!("none:{}", e)
            }
        }
    }
}
