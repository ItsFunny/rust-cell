use crate::constants::ProtocolStatus;
use crate::context::{BaseBuzzContext, BuzzContextTrait};
use crate::core::{AliasRequestType, AliasResponseType, ExecutorValueTrait, ProtocolID, RunType};
use crate::output::{OutputArchive, Serializable};
use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
use crate::response::MockResponse;
use crate::summary::{Summary, SummaryTrait};
use crate::wrapper::ContextResponseWrapper;
use bytes::Bytes;
use http::Response;
use hyper::Body;
use logsdk::common::LogLevel;
use logsdk::log4rs::DEFAULT_LOGGER;
use logsdk::module;
use logsdk::module::CellModule;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

pub type Function = dyn Fn(&mut dyn BuzzContextTrait, Option<&dyn ExecutorValueTrait>);

// pub trait Func<'a>: Sync + Send {
//     fn handle(&self, c: Box< dyn BuzzContextTrait+'a>, t: Option<&dyn ExecutorValueTrait>);
// }

unsafe impl Sync for ClosureFunc<'_> {}

unsafe impl Send for ClosureFunc<'_> {}

pub struct ClosureFunc<'a> {
    f: Arc<dyn Fn(&mut dyn BuzzContextTrait, Option<&dyn ExecutorValueTrait>)>,
    _marker_e: PhantomData<&'a ()>,
}

impl<'a> ClosureFunc<'a> {
    pub fn new(f: Arc<dyn Fn(&mut dyn BuzzContextTrait, Option<&dyn ExecutorValueTrait>)>) -> Self {
        Self {
            f,
            _marker_e: Default::default(),
        }
    }
}

impl<'a> ClosureFunc<'a> {
    fn handle(&self, c: &mut dyn BuzzContextTrait, t: Option<&dyn ExecutorValueTrait>) {
        (self.f)(c, t)
    }
}

pub trait CommandTrait: Clone {
    fn id(&self) -> ProtocolID;
    fn execute(&self, ctx: &mut dyn BuzzContextTrait);
    // fn to_command<'a>(&self) -> Command<'a>;
}

pub struct Command<'a> {
    pub protocol_id: ProtocolID,
    pub fun: Option<Arc<ClosureFunc<'a>>>,
    pub meta_data: MetaData,
    pub run_type: RunType,
    seal: bool,
}

pub fn mock_command<'a>() -> Command<'a> {
    let mut c = Command::default();
    let f = ClosureFunc::new(Arc::new(move |mut ctx, v| {
        println!("execute");
        let mut ret = ContextResponseWrapper::default();
        ret = ret.with_status(ProtocolStatus::SUCCESS);
        ret = ret.with_body(Bytes::from("123"));
        ctx.response(ret);
    }));
    c = c.with_protocol_id("/protocol").with_executor(Arc::new(f));
    return c;
}

pub fn mock_command2<'a>() -> Command<'a> {
    let mut c = Command::default();
    let f = ClosureFunc::new(Arc::new(move |mut ctx, v| {
        println!("execute");
        let mut ret = ContextResponseWrapper::default();
        ret = ret.with_status(ProtocolStatus::SUCCESS);
        ret = ret.with_body(Bytes::from("123"));
        ctx.response(ret);
    }));
    c = c.with_protocol_id("/protocol").with_executor(Arc::new(f));
    return c;
}

impl<'a> Clone for Command<'a> {
    fn clone(&self) -> Self {
        Command {
            protocol_id: self.protocol_id.clone(),
            fun: self.fun.clone(),
            meta_data: Default::default(),
            run_type: self.run_type,
            seal: false,
        }
    }
}

pub struct MetaData {
    pub asy: bool,
    pub request_type: AliasRequestType,
    pub response_type: AliasResponseType,
}

impl Clone for MetaData {
    fn clone(&self) -> Self {
        MetaData {
            asy: self.asy,
            request_type: self.request_type,
            response_type: self.response_type,
        }
    }
}

impl Default for MetaData {
    fn default() -> Self {
        MetaData {
            asy: false,
            request_type: 0,
            response_type: 0,
        }
    }
}

