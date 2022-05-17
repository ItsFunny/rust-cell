use std::rc::Rc;
use chrono::Local;
use context::context::Context;
use crate::command::CommandContext;


pub trait BuzzContextTrait: Context {
    fn response(&self, resp: ContextResponseWrapper);
    fn on_response(&self, resp: ContextResponseWrapper);
}

pub struct BaseBuzzContext<'a: 'b, 'b> {
    pub request_timestamp: i64,
    pub command_context: CommandContext<'a, 'b>,
}

impl<'a: 'b, 'b> Context for BaseBuzzContext<'a, 'b> {
    fn discard(&mut self) {
        todo!()
    }

    fn done(&self) -> bool {
        todo!()
    }
}

impl BuzzContextTrait for BaseBuzzContext {
    fn response(&self, resp: ContextResponseWrapper) {
        let now = Local::now().timestamp();
        let consume_time = now - self.request_timestamp;
        let s = Rc::clone(&self.command_context.summary);
        let sequence_id = self.command_context.summary.get_sequence_id();

        self.on_response(resp)
    }

    fn on_response(&self, resp: ContextResponseWrapper) {
        todo!()
    }
}

pub struct ContextResponseWrapper {}
