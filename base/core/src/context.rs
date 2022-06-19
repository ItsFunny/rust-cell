use std::borrow::Borrow;
use std::error::Error;
use std::fmt::{Debug, Formatter, write};
use std::io;
use std::rc::Rc;
use std::sync::Arc;
use chrono::Local;
use http::header::{CONTENT_LENGTH, HeaderName};
use futures;
use http::{HeaderValue, Response};
use hyper::Body;
use rocket::form::validate::len;
use rocket::futures::StreamExt;
use tokio::runtime::Handle;
use logsdk::{cinfo, log4rs, module};
use logsdk::common::LogLevel;
use crate::command::{Command, CommandContext};
use crate::wrapper::ContextResponseWrapper;
use async_trait::async_trait;
use bytes::{Buf, Bytes};
use logsdk::module::CellModule;
use pipeline2::pipeline2::DefaultPipelineV2;
use crate::cerror::CellResult;
use crate::core::ProtocolID;
use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
use crate::response::MockResponse;
use crate::summary::{Summary, SummaryTrait};


pub trait Context {
    fn discard(&mut self) {
        // do nothing
    }
    fn done(&mut self) -> bool;
    // fn unsafe_notify_done();
}


pub struct ContextWrapper<'a> {
    pub ctx: Box<dyn BuzzContextTrait<'a> + 'a>,
    // note: The reason I use rc instead of using box is "I dont have to clone data ,all I want is pointer ,it is enough"
    pub cmd: Arc<Command<'a>>,
}


impl<'a> ContextWrapper<'a> {
    pub fn new(ctx: Box<dyn BuzzContextTrait<'a> + 'a>, cmd: Arc<Command<'a>>) -> Self {
        Self { ctx, cmd }
    }
}

impl<'a> Debug for ContextWrapper<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "asd")
    }
}

pub trait BuzzContextTrait<'a>: Context + Send + Sync {
     fn response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()>;
     fn on_response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()>;
}

pub struct BaseBuzzContext<'a> {
    pub request_timestamp: i64,
    pub command_context: CommandContext<'a>,

    // pub concrete: Box<dyn BuzzContextTrait>,
}

unsafe impl<'a> Sync for BaseBuzzContext<'a> {}

impl<'a> Debug for BaseBuzzContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> Context for BaseBuzzContext<'a> {
    fn discard(&mut self) {
        todo!()
    }

    fn done(&mut self) -> bool {
        todo!()
    }
}

impl<'a> BaseBuzzContext<'a> {
    fn sync_response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()> {
        let now = Local::now().timestamp();
        let consume_time = now - self.request_timestamp;
        let sequence_id = self.command_context.summary.get_sequence_id();
        cinfo!(self.command_context.module,
            "response protocol={}, ip={},sequenceId={},cost={}",
            self.command_context.summary.get_protocol_id(),self.command_context.summary.get_request_ip(),
            sequence_id,consume_time,
        );

        let mut mut_resp = resp.borrow();
        // TODO status
        let status = resp.status();

        // TODO , fired or not

        // TODO , timeout?

        // TODO
        for header in mut_resp.headers().iter() {
            let key = header.0.as_str();
            let value = header.1.as_bytes();
            // hyp_res = hyp_res.header(key, value);
            let h_name: HeaderName;
            let h_value: HeaderValue;
            let name_res = HeaderName::try_from(key);
            let value_res = HeaderValue::try_from(value);
            match name_res {
                Ok(v) => {
                    h_name = v;
                }
                Err(e) => { continue; }
            }
            match value_res {
                Ok(v) => {
                    h_value = v;
                }
                Err(e) => { continue; }
            }

            self.command_context.server_response.add_header(h_name, h_value);
        }

        let body_opt = resp.body_mut();
        match body_opt {
            Some(body) => {
                let length_value = HeaderValue::try_from(body.len()).unwrap();
                self.command_context.server_response.add_header(CONTENT_LENGTH, length_value);
                let bbb = Body::from(body);
                let fire_resp = Response::builder().body(bbb).unwrap();
                self.command_context.server_response.fire_result(fire_resp)?
            }
            None => {
                // TODO
            }
        }

        Ok(())
    }
    pub fn new(request_timestamp: i64, command_context: CommandContext<'a>) -> Self {
        BaseBuzzContext { request_timestamp, command_context }
    }
}

impl<'a> BuzzContextTrait<'a> for BaseBuzzContext<'a> {
    fn response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()> {
        self.sync_response(resp)
    }

    fn on_response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use std::arch;
    use std::rc::Rc;
    use std::sync::Arc;
    use bytes::Bytes;
    use http::Response;
    use hyper::Body;
    use tokio::sync::oneshot;
    use tokio::sync::oneshot::{channel, Sender};
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::{CellModule, Module};
    use crate::command::{CommandContext, mock_context};
    use crate::context::{BaseBuzzContext, BuzzContextTrait};
    use crate::core::ProtocolID;
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::summary::{Summary, SummaryTrait};
    use crate::wrapper::ContextResponseWrapper;
    use crate::output::*;
    use serde::{Deserialize, Serialize};
    use crate::cerror::CellResult;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct AARet {
        name: String,
    }

    impl<'a> Serializable<'a> for AARet {}

    #[test]
    fn test_command_context() {
        let (c, rxx, mut ctx) = mock_context();

        let body = Body::from(String::from("asd"));
        let mut wrapper = ContextResponseWrapper::default();

        let bs = as_json_bytes(AARet { name: "charlie".to_string() });
        match bs {
            Ok(data) => {
                wrapper = wrapper.with_body(data);
            }
            _ => {
                wrapper = wrapper.with_body(Bytes::from("fail"))
            }
        }

        let r = tokio::runtime::Runtime::new().unwrap();
        futures::executor::block_on(ctx.response(wrapper));

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


    #[test]
    fn test_with_hyper() {
        // let addr = ([127, 0, 0, 1], 3000).into();
    }
}
