use std::marker::PhantomData;
use pipeline2::pipeline2::DefaultPipelineV2;
use crate::context::Context;
use crate::core::ExecutorValueTrait;
use crate::suit::CommandSuit;

pub trait ChannelTrait<'e, 'a, T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    fn read_command(&mut self, suit: &T);
}

pub struct ChannelChainExecutor {}

pub struct DefaultChannel<'e, 'a, T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a>,
        Self: 'e
{
    pip: DefaultPipelineV2<'e,T>,
    _marker_e: PhantomData<&'e ()>,
    _marker_a: PhantomData<&'a ()>,
    _marker_t: PhantomData<T>,
}

impl<'e, 'a, T> DefaultChannel<'e, 'a, T> where
    T: ExecutorValueTrait<'a> + CommandSuit<'a>,
    Self: 'e {
    pub fn new(p: DefaultPipelineV2<'e,T>) -> Self {
        Self { pip: p, _marker_e: Default::default(), _marker_a: Default::default(), _marker_t: Default::default() }
    }
}

impl<'e, 'a, T> DefaultChannel<'e, 'a, T> where
    T: ExecutorValueTrait<'a> + CommandSuit<'a>,
    Self: 'e {}

impl<'e, 'a, T> ChannelTrait<'e, 'a, T> for DefaultChannel<'e, 'a, T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    fn read_command(&mut self, suit: &T) {
        self.pip.execute(suit);
    }
}
impl<'e, 'a, T> DefaultChannel<'e,'a,T>
    where
        T: ExecutorValueTrait<'a> + CommandSuit<'a> + 'a,
{
    pub fn echo(&self){
        println!("{}",1)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Formatter};
    use std::marker::PhantomData;
    use std::rc::Rc;
    use std::sync::Arc;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use pipeline2::pipeline2::{ClosureExecutor, DefaultChainExecutor, DefaultReactorExecutor, MockExecutor, PipelineBuilder};
    use crate::channel::{ChannelTrait, DefaultChannel};
    use crate::command::{CommandContext, mock_context};
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


    #[test]
    fn test_suit() {
        let (c,rxx,mut ctx)=mock_context();
        let pip = PipelineBuilder::default().add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &DefaultCommandSuit| {
            println!("111 {:?}", v)
        }))))).add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &DefaultCommandSuit| {
            println!("222 {:?}", v)
        }))))).build();
        // let pip=PipelineBuilder::default().add_last(DefaultReactorExecutor::new(Box::new(MockExecutor::default()))).build();

        let mut channel = DefaultChannel::new(pip);

        let mut mock = EmptyCommandSuite::default();
        let mut box_mock = Box::new(mock);
        let mut suit = DefaultCommandSuit::new(&ctx);

        channel.read_command(&suit);
    }
}