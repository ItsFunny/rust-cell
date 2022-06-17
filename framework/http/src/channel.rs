use std::rc::Rc;
use cell_core::channel::*;
use cell_core::context::ContextWrapper;
use cell_core::dispatcher::DefaultDispatcher;
use cell_core::suit::CommandSuit;
use pipeline2::pipeline2::{ClosureExecutor, DefaultPipelineV2, DefaultReactorExecutor, PipelineBuilder};

pub struct HttpChannel<'e, 'a>
    where
        Self: 'e
{
    channel: DefaultChannel<'e, 'a>,
}

impl<'e, 'a> HttpChannel<'e, 'a> where
    Self: 'e {
    pub fn new(executors: DefaultPipelineV2<'e, ContextWrapper<'a>>) -> Self {
        HttpChannel { channel: DefaultChannel::new(executors) }
    }
}

impl<'e, 'a> Default for HttpChannel<'e, 'a> {
    fn default() -> Self {
        let pip = PipelineBuilder::default().add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &ContextWrapper| {
            println!("http:111 {:?}", v)
        }))))).add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &ContextWrapper| {
            println!("http:222 {:?}", v)
        })))))
            .add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(|v: &ContextWrapper| {

            })))))
            .build();
        HttpChannel::new(pip)
    }
}

impl<'e, 'a> ChannelTrait<'e, 'a> for HttpChannel<'e, 'a>
{
    fn read_command(&mut self, suit: ContextWrapper<'a>) {
        self.channel.read_command(suit)
    }
}


#[cfg(test)]
mod tests {
    use cell_core::channel::ChannelTrait;
    use cell_core::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::channel::HttpChannel;

    #[test]
    fn test_http_channel() {}
}
