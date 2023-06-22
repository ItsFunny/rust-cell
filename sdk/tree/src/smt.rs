use crate::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use crate::error::{TreeError, TreeResult};
use crate::iter::concat;
use crate::mem::slice_to_array_32;
use crate::merkle::MerkleRocksDBConfiguration;
use crate::operation::{Operation, RollBackOperation};
use crate::rocksdb::RocksDB;
use crate::shared::SharedDB;
use crate::tree::{batch_operation, Batch, Read, RootHash, TreeDB, Write, DB, KV};

use smt::{HashValue, NodeStore, SMTree, SPARSE_MERKLE_PLACEHOLDER_HASH};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::RangeBounds;
use std::rc::Rc;

pub struct SMTDb {
    // put is not mutable ,so it must use the couple
    merk: Rc<RefCell<SharedDB<Box<dyn DB>>>>,
}

impl SMTDb {
    pub fn new(merk: SharedDB<Box<dyn DB>>) -> Self {
        let internal = Rc::new(RefCell::new(merk));
        Self { merk: internal }
    }
}

impl NodeStore for SMTDb {
    fn get(&self, hash: &HashValue) -> anyhow::Result<Option<Vec<u8>>> {
        let key = hash.to_vec();
        let ret = self.merk.borrow().get(key.as_slice())?;
        return Ok(ret);
    }

    fn put(&self, key: HashValue, node: Vec<u8>) -> anyhow::Result<()> {
        let key = key.to_vec();
        self.merk.borrow_mut().set(key, node)?;
        return Ok(());
    }

    fn write_nodes(&self, nodes: BTreeMap<HashValue, Vec<u8>>) -> anyhow::Result<()> {
        let iterator = nodes.into_iter();
        for iter in iterator {
            self.put(iter.0, iter.1)?;
        }
        return Ok(());
    }
}

pub struct SMT {
    smt: Option<SMTree<Vec<u8>, Vec<u8>, SMTDb>>,
    internal_db: Option<SharedDB<Box<dyn DB>>>,
    configuration: MerkleRocksDBConfiguration,
    root_prefix: Vec<u8>,
}

pub fn new_smt(b: MerkleRocksDBConfiguration, root_prefix: Vec<u8>) -> SMT {
    let merk: Box<dyn DB> = Box::new(RocksDB::new(b.clone()));
    let rc_merk = SharedDB::new(merk);
    let root = get_smt_root(rc_merk.clone(), root_prefix.clone());
    let smt_db = SMTDb::new(rc_merk.clone());
    let smtree = SMTree::new(smt_db, Some(root));
    SMT {
        smt: Some(smtree),
        internal_db: Some(rc_merk.clone()),
        configuration: b.clone(),
        root_prefix,
    }
}
// pub fn new_smt(b: MerkleRocksDBConfiguration) -> SMT {
//     let merk: Box<dyn TreeDB> = Box::new(MerkleRocksDB::new(b.clone()));
//     let rc_merk = SharedWrapper::new(merk);
//     let root = get_smt_root(rc_merk.clone());
//     let smt_db = SMTDb::new(rc_merk.clone());
//     let smtree = SMTree::new(smt_db, Some(root));
//     SMT { smt: Some(smtree), internal_db: Some(rc_merk.clone()), configuration: b.clone() }
// }

pub fn new_smt_with_wrap_db(rc_merk: SharedDB<Box<dyn DB>>, root_prefix: Vec<u8>) -> SMT {
    let root = get_smt_root(rc_merk.clone(), root_prefix.clone());
    let smt_db = SMTDb::new(rc_merk.clone());
    let smtree = SMTree::new(smt_db, Some(root));
    SMT {
        smt: Some(smtree),
        internal_db: Some(rc_merk.clone()),
        configuration: rc_merk.get_configuration(),
        root_prefix: root_prefix.clone(),
    }
}

fn get_smt_root(merk: SharedDB<Box<dyn DB>>, root_prefix: Vec<u8>) -> HashValue {
    let key = build_root_key(root_prefix.as_slice());
    let root = {
        merk.get(key.as_slice())
            .expect("fail to get root")
            .map_or_else(
                || *SPARSE_MERKLE_PLACEHOLDER_HASH,
                |v| HashValue::new(*slice_to_array_32(v.as_slice())),
            )
    };
    root
}

