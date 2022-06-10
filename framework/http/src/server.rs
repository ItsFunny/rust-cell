use std::error::Error;
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
use futures::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use cell_core::cerror::{CellError, CellResult, ErrorEnumsStruct};
use cell_core::channel::ChannelTrait;
use cell_core::request::MockRequest;
use crate::channel::HttpChannel;
use crate::dispatcher::HttpDispatcher;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub struct HttpServer<'e, 'a> {
    dispatcher: HttpDispatcher<'e, 'a>,
    _marker_e: PhantomData<&'e ()>,
    _marker_a: PhantomData<&'a ()>,
}

unsafe impl<'e, 'a> Send for HttpServer<'e, 'a> {}

unsafe impl<'e, 'a> Sync for HttpServer<'e, 'a> {}
// fn create_tcp_listener(addr: net::SocketAddr, backlog: u32) -> CellResult<net::TcpListener> {
//     use socket2::{Domain, Protocol, Socket, Type};
//     let domain = Domain::for_address(addr);
//     let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
//     socket.set_reuse_address(true)?;
//     socket.bind(&addr.into())?;
//     // clamp backlog to max u32 that fits in i32 range
//     let backlog = cmp::min(backlog, i32::MAX as u32) as i32;
//     socket.listen(backlog)?;
//     Ok(net::TcpListener::from(socket))
// }

// impl HttpServer{
//     pub fn listen(mut self, lst: net::TcpListener) -> CellResult<Self> {
//
//     }
// }
//
//


impl<'e, 'a> HttpServer<'e, 'a> {
    pub async fn start(mut self) -> CellResult<()> {
        let addr = ([127, 0, 0, 1], 3000).into();

        let service = make_service_fn(|_conn: &AddrStream| {
            let a = _conn.clone();
            println!("{:?}", a);
            //         service_fn(move |req| {
            //             let http_req=HttpRequest::new(req);
            // let http_resp=HttpResponse::new();
            //             self.dispatch(http_req,http_resp);
            //             // cell_hyper_service_fn(req)
            //         });
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let http_req = HttpRequest::new(req);
                    let http_resp = HttpResponse::new();
                    // ss.dispatch(http_req, http_resp);
                    // cell_hyper_service_fn(req)
                    c()
                }))
            }
        }
        );

        let server = Server::bind(&addr).serve(service);

        println!("Listening on http://{}", addr);

        server.await.map_err(|e| {
            CellError::from(ErrorEnumsStruct::UNKNOWN).with_error(Box::new(e))
        });

        Ok(())
    }
    pub  fn dispatch(mut self, req: HttpRequest, resp: HttpResponse) {
        let mut a =self.dispatcher;
        a.dispatch(req,resp);
        // self.dispatcher.dispatch(req, resp)
    }
}
//
// async fn cell_hyper_service_fn<'e, 'a>(hyp_req: hyper::Request<hyper::Body>) -> Result<Response<Body>, hyper::Error> {
//     echo(hyp_req).await
// }

async fn c()-> Result<Response<Body>, hyper::Error>{
    Ok(Response::new(Body::from(String::from("asd"))))
}

// async fn dispatch<'e,'a>(mut server: Arc<HttpServer<'e,'a>>, req: Request<Body>){
//     let http_req=HttpRequest::new(req);
//     let http_resp=HttpResponse::new();
//     server.dispatch(http_req,http_resp);
// }

async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
        ))),

        // Simply echo the body back to the client.
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

        // Convert to uppercase before sending back to client using a stream.
        (&Method::POST, "/echo/uppercase") => {
            let chunk_stream = req.into_body().map_ok(|chunk| {
                chunk
                    .iter()
                    .map(|byte| byte.to_ascii_uppercase())
                    .collect::<Vec<u8>>()
            });
            Ok(Response::new(Body::wrap_stream(chunk_stream)))
        }

        // Reverse the entire body before sending back to the client.
        //
        // Since we don't know the end yet, we can't simply stream
        // the chunks as they arrive as we did with the above uppercase endpoint.
        // So here we do `.await` on the future, waiting on concatenating the full body,
        // then afterwards the content can be reversed. Only then can we return a `Response`.
        (&Method::POST, "/echo/reversed") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;

            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(Body::from(reversed_body)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::server::HttpServer;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_http_server() {}
}