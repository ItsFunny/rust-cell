use cell_core::cerror::CellResult;
use cell_core::context::Context;
use cell_core::wrapper::ContextResponseWrapper;
use cell_core::{context::{BaseBuzzContext, BuzzContextTrait}, command::CommandContext};


use async_trait::async_trait;

pub struct HttpContext<'a> {
    ctx: BaseBuzzContext<'a>,
}

impl<'a> HttpContext<'a> {
    pub fn new(timest: i64, command_context: CommandContext<'a>) -> Self {
        HttpContext { ctx: BaseBuzzContext::new(timest, command_context) }
    }
}


impl<'a> Context for HttpContext<'a> {
    fn done(&mut self) -> bool {
        todo!()
    }
}

#[async_trait]
impl<'a> BuzzContextTrait<'a> for HttpContext<'a> {
    async fn response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()> {
        self.ctx.response(resp).await
    }

    async fn on_response(&mut self, resp: cell_core::wrapper::ContextResponseWrapper<'a>) -> cell_core::cerror::CellResult<()> {
        todo!()
    }
}