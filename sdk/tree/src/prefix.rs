use crate::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use crate::error::TreeResult;
use crate::iter::prefix_iterator;
use crate::merkle::MerkleRocksDBConfiguration;
use crate::operation::{Operation, RollBackOperation};
use crate::tree::{Batch, Read, RootHash, TreeDB, Write, DB, KV};

pub struct PrefixWrapper<M> {
    prefix: Vec<u8>,
    inner: M,
}

impl<M> PrefixWrapper<M> {
    pub fn new(prefix: Vec<u8>, inner: M) -> Self {
        Self { prefix, inner }
    }
}

impl<M: TreeDB> Batch for PrefixWrapper<M> {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        self.inner.commit(operations)
    }
}

impl<M: TreeDB> DB for PrefixWrapper<M> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.inner.get_configuration()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.inner.roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        self.inner.height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.inner.clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        self.inner.flush()
    }
}

impl<M: TreeDB> Write for PrefixWrapper<M> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.inner
            .set(concat(self.prefix.as_slice(), k.as_slice()), v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.inner
            .delete(concat(self.prefix.as_slice(), k.as_slice()))
    }
}

impl<M: TreeDB> Read for PrefixWrapper<M> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.inner.get(concat(self.prefix.as_slice(), k).as_slice())
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        prefix_iterator(&self.inner, self.prefix.clone(), start.to_vec())
    }
}

impl<M: TreeDB> TreeDB for PrefixWrapper<M> {
    fn prove(&self, req: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        self.inner.prove(req)
    }

    fn verify(&self, req: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        self.inner.verify(req)
    }

    fn root_hash(&self) -> RootHash {
        self.inner.root_hash()
    }
}

#[inline]
pub fn concat(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut value = Vec::with_capacity(a.len() + b.len());
    value.extend_from_slice(a);
    value.extend_from_slice(b);
    value
}
