use crate::blocks::block::Block;
use crate::executor::{Executor, ExecutorResult};
use crate::store::Store;

pub struct MockExecutor {
    store: Box<dyn Store>,
}

impl Executor for MockExecutor {
    fn execute_block(&mut self, block: Block) -> ExecutorResult {
        Ok(())
    }

    fn commit(&mut self) -> ExecutorResult {
        Ok(())
    }
}
