use std::marker::PhantomData;
use std::rc::Rc;
use pipeline2::pipeline2::{ClosureExecutor, DefaultPipelineV2, DefaultReactorExecutor, PipelineBuilder};
use crate::context::{BuzzContextTrait, Context, ContextWrapper};
use crate::core::ExecutorValueTrait;
use crate::suit::{CommandSuit, DefaultCommandSuit};

pub trait ChannelTrait<'e, 'a>
{
    fn read_command(&mut self, suit:ContextWrapper<'a>);
}

pub struct ChannelChainExecutor {}

pub struct DefaultChannel<'e, 'a>
    where
        Self: 'e
{
    pip: DefaultPipelineV2<'e, ContextWrapper<'a>>,
    _marker_e: PhantomData<&'e ()>,
    _marker_a: PhantomData<&'a ()>,
    // _marker_t: PhantomData<T>,
}

impl<'e, 'a> DefaultChannel<'e, 'a> where
    // T: ExecutorValueTrait<'a> + CommandSuit<'a>,
    Self: 'e {
    pub fn new(p: DefaultPipelineV2<'e,ContextWrapper<'a>>) -> Self {
        Self {
            pip: p, _marker_e: Default::default(), _marker_a: Default::default(),
            // _marker_t: Default::default()
        }
    }
}

impl<'e, 'a> DefaultChannel<'e, 'a> where
    // T: ExecutorValueTrait<'a> + CommandSuit<'a>,
    Self: 'e {}

impl<'e, 'a> ChannelTrait<'e, 'a> for DefaultChannel<'e, 'a>
{
    fn read_command(&mut self, suit:ContextWrapper<'a>) {
        self.pip.execute(&suit);
    }
}

impl<'e, 'a> DefaultChannel<'e, 'a>
{
    pub fn echo(&self) {
        println!("{}", 1)
    }
}

pub fn mock_channel<'e, 'a>() -> DefaultChannel<'e, 'a> {
    let pip = PipelineBuilder::default().add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &ContextWrapper| {
        println!("111 {:?}", v)
    }))))).add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &ContextWrapper| {
        println!("222 {:?}", v)
    }))))).build();

    DefaultChannel::new(pip)
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
    use crate::channel::{ChannelTrait, DefaultChannel, mock_channel};
    use crate::command::{CommandContext, mock_context};
    use crate::context::{BaseBuzzContext, BuzzContextTrait, ContextWrapper};
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
        let (c, rxx, mut ctx) = mock_context();
        let mut channel = mock_channel();
        let wrapper=ContextWrapper::new(Box::new(ctx));
        channel.read_command(wrapper)
    }
}