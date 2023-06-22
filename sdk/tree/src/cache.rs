use crate::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use crate::error::TreeResult;
use crate::iter::{exclusive_range_from, iter_merge_next};
use crate::merkle::MerkleRocksDBConfiguration;
use crate::operation::{Operation, RollBackOperation};
use crate::tree::{Batch, Read, RootHash, TreeDB, Write, DB, KV};
use std::collections::BTreeMap;

type Map = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

pub struct CacheWrapper<M> {
    map: Option<Map>,
    inner: M,
}

impl<M> CacheWrapper<M> {
    pub fn new(mid: M) -> CacheWrapper<M> {
        Self {
            map: Some(Default::default()),
            inner: mid,
        }
    }

    #[inline]
    pub fn wrap(store: M) -> Self {
        CacheWrapper::new(store)
    }

    #[inline]
    pub fn wrap_with_map(store: M, map: Map) -> Self {
        Self {
            map: Some(map),
            inner: store,
        }
    }
}

impl<M: TreeDB> Batch for CacheWrapper<M> {
    fn commit(&mut self, mut operations: Vec<Operation>) -> RootHash {
        let map = self.map.take().unwrap();
        self.map = Some(Map::new());

        for (k, v) in map {
            match v.clone() {
                Some(value) => operations.push(Operation::Set(k, value)),
                None => operations.push(Operation::Delete(k)),
            }
        }
        self.inner.commit(operations)
    }
}

impl<M: TreeDB> DB for CacheWrapper<M> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.inner.get_configuration()
    }

    fn roll_back(&mut self, h: u64) -> TreeResult<()> {
        self.map = Some(Default::default());
        self.inner
            .roll_back_with_operation(RollBackOperation::MerkleHeight(h, Some(true)))
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.inner.roll_back_with_operation(ops)
    }
    fn height(&self) -> TreeResult<u64> {
        self.inner.height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        self.map = Some(Map::new());
        self.inner.clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        let mut map = self.map.replace(Default::default());
        let map = map.as_mut().unwrap();
        while let Some((key, v)) = map.pop_first() {
            match v {
                Some(value) => {
                    self.inner.set(key, value)?;
                }
                None => {
                    self.inner.delete(key)?;
                }
            }
        }
        self.inner.flush()
    }
}

impl<M: TreeDB> Write for CacheWrapper<M> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.map.as_mut().unwrap().insert(k.clone(), Some(v));
        Ok(k.clone())
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.map.as_mut().unwrap().insert(k, None);
        Ok(())
    }
}

impl<M: TreeDB> Read for CacheWrapper<M> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        match self.map.as_ref().unwrap().get(k) {
            Some(Some(value)) => Ok(Some(value.clone())),
            Some(None) => Ok(None),
            None => self.inner.get(k).map(|v| {
                if let Some(value) = v {
                    Some(value)
                } else {
                    None
                }
            }),
        }
    }

    fn get_next<'a>(&'a self, start: &[u8]) -> TreeResult<Option<KV>> {
        if self.map.as_ref().unwrap().is_empty() {
            return self.inner.get_next(start);
        }
        let mut map_iter = self
            .map
            .as_ref()
            .unwrap()
            .range(exclusive_range_from(start));
        let mut store_iter = (&self.inner).into_iter(exclusive_range_from(start))?;
        iter_merge_next::<M>(&mut map_iter, &mut store_iter)
    }
}

impl<M: TreeDB> TreeDB for CacheWrapper<M> {
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
