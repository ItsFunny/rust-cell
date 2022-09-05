use std::cell::RefCell;
use std::fmt::{Debug, Error, Formatter};
use std::future::Future;
use std::io;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::Arc;
use futures::future::ok;
use futures::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper::server::conn::AddrStream;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Sender;
use cell_core::cerror::{CellError, CellResult, ErrorEnums, ErrorEnumsStruct};
use cell_core::channel::ChannelTrait;
use cell_core::dispatcher::{DefaultDispatcher, DispatchContext};
use cell_core::extension::NodeContext;
use cell_core::request::MockRequest;
use cell_core::selector::{CommandSelector, SelectorStrategy};
use logsdk::{cerror, cinfo, module_enums};
use logsdk::common::LogLevel;
use crate::channel::HttpChannel;
use crate::dispatcher::HttpDispatcher;
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use crate::selector::HttpSelector;



module_enums!(
        (HTTP_SERVER,1,&logsdk::common::LogLevel::Info);
    );

pub struct HttpServer {
    dispatcher: DefaultDispatcher<'static, 'static>,
}

pub struct HttpServerBuilder {
    selector: Option<Box<dyn CommandSelector<'static>>>,
}

impl Default for HttpServerBuilder {
    fn default() -> Self {
        HttpServerBuilder { selector: None }
    }
}

impl HttpServerBuilder {
    pub fn with_selector(mut self, se: Box<dyn CommandSelector<'static>>) -> Self {
        self.selector = Some(se);
        self
    }
    pub fn build(self) -> HttpServer {
        let mut default_http_selector = Box::new(HttpSelector::default());
        let mut executors: Vec<Box<dyn CommandSelector>> = Vec::new();
        if let Some(v) = self.selector {
            executors.push(v);
        }
        executors.push(default_http_selector);

        let mut selector_strategy = SelectorStrategy::new(executors);
        let channel = HttpChannel::default();
        let http_dispatch = HttpDispatcher::new();
        let default_dispatcher = DefaultDispatcher::new(
            Box::new(channel),
            selector_strategy,
            Box::new(http_dispatch));
        HttpServer { dispatcher: default_dispatcher }
    }
}

impl Default for HttpServer {
    fn default() -> Self {
        let mut selector = HttpSelector::default();
        let vec_executors: Vec<Box<dyn CommandSelector>> = vec![Box::new(selector)];
        let mut selector_strategy = SelectorStrategy::new(vec_executors);
        let channel = HttpChannel::default();
        let http_dispatch = HttpDispatcher::new();
        let default_dispatcher = DefaultDispatcher::new(
            Box::new(channel),
            selector_strategy,
            Box::new(http_dispatch));
        HttpServer::new(default_dispatcher)
    }
}

impl Debug for HttpServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "asd")
    }
}


unsafe impl Send for HttpServer {}

unsafe impl Sync for HttpServer {}


impl HttpServer {
    pub async fn start(self) -> CellResult<()> {
        let addr = ([127, 0, 0, 1], 3000).into();
        let s1 = Arc::new(self);
        let service = make_service_fn(|_conn: &AddrStream| {
            let addr = _conn.remote_addr();
            let s2 = s1.clone();
            async move {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    async_hyper_service_fn(s2.clone(), req, addr)
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
    pub fn init(&mut self, ctx: Arc<RefCell<NodeContext>>) {
        self.dispatcher.init(ctx);
    }

    pub fn dispatch(mut self, req: HttpRequest, resp: HttpResponse) {
        let mut a = self.dispatcher;
        let ctx = DispatchContext::new(Box::new(req), Box::new(resp));
        a.dispatch(ctx);
    }
    pub fn new(dispatcher: DefaultDispatcher<'static, 'static>) -> Self {
        Self {
            dispatcher,
        }
    }
}


pub struct ChannelWrapper {
    Err: Option<io::Error>,
    Ret: Option<Response<Body>>,
}

pub async fn async_hyper_service_fn(mut server: Arc<HttpServer>, req: Request<Body>, remote_addr: SocketAddr) -> Result<Response<Body>, std::io::Error> {
    let (tx, rx) = oneshot::channel();
    let (txx, rxx) = std::sync::mpsc::channel::<Response<Body>>();
    tokio::spawn(async move {
        let http_req = Box::new(HttpRequest::new(req, remote_addr.ip().to_string()));
        let http_resp = Box::new(HttpResponse::new(txx));
        let ctx = DispatchContext::new(http_req, http_resp);
        server.dispatcher.dispatch(ctx).await;
        let rrr = rxx.recv().map_err(|e| {
            io::Error::new(io::ErrorKind::BrokenPipe, e)
        });
        let ret: ChannelWrapper;
        match rrr {
            Ok(v) => {
                ret = ChannelWrapper { Err: None, Ret: Some(v) };
            }
            Err(e) => {
                ret = ChannelWrapper { Err: Some(e), Ret: None };
            }
        }
        tx.send(ret);
    });
    let ret = rx.await.map_err(|e| io::Error::new(io::ErrorKind::BrokenPipe, e));
    match ret {
        Ok(v) => {
            if let Some(e) = v.Err {
                cerror!(ModuleEnumsStruct::HTTP_SERVER,"调用失败:{}",e.to_string());
                Err(e)
            } else {
                Ok(v.Ret.unwrap())
            }
        }
        Err(e) => {
            cerror!(ModuleEnumsStruct::HTTP_SERVER,"调用失败:{}",e.to_string());
            Err(e)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::thread::Thread;
    use std::time::Duration;
    use cell_core::command::mock_command;
    use cell_core::dispatcher::DefaultDispatcher;
    use cell_core::selector::{CommandSelector, SelectorRequest, SelectorStrategy};
    use pipeline2::pipeline2::{ClosureExecutor, DefaultReactorExecutor, PipelineBuilder};
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
        let vec_executors: Vec<Box<dyn CommandSelector>> = vec![Box::new(selector)];
        let mut selector_strategy = SelectorStrategy::new(vec_executors);
        let cmd = mock_command();
        selector_strategy.on_register_cmd(cmd);
        let channel = HttpChannel::default();
        let http_dispatch = HttpDispatcher::new();
        let default_dispatcher = DefaultDispatcher::new(
            Box::new(channel),
            selector_strategy,
            Box::new(http_dispatch));
        let s = HttpServer::new(default_dispatcher);
        let body = async {
            s.start().await
        };


        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed building the Runtime")
            .block_on(body);
    }
}