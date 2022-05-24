use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use http::Response;
use hyper::Body;
use tokio::sync::oneshot::Sender;
use logsdk::log4rs::DEFAULT_LOGGER;
use crate::context::BuzzContextTrait;
use crate::core::{AliasRequestType, AliasResponseType, ProtocolID, RunType};
use crate::ExecutorValueTrait;
use crate::request::{ServerRequestTrait, ServerResponseTrait};
use crate::summary::SummaryTrait;


pub type Function = dyn Fn(&dyn BuzzContextTrait, Option<&dyn ExecutorValueTrait>);


pub trait CommandTrait: 'static {
    fn id(&self) -> ProtocolID;
    fn execute(&self, ctx: &dyn BuzzContextTrait);
}

pub struct Command
    where
        Self: 'static,
{
    pub protocol_id: ProtocolID,
    pub fun: Option<&'static Function>,
    pub meta_data: MetaData,
    pub run_type: RunType,
    seal: bool,
}

pub struct MetaData {
    pub asy: bool,
    pub request_type: AliasRequestType,
    pub response_type: AliasResponseType,

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

impl Command {
    pub fn with_protocol_id(mut self, p: ProtocolID) -> Self {
        self.protocol_id = p;
        self
    }
    pub fn with_executor(mut self, e: &'static Function) -> Self {
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

pub struct CommandContext<'a>
{
    pub module: &'static dyn logsdk::module::Module,
    pub server_request: &'a mut dyn ServerRequestTrait,
    // TODO REFCELL
    pub server_response: &'a mut dyn ServerResponseTrait,
    // TODO, ARC
    pub summary: &'a mut dyn SummaryTrait,
    // TODO
    // pub channel: &dyn ChannelTrait,
    // pub command: &dyn CommandTrait<'b>,
    // TODO: ops
}

impl Default for Command {
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

impl<'a> CommandContext<'a> where
{
    pub fn new(module: &'static dyn logsdk::module::Module,
               server_request: &'a mut dyn ServerRequestTrait,
               server_response: &'a mut dyn ServerResponseTrait,
               st: &'a mut dyn SummaryTrait,
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

impl CommandTrait for Command {
    fn id(&self) -> ProtocolID {
        self.protocol_id
    }

    fn execute(&self, ctx: &dyn BuzzContextTrait) {
        // TODO input archive
        // TODO NOE
        (self.fun).unwrap()(ctx, None)
    }
}

impl Command {}


#[cfg(test)]
mod tests {
    use http::header::HeaderName;
    use crate::command::{Command, CommandTrait};
    use crate::core::ProtocolID;
    use crate::wrapper::ContextResponseWrapper;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_command() {
        let p: ProtocolID = "/protocol/v1" as ProtocolID;
        let mut c = Command::default();
        c = c.with_protocol_id(p).with_executor(&move |ctx, v| {
            println!("execute");
            let ret = ContextResponseWrapper::default();
        });
    }
}