impl DB for SMT {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.internal_db.as_ref().unwrap().get_configuration()
    }

    fn roll_back(&mut self, h: u64) -> TreeResult<()> {
        self.internal_db
            .as_mut()
            .unwrap()
            .roll_back_with_operation(RollBackOperation::MerkleHeight(h, Some(false)))?;
        self.replace();
        Ok(())
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.internal_db
            .as_mut()
            .unwrap()
            .roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        self.internal_db.as_ref().unwrap().height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        // smt wont do anything
        Ok(())
    }

    fn flush(&mut self) -> TreeResult<()> {
        let root = self.root_hash();
        let key = build_root_key(self.get_slice_root_prefix());
        self.internal_db
            .as_mut()
            .unwrap()
            .set(key, root.as_slice().to_vec())?;
        Ok(())
    }
}

unsafe impl Send for SMT {}

unsafe impl Sync for SMT {}

impl Write for SMT {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.smt
            .as_mut()
            .unwrap()
            .put(k.clone(), v)
            .map_err(|e| TreeError::from(e))?;
        return Ok(k.clone());
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.smt
            .as_mut()
            .unwrap()
            .remove(k)
            .map_err(|e| TreeError::from(e))?;
        Ok(())
    }
}

impl Read for SMT {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        Ok(self
            .smt
            .as_ref()
            .unwrap()
            .get(k.to_vec())
            .map_err(|e| TreeError::from(e))?)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        let iter = self
            .smt
            .as_ref()
            .unwrap()
            .iter(None)
            .map_err(|e| TreeError::from(e))?;

        let mut kvs = Vec::new();
        for v in iter {
            let kv = v?;
            if start.len() > 0 {
                match kv.0.as_slice().partial_cmp(start) {
                    None => {
                        continue;
                    }
                    Some(v) => match v {
                        Ordering::Less => {
                            continue;
                        }
                        _ => {
                            kvs.push(kv);
                        }
                    },
                }
            } else {
                kvs.push(kv);
            }
        }
        kvs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let mut iter = kvs.iter();
        return match iter.next() {
            None => Ok(None),
            Some(kv) => {
                if kv.0.clone().as_slice() == start {
                    match iter.next() {
                        None => Ok(None),
                        Some(kv2) => Ok(Some(kv2.clone())),
                    }
                } else {
                    Ok(Some(kv.clone()))
                }
            }
        };
    }

    fn into_iter<'a, B: RangeBounds<Vec<u8>>>(
        &'a self,
        bounds: B,
    ) -> TreeResult<Box<dyn Iterator<Item = TreeResult<KV>> + 'a>>
    where
        Self: Sized,
    {
        let iter = self
            .smt
            .as_ref()
            .unwrap()
            .iter(None)
            .map_err(|e| TreeError::from(e))?;

        let mut kvs = Vec::new();
        for v in iter {
            let kv = v?;
            if bounds.contains(&kv.0) {
                kvs.push(kv);
            } else {
                continue;
            }
        }
        kvs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        Ok(Box::new(SMTIteratorAdapter::new(kvs)))
    }
}

impl Batch for SMT {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        let aux_ops = batch_operation(self, operations).expect("fail to batch");
        let root = self.root_hash();
        self.commit_internal(aux_ops, root);
        root
    }
}

impl TreeDB for SMT {
    fn prove(&self, _req: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        todo!()
    }

    fn verify(&self, _req: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        todo!()
    }

    fn root_hash(&self) -> RootHash {
        *slice_to_array_32(self.smt.as_ref().unwrap().root_hash().to_vec().as_slice())
    }
}

impl SMT {
    fn commit_internal(&mut self, mut aux: Vec<Operation>, root: RootHash) {
        let key = build_root_key(self.get_slice_root_prefix());
        aux.push(Operation::Set(key, root.as_slice().to_vec()));
        self.internal_db.as_mut().unwrap().commit(aux);
    }