impl<'a> Command<'a> {
    pub fn with_protocol_id(mut self, p: ProtocolID) -> Self {
        self.protocol_id = p;
        self
    }
    // pub fn with_executor(mut self, e: &'static Function) -> Self {
    //     self.fun = Some(e);
    //     self
    // }
    pub fn with_executor(mut self, e: Arc<ClosureFunc<'a>>) -> Self {
        self.fun = Some(e);
        self
    }
    pub fn with_meta_data(mut self, m: MetaData) -> Self {
        self.meta_data = m;
        self
    }
    pub fn with_run_type(mut self, r: RunType) -> Self {
        self.run_type = r;
        self
    }
    pub fn do_seal(mut self) -> Self {
        self.seal = true;
        self
    }
}

impl MetaData {
    pub fn with_asy(mut self, asy: bool) -> Self {
        self.asy = asy;
        self
    }
    pub fn with_request_type(mut self, r: AliasRequestType) -> Self {
        self.request_type = r;
        self
    }

    pub fn with_response_type(mut self, r: AliasRequestType) -> Self {
        self.response_type = r;
        self
    }
}

pub struct CommandContext<'a> {
    pub module: &'static CellModule,
    pub server_request: Arc<Box<dyn ServerRequestTrait + 'a>>,
    // TODO REFCELL
    pub server_response: Box<dyn ServerResponseTrait + 'a>,
    // TODO, ARC
    pub summary: Box<dyn SummaryTrait + 'a>,
    // TODO
    // pub channel: &dyn ChannelTrait,
    // pub command: &'static dyn CommandTrait,
    // TODO: ops
}

impl<'a> Default for Command<'a> {
    fn default() -> Self {
        Command {
            protocol_id: "",
            fun: None,
            meta_data: Default::default(),
            run_type: 0,
            seal: false,
        }
    }
}

impl<'a> CommandContext<'a> {
    pub fn new(
        module: &'static CellModule,
        server_request: Arc<Box<dyn ServerRequestTrait + 'a>>,
        server_response: Box<dyn ServerResponseTrait + 'a>,
        st: Box<dyn SummaryTrait + 'a>,
    ) -> Self {
        CommandContext {
            module,
            server_request,
            server_response,
            summary: st,
        }
    }
}

////////////

impl<'a> CommandTrait for Command<'a> {
    // impl<'a> Command<'a> {
    fn id(&self) -> ProtocolID {
        self.protocol_id.clone()
    }

    fn execute(&self, ctx: &mut dyn BuzzContextTrait) {
        // TODO input archive
        // TODO NOE
        // (self.fun).unwrap()(ctx, None)
        // let a=self.fun.unwrap();
        self.fun.as_ref().unwrap().handle(ctx, None)
    }
}

pub fn mock_context<'a>() -> (
    Command<'a>,
    std::sync::mpsc::Receiver<Response<Body>>,
    BaseBuzzContext<'a>,
) {
    let (txx, mut rxx) = std::sync::mpsc::channel::<Response<Body>>();
    let p: ProtocolID = "/protocol/v1" as ProtocolID;
    let mut c = mock_command();

    static M: &CellModule = &module::CellModule::new(1, "CONTEXT", &LogLevel::Info);
    let box_request: Box<dyn ServerRequestTrait> = Box::new(MockRequest::new());
    let req = Arc::new(box_request);
    let resp = Box::new(MockResponse::new(txx));
    let ip = String::from("128");
    let sequence_id = String::from("seq");
    let summ = Box::new(Summary::new(Arc::new(ip), Arc::new(sequence_id), p));
    let c_ctx: CommandContext = CommandContext::new(M, req, resp, summ);
    let mut ctx = BaseBuzzContext::new(32, c_ctx);
    return (c, rxx, ctx);
}

#[cfg(test)]
mod tests {
    use crate::command::{mock_context, Command, CommandContext, CommandTrait};
    use crate::constants::ProtocolStatus;
    use crate::context::BaseBuzzContext;
    use crate::core::ProtocolID;
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::summary::{Summary, SummaryTrait};
    use crate::wrapper::ContextResponseWrapper;
    use bytes::Bytes;
    use http::header::HeaderName;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use pipeline2::pipeline2::is_send;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_command() {
        let (c, rxx, mut ctx) = mock_context();

        // futures::executor::block_on(c.execute(&mut ctx));
        c.execute(&mut ctx);
        is_send(c);
        println!("111111111");

        let ret = rxx.recv();
        match ret {
            Ok(vv) => {
                println!("执行成功:{:?}", vv)
            }
            Err(e) => {
                println!("执行失败:{:?}", e)
            }
        }
    }
}
