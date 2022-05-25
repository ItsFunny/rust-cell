use std::marker::PhantomData;
use crate::context::{BuzzContextTrait, Context};

pub trait CommandSuit<'a>: Context {
    fn get_buzz_context(&self) -> &'a dyn BuzzContextTrait;
}

pub struct DefaultCommandSuit<'a> {
    command_ctx: &'a dyn BuzzContextTrait<'a>,
    concrete: &'a mut dyn CommandSuit<'a>,
}

impl<'a> DefaultCommandSuit<'a> {
    pub fn new(command_ctx: &'a dyn BuzzContextTrait<'a>, e: &'a mut dyn CommandSuit<'a>) -> Self {
        DefaultCommandSuit { command_ctx, concrete: e }
    }

    pub fn set_concrete(&mut self, c: &'a mut dyn CommandSuit<'a>) {
        self.concrete = c
    }
}


impl<'a> Context for DefaultCommandSuit<'a> {
    fn discard(&mut self) {
        self.concrete.discard();
    }

    fn done(&mut self) -> bool {
        self.concrete.done()
    }
}

impl<'a> CommandSuit<'a> for DefaultCommandSuit<'a> {
    fn get_buzz_context(&self) -> &'a dyn BuzzContextTrait {
        self.command_ctx
    }
}

pub struct EmptyCommandSuite<'a> {
    _marker_a: PhantomData<&'a ()>,
}

impl<'a> Default for EmptyCommandSuite<'a> {
    fn default() -> Self {
        EmptyCommandSuite { _marker_a: Default::default() }
    }
}


impl<'a> Context for EmptyCommandSuite<'a> {
    fn discard(&mut self) {
        todo!()
    }

    fn done(&mut self) -> bool {
        todo!()
    }
}

impl<'a> CommandSuit<'a> for EmptyCommandSuite<'a> {
    fn get_buzz_context(&self) -> &'a dyn BuzzContextTrait {
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use crate::command::CommandContext;
    use crate::context::{BaseBuzzContext, BuzzContextTrait};
    use crate::core::ProtocolID;
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::suit::{DefaultCommandSuit, EmptyCommandSuite};
    use crate::summary::{Summary, SummaryTrait};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_template() {
        let (txx, mut rxx) = std::sync::mpsc::channel::<Response<Body>>();
        static M: &CellModule = &module::CellModule::new(1, "CONTEXT", &LogLevel::Info);
        let req: &mut dyn ServerRequestTrait = &mut MockRequest {};
        let resp: &mut dyn ServerResponseTrait = &mut MockResponse::new(txx);
        let ip = String::from("128");
        let sequence_id = String::from("seq");
        let protocol_id: ProtocolID = "p" as ProtocolID;
        let summ: &mut dyn SummaryTrait = &mut Summary::new(Arc::new(ip), Arc::new(sequence_id), protocol_id);
        let c_ctx: CommandContext = CommandContext::new(M, req, resp, summ);
        let mut ctx: &mut dyn BuzzContextTrait = &mut BaseBuzzContext::new(32, c_ctx);
        let mut mock = EmptyCommandSuite::default();
        let mut suit = DefaultCommandSuit::new(ctx, &mut mock);
    }
}
