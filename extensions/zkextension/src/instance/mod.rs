pub mod contract;
pub mod manager;
pub mod merkle;

use crate::blocks::block::Block;
use crate::store::trace::TraceTable;
use crate::store::Store;
use crc32fast::Hasher;
use tree::tree::{KeyHasher, NullHasher};

pub type ExecutorResult = Result<(), anyhow::Error>;

pub trait Executor<K: KeyHasher> {
    fn begin_block(&mut self, block: &Block) -> ExecutorResult;
    fn execute_block(&mut self, block: &Block) -> ExecutorResult;
    fn get_traces(&self) -> TraceTable<K>;
}
pub trait Contract<H: KeyHasher> {
    fn execute_block(&mut self, store: &mut Box<dyn Store<H>>, block: &Block) -> ExecutorResult;
    fn byte_codes(&self) -> Vec<u8>;
}

pub struct Instance {
    store: Box<dyn Store<NullHasher>>,
    contract: Box<dyn Contract<NullHasher>>,
}
impl Instance {
    pub fn new(store: Box<dyn Store<NullHasher>>, contract: Box<dyn Contract<NullHasher>>) -> Self {
        Self { store, contract }
    }
}

pub fn calculate_contract_id<H: KeyHasher>(c: &Box<dyn Contract<H>>) -> u32 {
    let byte_codes = c.byte_codes();
    calculate_unique_u32(byte_codes.as_slice())
}
fn calculate_checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

fn calculate_unique_u32(data: &[u8]) -> u32 {
    let checksum = calculate_checksum(data);

    let mut hash_result = [0u8; 4];
    hash_result.copy_from_slice(&checksum.to_le_bytes()[..4]);

    u32::from_le_bytes(hash_result)
}

impl Executor<NullHasher> for Instance {
    fn begin_block(&mut self, block: &Block) -> ExecutorResult {
        self.store.flush().unwrap();
        Ok(())
    }

    fn execute_block(&mut self, block: &Block) -> ExecutorResult {
        self.contract.execute_block(&mut self.store, block)
    }

    fn get_traces(&self) -> TraceTable<NullHasher> {
        self.store.get_traces()
    }
}
