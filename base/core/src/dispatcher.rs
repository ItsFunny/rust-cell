use core::cell::RefCell;
use core::ops::Deref;
use std::collections::HashMap;
use std::rc::Rc;
use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};
use crate::channel::ChannelTrait;
use crate::command::{Command, CommandContext, mock_context};
use crate::context::{BaseBuzzContext, BuzzContextTrait, ContextWrapper};
use crate::core::{ExecutorValueTrait, ProtocolID};
use crate::request::{ServerRequestTrait, ServerResponseTrait};
use crate::selector::{CommandSelector, SelectorRequest};
use crate::suit::{CommandSuit, DefaultCommandSuit};


pub trait Dispatcher {
    fn get_info<'a>(&self, req: Rc<Box<dyn ServerRequestTrait + 'a>>, resp: Box<dyn ServerResponseTrait + 'a>) -> CellResult<Box<dyn BuzzContextTrait<'a> + 'a>>;
}

pub struct DefaultDispatcher<'e: 'a, 'a>
{
    channel: Box<dyn ChannelTrait<'e, 'a> + 'e>,
    command_selector: Box<dyn CommandSelector + 'e>,
    dispatcher: Box<dyn Dispatcher + 'e>,
}

impl<'e: 'a, 'a> DefaultDispatcher<'e, 'a> where
{
    pub fn new(channel: Box<dyn ChannelTrait<'e, 'a>>, command_selector: Box<dyn CommandSelector>, dis: Box<dyn Dispatcher + 'e>) -> Self {
        Self { channel, command_selector, dispatcher: dis }
    }
}

pub struct DispatchContext<'a> {
    pub req: Box<dyn ServerRequestTrait + 'a>,
    pub resp: Box<dyn ServerResponseTrait + 'a>,
}

impl<'a> DispatchContext<'a> {
    pub fn new(req: Box<dyn ServerRequestTrait + 'a>, resp: Box<dyn ServerResponseTrait + 'a>) -> Self {
        Self { req, resp }
    }
}

impl<'e: 'a, 'a> DefaultDispatcher<'e, 'a>
{
    pub async fn dispatch(&mut self, ctx: DispatchContext<'a>) {
        let req_rc = Rc::new(ctx.req);
        // TODO ,resp need wrapped by rc
        let resp = ctx.resp;
        let cmd_res = self.get_cmd_from_request(req_rc.clone());

        let cmd;
        match cmd_res {
            Err(e) => {
                // TODO ,log
                panic!("asd")
            }
            Ok(c) => {
                cmd = c;
            }
        }
        let b_ctx: Box<dyn BuzzContextTrait + 'a>;
        let command_ctx_res = self.dispatcher.get_info(Rc::clone(&req_rc), resp);
        match command_ctx_res {
            Err(e) => {
                // TODO
                panic!("as")
            }
            Ok(v) => {
                b_ctx = v;
            }
        }
        self.channel.read_command(ContextWrapper::new(b_ctx, Rc::new(cmd))).await
        // let bbb=b_ctx.as_ref();
        // let a: &'a dyn BuzzContextTrait = b_ctx.deref();
        // let suit = DefaultCommandSuit::new(bbb);
        // self.channel.read_command(&suit);
        // let suit_resp = (self.suit_func)(Rc::clone(&req_rc), resp);
        // // TODO:
        // match suit_resp {
        //     Err(e) => {
        //         return;
        //     }
        //     Ok(v) => {
        //         suit = v;
        //         // TODO fill argu
        //         self.channel.read_command(&suit)
        //     }
        // }
    }


    pub fn get_cmd_from_request(&self, req: Rc<Box<dyn ServerRequestTrait + 'a>>) -> CellResult<Command> {
        // TODO ,useless
        let (txx, mut rxx) = std::sync::mpsc::channel::<Command>();
        // let (tx, rx) = oneshot::channel();
        let req = SelectorRequest::new(req, txx);
        let ret = self.command_selector.select(&req);
        match ret {
            None => {
                Err(CellError::from(ErrorEnumsStruct::UNKNOWN))
            }
            Some(v) => {
                Ok(v)
            }
        }
    }
}

pub struct MockDispatcher {}

impl Dispatcher for MockDispatcher {
    fn get_info<'a>(&self, req: Rc<Box<dyn ServerRequestTrait + 'a>>, resp: Box<dyn ServerResponseTrait + 'a>) -> CellResult<Box<dyn BuzzContextTrait<'a> + 'a>> {
        let (c, rxx, ctx) = mock_context();
        let res: Box<dyn BuzzContextTrait<'a>> = Box::new(ctx);
        Ok(res)
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
    use crate::selector::MockDefaultPureSelector;
    use crate::suit::{DefaultCommandSuit, EmptyCommandSuite};
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
        let mut dispatcher = DefaultDispatcher::new(Box::new(channel), Box::new(selector), Box::new(mock_dispatcher));
        let req = Box::new(MockRequest::new());
        let resp = Box::new(MockResponse::new(txx));
        let ctx = DispatchContext::new(req, resp);
        futures::executor::block_on(dispatcher.dispatch(ctx));
    }
}