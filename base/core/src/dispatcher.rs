use core::cell::RefCell;
use core::ops::Deref;
use std::arch;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use http::Response;
use hyper::Body;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};
use crate::channel::ChannelTrait;
use crate::command::{Command, CommandContext, mock_context};
use crate::context::{BaseBuzzContext, BuzzContextTrait, ContextWrapper};
use crate::core::{ExecutorValueTrait, ModuleEnumsStruct, ProtocolID};
use crate::request::{ServerRequestTrait, ServerResponseTrait};
use crate::selector::{CommandSelector, SelectorRequest, SelectorStrategy};


pub trait Dispatcher: Send + Sync {
    fn get_info<'a>(&self, req: Arc<Box<dyn ServerRequestTrait + 'a>>, resp: Box<dyn ServerResponseTrait + 'a>, cmd: &Command<'a>) -> Box<dyn BuzzContextTrait<'a> + 'a>;
}


pub struct DefaultDispatcher<'e: 'a, 'a>
{
    channel: Box<dyn ChannelTrait<'e, 'a> + 'e>,
    command_selector: SelectorStrategy<'e>,
    dispatcher: Box<dyn Dispatcher + 'e>,
}

impl<'e: 'a, 'a> DefaultDispatcher<'e, 'a> where
{
    pub fn new(channel: Box<dyn ChannelTrait<'e, 'a>>, command_selector: SelectorStrategy<'e>, dis: Box<dyn Dispatcher + 'e>) -> Self {
        Self { channel, command_selector, dispatcher: dis }
    }
}

pub struct DispatchContext<'a> {
    pub req: Box<dyn ServerRequestTrait + 'a>,
    pub resp: Box<dyn ServerResponseTrait + 'a>,
}

unsafe impl<'a> Send for DispatchContext<'a> {}

unsafe impl<'a> Sync for DispatchContext<'a> {}

impl<'a> DispatchContext<'a> {
    pub fn new(req: Box<dyn ServerRequestTrait + 'a>, resp: Box<dyn ServerResponseTrait + 'a>) -> Self {
        Self { req, resp }
    }
}

impl<'e: 'a, 'a> DefaultDispatcher<'e, 'a>
{
    #[inline]
    pub async fn dispatch(&self, mut ctx: DispatchContext<'a>) {
        let req_rc = Arc::new(ctx.req);
        // TODO ,resp need wrapped by rc
        let mut resp = ctx.resp;
        let cmd_res = self.get_cmd_from_request(req_rc.clone());

        let cmd: Command;
        if let Some(c) = cmd_res {
            cmd = c;
        } else {
            cerror!(ModuleEnumsStruct::DISPATCHER,"command not exists,ip:{},protocol:{}",req_rc.get_ip(),req_rc.get_string_protocol());
            resp.fire_result(Response::new(Body::from(ErrorEnumsStruct::COMMAND_NOT_EXISTS.get_msg())));
            return;
        }
        let b_ctx: Box<dyn BuzzContextTrait + 'a> = self.dispatcher.get_info(req_rc.clone(), resp, &cmd);
        self.channel.read_command(ContextWrapper::new(b_ctx, Arc::new(cmd))).await
    }


    pub fn get_cmd_from_request(&self, req: Arc<Box<dyn ServerRequestTrait + 'a>>) -> Option<Command<'a>> {
        // TODO ,useless
        let (txx, mut rxx) = std::sync::mpsc::channel::<Command>();
        let req = SelectorRequest::new(req, txx);
        self.command_selector.select(&req)
    }
}

pub struct MockDispatcher {}

impl Dispatcher for MockDispatcher {
    fn get_info<'a>(&self, req: Arc<Box<dyn ServerRequestTrait + 'a>>, resp: Box<dyn ServerResponseTrait + 'a>, cmd: &Command<'a>) -> Box<dyn BuzzContextTrait<'a> + 'a> {
        let (c, rxx, ctx) = mock_context();
        let res: Box<dyn BuzzContextTrait<'a>> = Box::new(ctx);
        res
    }
}


#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use std::sync::Arc;
    use bytes::Bytes;
    use http::header::HeaderName;
    use http::Response;
    use hyper::Body;
    use tokio::runtime::Runtime;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use crate::cerror::CellResult;
    use crate::channel::mock_channel;
    use crate::command::{Command, CommandContext, mock_context};
    use crate::constants::ProtocolStatus;
    use crate::context::BaseBuzzContext;
    use crate::core::{ProtocolID, RunType};
    use crate::dispatcher::{DefaultDispatcher, DispatchContext, MockDispatcher};
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::selector::{CommandSelector, MockDefaultPureSelector, SelectorStrategy};
    use crate::summary::{Summary, SummaryTrait};
    use crate::wrapper::ContextResponseWrapper;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_command() {
        let c = Command::default();
        let c2 = c.with_run_type(1 as RunType);
    }

    #[test]
    fn test_dispatcher() {
        let (txx, mut rxx) = std::sync::mpsc::channel::<Response<Body>>();
        let channel = mock_channel();
        let selector = MockDefaultPureSelector::new();
        let mock_dispatcher = MockDispatcher {};
        let vec_executors: Vec<Box<dyn CommandSelector>> = vec![Box::new(selector)];
        let selector = SelectorStrategy::new(vec_executors);
        let mut dispatcher = DefaultDispatcher::new(Box::new(channel), selector, Box::new(mock_dispatcher));
        let req = Box::new(MockRequest::new());
        let resp = Box::new(MockResponse::new(txx));
        let ctx = DispatchContext::new(req, resp);
        futures::executor::block_on(dispatcher.dispatch(ctx));
    }
}