    fn replace(&mut self) {
        let mut smt = new_smt(self.configuration.clone(), self.root_prefix.clone());
        self.internal_db.replace(smt.internal_db.take().unwrap());
        self.smt.replace(smt.smt.take().unwrap());
    }

    fn get_slice_root_prefix(&self) -> &[u8] {
        self.root_prefix.as_slice()
    }
}

fn build_root_key(root_prefix: &[u8]) -> Vec<u8> {
    concat(root_prefix, b"smt_root")
}

pub struct SMTIteratorAdapter {
    kvs: Vec<KV>,
    index: usize,
    limit: usize,
}

impl SMTIteratorAdapter {
    pub fn new(kvs: Vec<KV>) -> Self {
        let l = kvs.len();
        Self {
            kvs,
            index: 0,
            limit: l,
        }
    }
}

impl Iterator for SMTIteratorAdapter {
    type Item = TreeResult<KV>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.limit {
            return None;
        }
        self.index = self.index + 1;
        let ret: KV = self.kvs.remove(0);
        return Some(Ok(ret));
    }
}

#[cfg(test)]
mod test {
    use crate::merkle::MerkleRocksDBConfiguration;
    use crate::smt::new_smt;
    use crate::tree::{Batch, Read, Write, DB};

    pub fn merk_configuration() -> MerkleRocksDBConfiguration {
        MerkleRocksDBConfiguration::new("./merk.db")
    }

    #[test]
    pub fn test_set() {
        let cfg = merk_configuration();
        let mut smt = new_smt(cfg.clone(), vec![]);
        let key = b"dog".to_vec();
        let value = b"cat".to_vec();
        smt.set(key.clone(), value.clone()).expect("fail to set");
        let res = smt
            .get(key.clone().as_slice())
            .expect("fail to get ")
            .unwrap();
        assert_eq!(res, value.as_slice());

        std::fs::remove_dir_all(&cfg.clone().home).unwrap();
    }

    // #[test]
    // pub fn get_one() {
    //     let cfg = merk_configuration();
    //     let mut db = new_smt(cfg);
    //     let key = b"dog";
    //     let value = b"cat";
    //     let value = db.get(key).expect("fail to get").unwrap();
    //     println!("{:?}", String::from_utf8(value));
    // }

    #[test]
    pub fn test_roll_back() {
        let cfg = merk_configuration();
        let mut db = new_smt(cfg, vec![]);

        let key = b"dog";
        let value = b"cat";
        {
            db.set(key.to_vec(), value.to_vec()).expect("fail to set");
            db.commit(vec![]);
            let h = db.height().expect("fail to get height");
            assert_eq!(h, 1);
            let data = db.get(key).expect("fail to get").unwrap();
            println!("{:?}", String::from_utf8(data.clone()));
            assert_eq!(data.clone().as_slice(), value);
        }

        let value2 = b"aaa";
        {
            db.set(key.to_vec(), value2.to_vec()).expect("fail to set");
            db.commit(vec![]);
            {
                let h = db.height().expect("fail to get height");
                assert_eq!(h, 2);
                let data = db.get(key).expect("fail to get").unwrap();
                assert_eq!(data.as_slice(), value2);
            }
        }

        {
            let value = b"ccc";
            db.set(key.to_vec(), value.to_vec()).expect("fail to set");
            db.commit(vec![]);
            {
                let h = db.height().expect("fail to get height");
                assert_eq!(h, 3);
                let data = db.get(key).expect("fail to get").unwrap();
                assert_eq!(data.as_slice(), value);
            }
        }

        {
            let value = b"ddd";
            db.set(key.to_vec(), value.to_vec()).expect("fail to set");
            db.commit(vec![]);
            {
                let h = db.height().expect("fail to get height");
                assert_eq!(h, 4);
                let data = db.get(key).expect("fail to get").unwrap();
                assert_eq!(data.as_slice(), value);
            }
        }

        db.roll_back(2).expect("fail to rollback");
        let ret = db.get(key).expect("fail to get").unwrap();
        println!("{:?}", String::from_utf8(ret.clone()));
        assert_eq!(value2, ret.clone().as_slice());

        std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
    }
}
