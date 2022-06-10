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


#[cfg(test)]
mod tests {
    use cell_core::channel::ChannelTrait;
    use cell_core::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use pipeline::executor::{DefaultChainExecutor, DefaultClosureReactorExecutor, ReactorExecutor};
    use crate::channel::HttpChannel;
    use crate::suit::HttpSuit;

    #[test]
    fn test_http_channel() {
        let ess2: &mut Vec<&dyn ReactorExecutor<DefaultChainExecutor<HttpSuit>, HttpSuit>> = &mut Vec::new();
        let f = |v: &HttpSuit| {
            println!("{:?}", v)
        };
        let e1: &dyn ReactorExecutor<DefaultChainExecutor<HttpSuit>, HttpSuit> = &DefaultClosureReactorExecutor::new(&f);
        ess2.push(e1);
        let mut chain_executor = DefaultChainExecutor::new(ess2);
        let c = HttpChannel::new(chain_executor);

        // let req: &mut dyn ServerRequestTrait = &mut MockRequest::new();
        // let resp: &mut dyn ServerResponseTrait = &mut MockResponse::new(txx);
        // let ip = String::from("128");
        // let sequence_id = String::from("seq");
        // let protocol_id: ProtocolID = "p" as ProtocolID;
        // let summ: &mut dyn SummaryTrait = &mut Summary::new(Arc::new(ip), Arc::new(sequence_id), protocol_id);
        // let c_ctx: CommandContext = CommandContext::new(M, req, resp, summ);
        // let mut ctx = BaseBuzzContext::new(32, c_ctx);
        //
        // let suit=HttpSuit::new()
        // c.read_command()
    }
}
