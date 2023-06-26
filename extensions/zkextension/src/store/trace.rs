use crate::store::Store;
use merkle_tree::primitives::GetBits;
use std::cell::RefCell;
use tree::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use tree::error::TreeResult;
use tree::merkle::MerkleRocksDBConfiguration;
use tree::operation::{Operation, RollBackOperation};
use tree::tree::{Batch, KeyHasher, NullHasher, Read, RootHash, TreeDB, Write, DB};

pub type KeyBits = Vec<u8>;
pub type ValueBits = Vec<u8>;

pub struct TraceStore<H: KeyHasher> {
    trace: RefCell<TraceTable<H>>,
    internal: Box<dyn DB>,
}

#[derive(Default, Clone)]
pub struct TraceTable<H: KeyHasher> {
    pub alloc: u32,
    pub write: WriteTable<H>,
    pub read: ReadTable,
}
impl<H: KeyHasher> TraceTable<H> {
    pub fn is_empty(&self) -> bool {
        self.alloc == 0
    }
}
#[derive(Default, Clone)]
pub struct WriteTable<H: KeyHasher> {
    pub traces: Vec<WriteTrace<H>>,
}
#[derive(Default, Clone)]
pub struct ReadTable {
    traces: Vec<ReadTrace>,
}
impl<H: KeyHasher> TraceTable<H> {
    fn trace_set(&mut self, k: &Vec<u8>, v: &Vec<u8>) {
        self.write.traces.push(WriteTrace::Set(
            self.alloc,
            H::hash(k.as_slice()),
            H::hash(v.as_slice()),
        ));
        self.alloc_incr();
    }
    fn trace_delete(&mut self, k: &Vec<u8>) {
        self.write
            .traces
            .push(WriteTrace::Delete(self.alloc, H::hash(k.as_slice())));
        self.alloc_incr();
    }
    fn trace_read(&mut self, k: &[u8]) {
        self.read
            .traces
            .push(ReadTrace::Get(self.alloc, k.to_vec()));
        self.alloc_incr();
    }

    fn alloc_incr(&mut self) {
        self.alloc = self.alloc + 1;
    }
}
#[derive(Clone)]
pub enum TraceEnum<H: KeyHasher> {
    Write(WriteTrace<H>),
    Read(ReadTrace),
}

#[derive(Clone)]
pub enum WriteTrace<H: KeyHasher> {
    Set(u32, H::Out, H::Out),
    Delete(u32, H::Out),
}

#[derive(Clone)]
pub enum ReadTrace {
    Get(u32, KeyBits),
}

pub trait IndexAble {
    fn get_index(&self) -> u32;
}

impl<H: KeyHasher> TraceStore<H> {
    pub fn new(internal: Box<dyn DB>) -> Self {
        Self {
            trace: RefCell::new(TraceTable::default()),
            internal,
        }
    }
}

impl<H: KeyHasher> Store<H> for TraceStore<H> {
    fn get_traces(&self) -> TraceTable<H> {
        self.trace.borrow_mut().clone()
    }
}

impl<H: KeyHasher> DB for TraceStore<H> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.internal.get_configuration()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.internal.roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        self.internal.height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.internal.clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        self.internal.flush()
    }
}

impl<H: KeyHasher> Write for TraceStore<H> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.trace.borrow_mut().trace_set(&k, &v);
        self.internal.set(k, v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.trace.borrow_mut().trace_delete(&k);
        self.internal.delete(k)
    }
}

impl<H: KeyHasher> Read for TraceStore<H> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.trace.borrow_mut().trace_read(k);
        self.internal.get(k)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<tree::tree::KV>> {
        self.internal.get_next(start)
    }
}

impl<H: KeyHasher> Batch for TraceStore<H> {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        self.internal.commit(operations)
    }
}
