use std::marker::PhantomData;
use pipeline::executor::{ChainExecutor, DefaultChainExecutor, ExecutorValueTrait};
use crate::context::Context;
use crate::suit::CommandSuit;

pub trait ChannelTrait<'e, 'a, T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    fn read_command(&'e mut self, suit: &'a T);
}

pub struct ChannelChainExecutor {}

pub struct DefaultChannel<'e, 'a, T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a>,
        Self: 'e
{
    executor: DefaultChainExecutor<'e, 'a, T>,
}

impl<'e, 'a, T> DefaultChannel<'e, 'a, T> where
    T: ExecutorValueTrait<'a> + CommandSuit<'a>,
    Self: 'e {
    pub fn new(ex: DefaultChainExecutor<'e, 'a, T>) -> Self {
        DefaultChannel { executor: ex }
    }
}

impl<'e, 'a, T> ChannelTrait<'e, 'a, T> for DefaultChannel<'e, 'a, T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    fn read_command(&'e mut self, suit: &'a T) {
        self.executor.execute(suit);
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Formatter};
    use std::marker::PhantomData;
    use std::sync::Arc;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use pipeline::executor::{DefaultChainExecutor, DefaultReactorExecutor, ExecutorValueTrait, ReactorExecutor};
    use crate::channel::{ChannelTrait, DefaultChannel};
    use crate::command::CommandContext;
    use crate::context::{BaseBuzzContext, BuzzContextTrait};
    use crate::core::ProtocolID;
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::suit::{CommandSuit, DefaultCommandSuit, EmptyCommandSuite};
    use crate::summary::{Summary, SummaryTrait};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    pub struct MockChannelExecutor<'e, 'a>
    {
        _marker_a: PhantomData<&'a ()>,
        _marker_e: PhantomData<&'e ()>,
    }

    impl<'e, 'a> Debug for MockChannelExecutor<'e, 'a> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            todo!()
        }
    }

    impl<'e, 'a, V> ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> for MockChannelExecutor<'e, 'a>
        where
            V: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
    {
        fn on_execute(&'e self, v: &'a V) {
            println!("{:?}", v);
        }
    }

    #[test]
    fn test_suit() {
        let ess2: &mut Vec<&dyn ReactorExecutor<DefaultChainExecutor<DefaultCommandSuit>, DefaultCommandSuit>> = &mut Vec::new();
        let e1: &dyn ReactorExecutor<DefaultChainExecutor<DefaultCommandSuit>, DefaultCommandSuit> = &MockChannelExecutor { _marker_a: Default::default(), _marker_e: Default::default() };
        let e2: &dyn ReactorExecutor<DefaultChainExecutor<DefaultCommandSuit>, DefaultCommandSuit> = &MockChannelExecutor { _marker_a: Default::default(), _marker_e: Default::default() };
        ess2.push(e1);
        ess2.push(e2);
        let mut chain_executor = DefaultChainExecutor::new(ess2);
        let mut channel = DefaultChannel::new(chain_executor);

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
        let mut box_mock = Box::new(mock);
        let mut suit = DefaultCommandSuit::new(ctx);
        // suit.set_concrete(box_mock);

        channel.read_command(&suit);
    }
}