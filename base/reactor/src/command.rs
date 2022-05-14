use std::cell::RefCell;
use std::rc::Rc;
use context::context::{ServerRequestTrait, ServerResponseTrait, SummaryTrait};
use crate::channel::ChannelTrait;
use crate::ProtocolID;

pub trait CommandTrait {
    fn id(&self) -> ProtocolID;
}


pub struct CommandContext {
    pub server_request: Rc<Box<dyn ServerRequestTrait>>,
    pub server_response: Rc<RefCell<dyn ServerResponseTrait>>,
    pub summary: Rc<Box<dyn SummaryTrait>>,
    // TODO
    pub channel: &'static dyn ChannelTrait,
    pub command: &'static dyn CommandTrait,
    // TODO: ops
}