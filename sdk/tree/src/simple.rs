use crate::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use crate::error::TreeResult;
use crate::merkle::MerkleRocksDBConfiguration;
use crate::operation::{Operation, RollBackOperation};
use crate::shared::SharedDB;
use crate::tree::{Batch, Read, RootHash, TreeDB, Write, DB, KV};

pub struct SimpleTreeWrapper(SharedDB<Box<dyn DB>>);

unsafe impl Send for SimpleTreeWrapper {}

unsafe impl Sync for SimpleTreeWrapper {}
impl SimpleTreeWrapper {
    pub fn new(f: SharedDB<Box<dyn DB>>) -> Self {
        Self(f)
    }
    pub fn get_db(&self) -> SharedDB<Box<dyn DB>> {
        return self.0.clone();
    }
}

impl Clone for SimpleTreeWrapper {
    fn clone(&self) -> Self {
        SimpleTreeWrapper::new(self.0.clone())
    }
}

impl DB for SimpleTreeWrapper {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.0.get_configuration()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.0.roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        self.0.height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.0.clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        self.0.flush()
    }
}

impl Write for SimpleTreeWrapper {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.0.set(k, v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.0.delete(k)
    }
}

impl Read for SimpleTreeWrapper {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.0.get(k)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        self.0.get_next(start)
    }
}

impl Batch for SimpleTreeWrapper {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        self.0.commit(operations)
    }
}

impl TreeDB for SimpleTreeWrapper {
    fn prove(&self, _req: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        unreachable!()
    }

    fn verify(&self, _req: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        unreachable!()
    }

    fn root_hash(&self) -> RootHash {
        unreachable!()
    }
}
