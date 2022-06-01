use cell_core::channel::*;
use cell_core::suit::CommandSuit;
use pipeline::executor::{DefaultChainExecutor, ExecutorValueTrait};
use crate::suit::HttpSuit;

pub struct HttpChannel<'e, 'a>
    where
        Self: 'e
{
    channel: DefaultChannel<'e, 'a, HttpSuit<'a>>,
}

impl<'e, 'a> HttpChannel<'e, 'a> where
    Self: 'e {
    pub fn new(executors: DefaultChainExecutor<'e, 'a, HttpSuit<'a>>) -> Self {
        HttpChannel { channel: DefaultChannel::new(executors) }
    }
}

impl<'e, 'a> ChannelTrait<'e, 'a, HttpSuit<'a>> for HttpChannel<'e, 'a>
    where
        Self: 'e
{
    fn read_command(&'e mut self, suit: &'a HttpSuit<'a>) {
        self.channel.read_command(suit)
    }
}