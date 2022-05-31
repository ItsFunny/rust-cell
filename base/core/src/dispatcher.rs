use core::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use pipeline::executor::ExecutorValueTrait;
use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};
use crate::channel::ChannelTrait;
use crate::command::CommandTrait;
use crate::core::ProtocolID;
use crate::request::{ServerRequestTrait, ServerResponseTrait};
use crate::selector::{CommandSelector, SelectorRequest};
use crate::suit::CommandSuit;


pub type CreateSuit<'a, T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a, > = dyn Fn(Rc<Box<dyn ServerRequestTrait + 'a>>, Box<dyn ServerResponseTrait + 'a>) -> CellResult<&'a T>;

pub struct DefaultDispatcher<'e: 'a, 'a, T>
    where
        Self: 'static,
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    commands: HashMap<String, Box<dyn CommandTrait>>,
    channel: Box<dyn ChannelTrait<'e, 'a, T>>,
    suit_func: &'static CreateSuit<'a, T>,
    command_selector: Box<dyn CommandSelector>,
}

pub struct DispatchContext<'a> {
    pub req: Box<dyn ServerRequestTrait + 'a>,
    pub resp: Box<dyn ServerResponseTrait + 'a>,
}

impl<'e: 'a, 'a, T> DefaultDispatcher<'e, 'a, T>
    where
        Self: 'static,
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    pub fn dispatch(&'e mut self, ctx: DispatchContext<'a>) {
        let req_rc = Rc::new(ctx.req);
        // TODO ,resp need wrapped by rc
        let resp = ctx.resp;
        let cmd_res = self.get_cmd_from_request(Rc::clone(&req_rc));

        let cmd;
        match cmd_res {
            Err(e) => {
                // TODO ,log
            }
            Ok(c) => {
                cmd = c;
            }
        }
        let suit;

        let suit_resp = (self.suit_func)(Rc::clone(&req_rc), resp);
        // TODO:
        match suit_resp {
            Err(e) => {
                return;
            }
            Ok(v) => {
                suit = v;
                // TODO fill argu
                self.channel.read_command(&suit)
            }
        }
    }


    pub fn get_cmd_from_request(&self, req: Rc<Box<dyn ServerRequestTrait>>) -> CellResult<Box<dyn CommandTrait>> {
        let (txx, mut rxx) = std::sync::mpsc::channel::<&'static dyn CommandTrait>();
        let req = SelectorRequest::new(req, txx);
        self.command_selector.select(&req);
        let ret = rxx.recv();
        Err(CellError::from(ErrorEnumsStruct::JSON_SERIALIZE))
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use bytes::Bytes;
    use http::header::HeaderName;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use crate::command::{Command, CommandContext, CommandTrait};
    use crate::constants::ProtocolStatus;
    use crate::context::BaseBuzzContext;
    use crate::core::ProtocolID;
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::summary::{Summary, SummaryTrait};
    use crate::wrapper::ContextResponseWrapper;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}