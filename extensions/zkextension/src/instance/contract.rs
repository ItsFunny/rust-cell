use crate::blocks::block::Block;
use crate::instance::{Contract, ExecutorResult};
use crate::store::Store;
use tree::tree::NullHasher;

pub struct RustSimpleContract {}

impl RustSimpleContract {
    pub fn new() -> Self {
        Self {}
    }
}

impl Contract<NullHasher> for RustSimpleContract {
    fn execute_block(
        &mut self,
        store: &mut Box<dyn Store<NullHasher>>,
        block: &Block,
    ) -> ExecutorResult {
        store.set(vec![1], vec![1, 2, 3, 4, 5])?;
        store.set(vec![99], vec![6, 7, 8, 9])?;
        Ok(())
    }

    fn byte_codes(&self) -> Vec<u8> {
        vec![1, 2, 3]
    }
}
