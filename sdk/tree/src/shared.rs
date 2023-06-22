use crate::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use crate::error::TreeResult;
use crate::merkle::MerkleRocksDBConfiguration;
use crate::operation::{Operation, RollBackOperation};
use crate::tree::{Batch, Read, RootHash, TreeDB, Write, DB, KV};
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;

pub struct SharedWrapper<M: TreeDB>(Rc<RefCell<M>>);

impl<M: TreeDB> SharedWrapper<M> {
    #[inline]
    pub fn new(inner: M) -> Self {
        SharedWrapper(Rc::new(RefCell::new(inner)))
    }

    pub fn into_inner(self) -> M {
        match Rc::try_unwrap(self.0) {
            Ok(inner) => inner.into_inner(),
            _ => panic!("Store is already borrowed"),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<M> {
        self.0.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<M> {
        self.0.borrow()
    }
}

impl<M: TreeDB> Clone for SharedWrapper<M> {
    #[inline]
    fn clone(&self) -> Self {
        SharedWrapper(self.0.clone())
    }
}

impl<M: TreeDB> DB for SharedWrapper<M> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.0.borrow().get_configuration()
    }
    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.0.borrow_mut().roll_back_with_operation(ops)
    }
    fn height(&self) -> TreeResult<u64> {
        self.0.borrow().height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.0.borrow_mut().clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        self.0.borrow_mut().flush()
    }
}

unsafe impl<M: TreeDB> Send for SharedWrapper<M> {}

unsafe impl<M: TreeDB> Sync for SharedWrapper<M> {}

impl<M: TreeDB> Write for SharedWrapper<M> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        let mut inner = self.0.borrow_mut();
        inner.set(k, v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.0.borrow_mut().delete(k)
    }
}

impl<M: TreeDB> Read for SharedWrapper<M> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.0.borrow().get(k)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        self.0.borrow().get_next(start)
    }
}

impl<M: TreeDB> Batch for SharedWrapper<M> {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        self.0.borrow_mut().commit(operations)
    }
}

impl<M: TreeDB> TreeDB for SharedWrapper<M> {
    fn prove(&self, req: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        self.0.borrow().prove(req)
    }

    fn verify(&self, req: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        self.0.borrow().verify(req)
    }

    fn root_hash(&self) -> RootHash {
        self.0.borrow().root_hash()
    }
}

/////
pub struct SharedDB<M: DB>(Rc<RefCell<M>>);

impl<M: DB> Clone for SharedDB<M> {
    #[inline]
    fn clone(&self) -> Self {
        SharedDB(self.0.clone())
    }
}

impl<M: DB> SharedDB<M> {
    #[inline]
    pub fn new(inner: M) -> Self {
        SharedDB(Rc::new(RefCell::new(inner)))
    }

    pub fn into_inner(self) -> M {
        match Rc::try_unwrap(self.0) {
            Ok(inner) => inner.into_inner(),
            _ => panic!("Store is already borrowed"),
        }
    }

    pub fn borrow_mut(&mut self) -> RefMut<M> {
        self.0.borrow_mut()
    }

    pub fn borrow(&self) -> Ref<M> {
        self.0.borrow()
    }
}

impl<M: DB> Write for SharedDB<M> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.0.borrow_mut().set(k, v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.0.borrow_mut().delete(k)
    }
}

impl<M: DB> Read for SharedDB<M> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.0.borrow().get(k)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        self.0.borrow().get_next(start)
    }
}

impl<M: DB> Batch for SharedDB<M> {
    fn commit(&mut self, ops: Vec<Operation>) -> RootHash {
        self.0.borrow_mut().commit(ops)
    }
}

impl<M: DB> DB for SharedDB<M> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.0.borrow().get_configuration()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.0.borrow_mut().roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        self.0.borrow_mut().height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.0.borrow_mut().clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        self.0.borrow_mut().flush()
    }
}
