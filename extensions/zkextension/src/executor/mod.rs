pub mod mock;

use crate::blocks::block::Block;

pub type ExecutorResult = Result<(), anyhow::Error>;

pub trait Executor {
    fn execute_block(&mut self, block: Block) -> ExecutorResult;
    fn commit(&mut self) -> ExecutorResult;
}
