use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::sync::Arc;
use futures::future::ok;
use futures::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use cell_core::cerror::{CellError, CellResult, ErrorEnumsStruct};
use cell_core::channel::ChannelTrait;
use cell_core::dispatcher::{DefaultDispatcher, DispatchContext};
use cell_core::request::MockRequest;
use crate::channel::HttpChannel;
use crate::dispatcher::HttpDispatcher;
use crate::request::HttpRequest;
use crate::response::HttpResponse;

pub struct HttpServer {
    dispatcher: DefaultDispatcher<'static, 'static>,
    // _marker_e: PhantomData<&'e ()>,
    // _marker_a: PhantomData<&'a ()>,
}

impl  Debug for HttpServer{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"asd")
    }
}

unsafe impl Send for HttpServer{}

unsafe impl Sync for HttpServer{}
// impl<'static> Clone for HttpServer<'static>{
//     fn clone(&self) -> Self {
//         HttpServer{
//             dispatcher: (),
//             _marker_e: Default::default(),
//             _marker_a: Default::default()
//         }
//     }
// }

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

// fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let body = async {
//         let addr = ([127, 0, 0, 1], 3000).into();
//         let service = make_service_fn(|_conn: &AddrStream| {
//             let a = _conn.clone();
//             ::std::io::_print(::core::fmt::Arguments::new_v1(
//                 &["", "\n"],
//                 &[::core::fmt::ArgumentV1::new_debug(&a)],
//             ));
//             async move { Ok::<_, hyper::Error>(service_fn(echo)) }
//         });
//         let server = Server::bind(&addr).serve(service);
//         ::std::io::_print(::core::fmt::Arguments::new_v1(
//             &["Listening on http://", "\n"],
//             &[::core::fmt::ArgumentV1::new_display(&addr)],
//         ));
//         server.await?;
//         Ok(())
//     };
//     #[allow(clippy::expect_used)]
//     tokio::runtime::Builder::new_multi_thread()
//         .enable_all()
//         .build()
//         .expect("Failed building the Runtime")
//         .block_on(body)
// }


impl HttpServer{
    pub async fn start(mut self) -> CellResult<()> {
        let addr = ([127, 0, 0, 1], 3000).into();
        let s1 = Arc::new(self);
        let service = make_service_fn(|_conn: &AddrStream| {
            let a = _conn.clone();
            println!("{:?}", a);
            //         service_fn(move |req| {
            //             let http_req=HttpRequest::new(req);
            // let http_resp=HttpResponse::new();
            //             self.dispatch(http_req,http_resp);
            //             // cell_hyper_service_fn(req)
            //         });
            let s2 = s1.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    // c()
                    async_hyper_service_fn(s2.clone(),req)

                    // let (tx, rx) = oneshot::channel();
                    // let http_req = HttpRequest::new(req);
                    // let http_resp = HttpResponse::new(tx);
                    // let ctx = DispatchContext::new(Box::new(http_req), Box::new(http_resp));
                    // self.dispatcher.dispatch(ctx);

                    // async_hyper_service_fn(s2.clone(),req)

                    // let (tx, rx) = oneshot::channel();
                    // tokio::spawn(async move {
                    //     let server=s2.clone();
                    //     // hyper_service_fn(req, tx)
                    //     rx.await.map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
                    // });


                    // let (txx, mut rxx) = std::sync::mpsc::channel::<Response<Body>>();
                    // let (tx, rx) = oneshot::channel();
                    // tokio::spawn(async move {
                    //     let http_req=HttpRequest::new(req);
                    //     let http_resp=HttpResponse::new(tx);
                    //     let ctx=DispatchContext::new(Box::new(http_req),Box::new(http_resp));
                    //     self.dispatcher.dispatch(ctx);
                    // });
                    // // self.dispatcher.dispatch(ctx);
                    // // ss.dispatch(http_req, http_resp);
                    // // cell_hyper_service_fn(req)
                    // // self.hyper_service_fn(req)
                    // // rxx.recv()
                    // rx.await.map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e))
                    // c()
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

    pub fn dispatch(mut self, req: HttpRequest, resp: HttpResponse) {
        let mut a = self.dispatcher;
        let ctx = DispatchContext::new(Box::new(req), Box::new(resp));
        a.dispatch(ctx);
    }
    pub fn new(dispatcher: DefaultDispatcher<'static,'static>) -> Self {
        Self { dispatcher,
        }
    }
}

pub async fn async_hyper_service_fn(mut server: Arc<HttpServer>, req: Request<Body>) -> Result<Response<Body>, std::io::Error> {
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move {
        let http_req = Box::new(HttpRequest::new(req));
        let http_resp = Box::new(HttpResponse::new(Arc::new(tx)));
        let ctx = DispatchContext::new(http_req, http_resp);
        // println!("{:?}",server);
        server.dispatcher.dispatch(ctx).await;
    });
    let ret=rx.await.map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e));
    match ret {
        Ok(V)=>{
            println!("success");
            Ok(V)
        },
        Err(e)=>{
            println!("failed:{}",e);
            Err(e)
        }
    }
}


async fn c() -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from(String::from("asd"))))
}


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
    use cell_core::command::mock_command;
    use cell_core::dispatcher::DefaultDispatcher;
    use cell_core::selector::CommandSelector;
    use crate::channel::HttpChannel;
    use crate::dispatcher::HttpDispatcher;
    use crate::selector::HttpSelector;
    use crate::server::HttpServer;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_http_server() {
        let mut selector = HttpSelector::default();
        let cmd=mock_command();
        selector.on_register_cmd(cmd);
        let channel = HttpChannel::default();
        let http_dispatch = HttpDispatcher::new();
        let default_dispatcher = DefaultDispatcher::new(
            Box::new(channel),
            Box::new(selector),
            Box::new(http_dispatch));
        let s = HttpServer::new(default_dispatcher);
        let body = async {
            s.start().await
        };
        // futures::executor::block_on();

        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}