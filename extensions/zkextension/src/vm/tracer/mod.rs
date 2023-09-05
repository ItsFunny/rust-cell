use crate::error::ZKResult;
use crate::vm::types::op::{Operation, Trace, TypeWrapper};

pub mod simple;

pub trait VMTracer {
    fn trace(&mut self, data: &Operation);

    fn trace_get(&mut self, data: &TypeWrapper) {
        let operation = Operation::Get(data.clone());
        self.trace(&operation)
    }
    fn trace_set(&mut self, data: &TypeWrapper) {
        let operation = Operation::Set(data.clone());
        self.trace(&operation)
    }

    fn trace_delete(&mut self, data: &TypeWrapper) {
        let operation = Operation::Delete(data.clone());
        self.trace(&operation)
    }

    fn trace_alloc(&mut self, data: &TypeWrapper) {
        let operation = Operation::Alloc(data.clone());
        self.trace(&operation)
    }

    fn finalize(&mut self) -> ZKResult<Trace>;
}

#[test]
pub fn test_works() {}
