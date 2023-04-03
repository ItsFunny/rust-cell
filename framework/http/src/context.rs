use cell_core::cerror::CellResult;
use cell_core::context::{Context, RequestTrait};
use cell_core::wrapper::ContextResponseWrapper;
use cell_core::{
    command::CommandContext,
    context::{BaseBuzzContext, BuzzContextTrait},
};
use std::sync::Arc;

use async_trait::async_trait;
use cell_core::request::ServerRequestTrait;

pub struct HttpContext<'a> {
    ctx: BaseBuzzContext<'a>,
}

impl<'a> HttpContext<'a> {
    pub fn new(timest: i64, command_context: CommandContext<'a>) -> Self {
        HttpContext {
            ctx: BaseBuzzContext::new(timest, command_context),
        }
    }
}

impl<'a> Context for HttpContext<'a> {
    fn done(&mut self) -> bool {
        todo!()
    }
}

impl<'a> RequestTrait<'a> for HttpContext<'a> {
    fn get_request(&mut self) -> Arc<Box<dyn ServerRequestTrait + 'a>> {
        self.ctx.get_request()
    }
}

#[async_trait]
impl<'a> BuzzContextTrait<'a> for HttpContext<'a> {
    fn response(&mut self, resp: ContextResponseWrapper<'a>) -> CellResult<()> {
        self.ctx.response(resp)
    }

    fn on_response(
        &mut self,
        resp: cell_core::wrapper::ContextResponseWrapper<'a>,
    ) -> cell_core::cerror::CellResult<()> {
        todo!()
    }
}
