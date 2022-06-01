use std::fmt::{Debug, Formatter};
use cell_core::suit::*;
use cell_core::context::*;
use pipeline::executor::ExecutorValueTrait;
use crate::context::HttpContext;

pub struct HttpSuit<'a> {
    suit: DefaultCommandSuit<'a>,
}

impl<'a> HttpSuit<'a> {
    pub fn new(command_ctx: &'a HttpContext<'a>) -> Self {
        HttpSuit { suit: DefaultCommandSuit::new(command_ctx) }
    }
}

impl<'a> cell_core::context::Context for HttpSuit<'a> {
    fn discard(&mut self) {
        todo!()
    }

    fn done(&mut self) -> bool {
        todo!()
    }
}

impl<'a> ExecutorValueTrait<'a> for HttpSuit<'a> {}

impl<'a> Debug for HttpSuit<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<'a> CommandSuit<'a> for HttpSuit<'a> {
    fn get_buzz_context(&self) -> &'a dyn cell_core::context::BuzzContextTrait {
        self.suit.get_buzz_context()
    }
}