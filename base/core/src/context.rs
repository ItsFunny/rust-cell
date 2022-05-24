use std::error::Error;
use std::fmt::{Debug, Formatter, write};
use std::io;
use std::rc::Rc;
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
use crate::command::CommandContext;
use crate::wrapper::ContextResponseWrapper;
use async_trait::async_trait;


pub trait Context {
    fn discard(&mut self);
    fn done(&self) -> bool;
    // fn unsafe_notify_done();
}


#[async_trait]
pub trait BuzzContextTrait<'a>: Context {
    async fn response(self, resp: &mut ContextResponseWrapper<'a>) -> io::Result<()>;
    async fn on_response(self, resp: &mut ContextResponseWrapper<'a>) -> io::Result<()>;
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

    fn done(&self) -> bool {
        todo!()
    }
}

impl<'a> BaseBuzzContext<'a> {
    async fn async_response(mut self, resp: &mut ContextResponseWrapper<'_>) -> io::Result<()> {
        let now = Local::now().timestamp();
        let consume_time = now - self.request_timestamp;
        let sequence_id = self.command_context.summary.get_sequence_id();
        cinfo!(self.command_context.module,
            "response protocol={}, ip={},sequenceId={},cost={}",
            self.command_context.summary.get_protocol_id(),self.command_context.summary.get_request_ip(),
            sequence_id,consume_time,
        );

        // TODO
        for header in resp.headers().iter() {
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
                let ret = body.as_bytes();
                let length_value = HeaderValue::try_from(ret.len()).unwrap();
                self.command_context.server_response.add_header(CONTENT_LENGTH, length_value);
                let bbb = Body::from(ret);
                let fire_resp = Response::builder().body(bbb).unwrap();
                self.command_context.server_response.fire_result(fire_resp);
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

#[async_trait]
impl<'a> BuzzContextTrait<'a> for BaseBuzzContext<'a> {
    async fn response(mut self, resp: &mut ContextResponseWrapper<'a>) -> io::Result<()> {
        self.async_response(resp).await
    }

    async fn on_response(mut self, resp: &mut ContextResponseWrapper<'a>) -> io::Result<()> {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use std::arch;
    use std::rc::Rc;
    use std::sync::Arc;
    use http::Response;
    use hyper::Body;
    use tokio::sync::oneshot;
    use tokio::sync::oneshot::{channel, Sender};
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::{CellModule, Module};
    use crate::command::CommandContext;
    use crate::context::{BaseBuzzContext, BuzzContextTrait};
    use crate::core::ProtocolID;
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::summary::{Summary, SummaryTrait};
    use crate::wrapper::ContextResponseWrapper;
    use crate::output::{JSONOutputArchive, OutputArchive, Serializable};
    use serde::{Deserialize, Serialize};

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
        let (txx, mut rxx) = std::sync::mpsc::channel::<Response<Body>>();
        static M: &CellModule = &module::CellModule::new(1, "CONTEXT", &LogLevel::Info);
        let req: &mut dyn ServerRequestTrait = &mut MockRequest {};
        let resp: &mut dyn ServerResponseTrait = &mut MockResponse::new(txx);
        let ip = String::from("128");
        let sequence_id = String::from("seq");
        let protocol_id: ProtocolID = "p" as ProtocolID;
        let summ: &mut dyn SummaryTrait = &mut Summary::new(Arc::new(ip), Arc::new(sequence_id), protocol_id);
        let c_ctx: CommandContext = CommandContext::new(M, req, resp, summ);
        let mut ctx = BaseBuzzContext::new(32, c_ctx);
        let body = Body::from(String::from("asd"));
        let mut wrapper = ContextResponseWrapper::default();
        wrapper = wrapper.with_body(Box::new(JSONOutputArchive::new(Box::new(AARet { name: "charlie".to_string() }))));
        let r = tokio::runtime::Runtime::new().unwrap();
        futures::executor::block_on(ctx.response(&mut wrapper));

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
