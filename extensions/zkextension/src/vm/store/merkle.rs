use crate::error::ZKResult;
use crate::vm::store::Storage;
use crate::vm::types::op::{Operation, SortOperation, Trace};
use merkle_tree::smt::parallel_smt::ItemIndex;
use merkle_tree::smt::rescue_hasher::RescueHasher;
use merkle_tree::smt::storage::SMTStorage;
use merkle_tree::{DBCompatibleMerkleTree, Engine, Fr};

pub struct MerkleStorage<DB>
where
    DB: SMTStorage<ItemIndex, SortOperation>,
{
    tree: DBCompatibleMerkleTree<SortOperation, Fr, RescueHasher<Engine>, DB>,
}

impl<DB: SMTStorage<ItemIndex, SortOperation>> Storage for MerkleStorage<DB> {
    fn consume(&mut self, traces: &Trace) -> ZKResult<()> {
        for op in &traces.operations {
            let index = self.hash(op);
            self.tree.insert(index, op.clone());
        }
        Ok(())
    }

    fn hash(&self, op: &SortOperation) -> u64 {
        todo!()
    }
}

#[test]
pub fn test_asd() {}
