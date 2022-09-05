use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use cell_core::channel::*;
use cell_core::context::{BuzzContextTrait, ContextWrapper};
use cell_core::dispatcher::DefaultDispatcher;
use pipeline2::pipeline2::{ClosureExecutor, DefaultPipelineV2, DefaultReactorExecutor, PipelineBuilder};
use async_trait::async_trait;
use cell_core::command::CommandTrait;

pub struct HttpChannel<'e:'a, 'a>
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

impl<'e:'a, 'a> Default for HttpChannel<'e, 'a> {
    fn default() -> Self {
        let pip = PipelineBuilder::default().add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Arc::new(|v: &mut ContextWrapper<'a>| {
            println!("http:111 {:?}", v)
        }))))).add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Arc::new(|v: &mut ContextWrapper<'a>| {
            println!("http:222 {:?}", v)
        })))))
            .add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Arc::new(|v: &mut ContextWrapper<'a>| {
                // let  cc:&mut dyn BuzzContextTrait = &mut v.ctx.deref();
                let mut ctx=v.ctx.as_mut();
                v.cmd.execute(ctx);
            })))))
            .build();
        HttpChannel::new(pip)
    }
}

#[async_trait]
impl<'e, 'a> ChannelTrait<'e, 'a> for HttpChannel<'e, 'a>
{
    async fn read_command(&self, suit: ContextWrapper<'a>) {
        self.channel.read_command(suit).await
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
