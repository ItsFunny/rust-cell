use std::cell::RefCell;
use std::rc::Rc;
use context::context::{ServerRequestTrait, ServerResponseTrait, SummaryTrait};
use context::ExecutorValueTrait;
use crate::channel::ChannelTrait;
use crate::context::BuzzContextTrait;
use crate::{AliasRequestType, AliasResponseType, ProtocolID, RunType};


pub type Function<'a> = dyn Fn(&'a dyn BuzzContextTrait, Option<&'a dyn ExecutorValueTrait>) + 'a;


pub trait CommandTrait<'a>: 'static {
    fn id(&self) -> ProtocolID;
    fn execute(&self, ctx: &'a dyn BuzzContextTrait);
}

pub struct Command<'a>
    where
        Self: 'static,
{
    pub protocol_id: ProtocolID,
    pub fun: &'static Function<'a>,
    pub meta_data: MetaData,
    pub run_type: RunType,
}

pub struct MetaData {
    pub asy: bool,
    pub request_type: AliasRequestType,
    pub response_type: AliasResponseType,

}

pub struct CommandContext<'a: 'b, 'b>
    where
        Self: 'b,
{
    pub server_request: Rc<Box<dyn ServerRequestTrait>>,
    pub server_response: Rc<RefCell<dyn ServerResponseTrait>>,
    // TODO, ARC
    pub summary: Rc<Box<dyn SummaryTrait>>,
    // TODO
    pub channel: &'a dyn ChannelTrait,
    pub command: &'a dyn CommandTrait<'b>,
    // TODO: ops
}

////////////

impl<'a> CommandTrait<'a> for Command<'a> {
    fn id(&self) -> ProtocolID {
        self.protocol_id
    }

    fn execute(&self, ctx: &'a dyn BuzzContextTrait) {
        // TODO input archive
        (self.fun)(ctx, None)
    }
}

impl<'a> Command<'a> {}