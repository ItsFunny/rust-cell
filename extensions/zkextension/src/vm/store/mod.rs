use crate::error::ZKResult;
use crate::vm::types::op::{SortOperation, Trace};

mod merkle;

pub trait Storage {
    fn consume(&mut self, traces: &Trace) -> ZKResult<()>;

    fn hash(&self, op: &SortOperation) -> u64;
}
