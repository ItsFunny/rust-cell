use crate::error::ZKResult;
use crate::vm::tracer::VMTracer;
use crate::vm::types::op::{Operation, SortOperation, Trace};

pub struct SimpleTracer {
    index: u64,

    operations: Vec<SortOperation>,
}

impl VMTracer for SimpleTracer {
    fn trace(&mut self, data: &Operation) {
        let op = SortOperation::new(self.index, data.clone());
        self.operations.push(op);
        self.index = self.index + 1;
    }

    fn finalize(&mut self) -> ZKResult<Trace> {
        let trace = Trace {
            operations: self.operations.clone(),
        };
        Ok(trace)
    }
}

impl SimpleTracer {}
