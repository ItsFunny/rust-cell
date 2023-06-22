use crate::couple::{
    ProveRequestEnums, ProveResponseEnums, SimpleProveResponse, VerifyRequestEnums, VerifyResponse,
};
use crate::error::TreeResult;
use crate::iter::exclusive_range_from;
use crate::merkle::MerkleRocksDBConfiguration;

use crate::operation::{Operation, RollBackOperation};
use crate::tree::{Batch, Read, RootHash, TreeDB, Write, DB, KV};

use crate::shared::SharedDB;
use std::collections::BTreeMap;

type Map = BTreeMap<Vec<u8>, Option<Vec<u8>>>;

pub struct MemDB {
    map: Option<Map>,
    height: u64,

    histories: Vec<Map>,
}

impl Default for MemDB {
    fn default() -> Self {
        Self {
            map: Some(Default::default()),
            height: 0,
            histories: Default::default(),
        }
    }
}

pub fn new_mem_shared_db() -> SharedDB<Box<dyn DB>> {
    SharedDB::new(Box::new(MemDB::default()))
}

impl Batch for MemDB {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        for op in operations {
            match op {
                Operation::Set(k, v) => {
                    self.set(k, v).unwrap();
                }
                Operation::Delete(k) => {
                    self.delete(k).unwrap();
                }
                Operation::Aux(_) => {
                    panic!("todo")
                }
            }
        }
        self.height = self.height + 1;
        self.histories.push(self.map.as_ref().unwrap().clone());
        self.root_hash()
    }
}

impl DB for MemDB {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        MerkleRocksDBConfiguration::new("./merk.db")
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        let mut count = 1;
        let h_opt = ops.get_height();
        if h_opt.is_none() {
            panic!()
        }
        let h = h_opt.unwrap();
        if h > self.histories.len() as u64 {
            panic!()
        }
        let mut current: Map = Default::default();
        self.histories.retain(|v: &Map| {
            let mut dele = false;
            if count == h {
                current = v.clone();
            }
            if count > h {
                dele = true;
            }
            count = count + 1;
            !dele
        });
        self.map.replace(current);
        Ok(())
    }
    fn height(&self) -> TreeResult<u64> {
        Ok(self.height)
    }

    fn clean(&mut self) -> TreeResult<()> {
        Ok(())
    }

    fn flush(&mut self) -> TreeResult<()> {
        Ok(())
    }
}

impl Write for MemDB {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.map.as_mut().unwrap().insert(k.clone(), Some(v));
        Ok(k.clone())
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.map.as_mut().unwrap().remove(k.as_slice());
        Ok(())
    }
}

impl Read for MemDB {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        match self.map.as_ref().unwrap().get(k) {
            Some(Some(value)) => Ok(Some(value.clone())),
            Some(None) => Ok(None),
            None => Ok(None),
        }
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        let _map_iter = self
            .map
            .as_ref()
            .unwrap()
            .range(exclusive_range_from(start));
        Ok(None)
    }
}

impl TreeDB for MemDB {
    fn prove(&self, _req: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        Ok(ProveResponseEnums::SimpleProve(SimpleProveResponse {
            proof: b"mock_proof".to_vec(),
        }))
    }

    fn verify(&self, _req: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        Ok(VerifyResponse { valid: true })
    }

    fn root_hash(&self) -> RootHash {
        *slice_to_array_32(b"mockmockmockmockmockmockmockmock")
    }
}

pub(crate) fn slice_to_array_32<U8>(slice: &[U8]) -> &[U8; 32] {
    if slice.len() == 32 {
        let ptr = slice.as_ptr() as *const [U8; 32];
        unsafe { &*ptr }
    } else {
        panic!()
    }
}

#[cfg(test)]
mod test {
    use crate::mem::MemDB;
    use crate::tree::{Read, Write};

    #[test]
    pub fn test_set() {
        let mut m = MemDB::default();
        m.set(vec![1, 2], vec![4, 5]).expect("fail to set");
        let res = m.get(vec![1, 2].as_slice()).expect("fail to get").unwrap();
        assert_eq!(res, vec![4, 5])
    }
}
