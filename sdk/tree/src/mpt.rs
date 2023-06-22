use crate::error::{TreeError, TreeResult};
use crate::merkle::{MerkleRocksDB, MerkleRocksDBConfiguration};
use crate::operation::{Operation, RollBackOperation};
use crate::tree::{batch_operation, Batch, NullHasher, Read, RootHash, TreeDB, Write, DB, KV};

use hash_db::{AsHashDB, HashDB, HashDBRef, Hasher, Prefix};
use keccak_hasher::KeccakHasher;

use sp_trie::{LayoutV0, TrieDBBuilder};
use std::any::Any;
use std::cell::Cell;

use crate::couple::{
    MPTProveResponse, ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse,
};
use crate::iter::concat;
use crate::mem::slice_to_array_32;
use crate::shared::SharedDB;
use trie_db::proof::{generate_proof, verify_proof};
use trie_db::{Trie, TrieDBMut, TrieDBMutBuilder, TrieMut};

// TODO, wrapped with db
pub struct MPTreeDB {
    merk: SharedDB<Box<dyn DB>>,
    null_node_data: Vec<u8>,
    hashed_null_node: [u8; 32],
    root_prefix: Vec<u8>,
}

impl MPTreeDB {
    fn extend_prefix<H: Hasher>(&self, key: &H::Out, _prefix: Prefix) -> Vec<u8> {
        key.as_ref().to_vec()
        // prefixed_key::<H>(key, prefix)
    }
    pub fn new(
        m: SharedDB<Box<dyn DB>>,
        null_key: &[u8],
        null_node_data: Vec<u8>,
        root_prefix: Vec<u8>,
    ) -> Self {
        Self {
            merk: m,
            null_node_data,
            hashed_null_node: NullHasher::hash(null_key),
            root_prefix,
        }
    }
    pub fn new_from_null_node(m: SharedDB<Box<dyn DB>>, root_prefix: Vec<u8>) -> Self {
        MPTreeDB::new(m, &[0u8][..], [0u8][..].into(), root_prefix)
    }
    fn get_slice_root_prefix(&self) -> &[u8] {
        self.root_prefix.as_slice()
    }
    fn commit_internal(&mut self, root: Vec<u8>) {
        let key = build_root_key(self.get_slice_root_prefix());
        self.merk
            .commit(vec![Operation::Set(key.clone(), root.clone())]);
    }
}

pub type KeccakHash = [u8; 32];

pub fn build_height_key(h: u64) -> Vec<u8> {
    let mut ret = Vec::new();
    ret.extend_from_slice(b"height_hash");
    ret.extend_from_slice(h.to_be_bytes().as_slice());
    return ret;
}

fn build_root_key(root_prefix: &[u8]) -> Vec<u8> {
    concat(root_prefix, b"mpt_root")
}

impl AsHashDB<NullHasher, Vec<u8>> for MPTreeDB {
    fn as_hash_db(&self) -> &dyn HashDB<NullHasher, Vec<u8>> {
        self
    }

    fn as_hash_db_mut<'a>(&'a mut self) -> &'a mut (dyn HashDB<NullHasher, Vec<u8>> + 'a) {
        self
    }
}

//
impl HashDB<NullHasher, Vec<u8>> for MPTreeDB {
    fn get(&self, key: &<NullHasher as Hasher>::Out, prefix: Prefix) -> Option<Vec<u8>> {
        if key == &self.hashed_null_node {
            return Some(self.null_node_data.clone());
        }
        let k = self.extend_prefix::<NullHasher>(key, prefix);
        self.merk
            .get(k.clone().as_slice())
            .map_or_else(|_e| None, |v| v)
    }

    fn contains(&self, key: &<NullHasher as Hasher>::Out, prefix: Prefix) -> bool {
        HashDB::get(self, key, prefix).map_or_else(|| false, |_v| true)
    }

    fn insert(&mut self, p: Prefix, value: &[u8]) -> <NullHasher as Hasher>::Out {
        if value == self.null_node_data {
            return self.hashed_null_node;
        }
        let key = NullHasher::hash(value);
        self.emplace(key, p, value.into());
        key
    }

    fn emplace(&mut self, key: <NullHasher as Hasher>::Out, prefix: Prefix, value: Vec<u8>) {
        if value == self.null_node_data {
            return;
        }
        let key = self.extend_prefix::<NullHasher>(&key, prefix);
        self.merk.set(key, value).expect("fail to set");
    }

    fn remove(&mut self, key: &<NullHasher as Hasher>::Out, prefix: Prefix) {
        if key == &self.hashed_null_node {
            return;
        }
        let key = self.extend_prefix::<NullHasher>(&key, prefix);
        self.merk.delete(key).expect("fail to delete");
    }
}

pub struct TrieDB<'a> {
    trie: Cell<Option<TrieDBMut<'a, sp_trie::LayoutV0<NullHasher>>>>,
    configuration: MerkleRocksDBConfiguration,
    internal_db: Option<SharedDB<Box<dyn DB>>>,
    root_prefix: Vec<u8>,
}

unsafe impl<'a> Send for TrieDB<'a> {}

unsafe impl<'a> Sync for TrieDB<'a> {}

impl<'a> TrieDB<'a> {
    pub fn new(
        db: &'a mut MPTreeDB,
        merk: SharedDB<Box<dyn DB>>,
        root: &'a mut [u8; 32],
        root_prefix: Vec<u8>,
    ) -> TrieDB<'a> {
        let trie;
        let def: &[u8; 32] = &Default::default();
        if root == def {
            trie = TrieDBMutBuilder::new(db, root).build();
        } else {
            trie = TrieDBMutBuilder::from_existing(db, root).build();
        }
        let cfg = merk.borrow().get_configuration();
        TrieDB {
            trie: Cell::new(Some(trie)),
            configuration: cfg,
            internal_db: Some(merk),
            root_prefix,
        }
    }
    pub fn get_db(&self) -> SharedDB<Box<dyn DB>> {
        self.internal_db.as_ref().unwrap().clone()
    }
    fn get_slice_root_prefix(&self) -> &[u8] {
        self.root_prefix.as_slice()
    }

    pub fn replace(&mut self) {
        let trie = new_mpt_from_cfg(self.configuration.clone(), self.root_prefix.clone());
        let internal_db = trie.get_db();
        let tree = trie.trie.take();
        self.trie.replace(tree);
        self.internal_db.replace(internal_db);
    }
}

impl<'a> TrieDB<'a> {
    fn use_db<T>(
        &self,
        f: impl FnOnce(Option<&TrieDBMut<sp_trie::LayoutV0<NullHasher>>>) -> T,
    ) -> T {
        let tree = self.trie.take();
        let res = f(tree.as_ref());
        self.trie.set(tree);
        res
    }
    fn use_db_mut<T>(
        &self,
        f: impl FnOnce(Option<&mut TrieDBMut<sp_trie::LayoutV0<NullHasher>>>) -> T,
    ) -> T {
        let mut tree = self.trie.take();
        let res = f(tree.as_mut());
        self.trie.set(tree);
        res
    }
}

impl<'a> Read for TrieDB<'a> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        self.use_db(|v| v.unwrap().get(k).map_err(|e| TreeError::from(e)))
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        let hash = self.root_hash();
        self.use_db_mut(|tree| {
            let db = tree.unwrap();
            let binding = db.db();
            let db = &binding.as_hash_db() as &dyn HashDBRef<NullHasher, Vec<u8>>;
            let trie_db = TrieDBBuilder::<LayoutV0<NullHasher>>::new(db, &hash).build();
            let mut iter = trie_db.iter().expect("fail to get iterator");
            iter.seek(start).expect("fail to seek");
            // TODO, ugly codes
            return match iter.next() {
                Some(Ok((key, value))) => {
                    if key.clone().as_slice() == start {
                        match iter.next() {
                            None => Ok(None),
                            Some(Ok((key2, value2))) => Ok(Some((key2, value2))),
                            Some(Err(_error)) => {
                                // TODO
                                Err(TreeError::Unknown)
                            }
                        }
                    } else {
                        Ok(Some((key, value)))
                    }
                }
                Some(Err(_error)) => {
                    // TODO
                    Err(TreeError::Unknown)
                }
                _ => Ok(None),
            };
        })
    }

    fn key_decorator(&self, k: Vec<u8>) -> Vec<u8> {
        KeccakHasher::hash(k.as_slice()).to_vec()
    }
}

impl<'a> Write for TrieDB<'a> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        self.use_db_mut(|db| {
            db.unwrap()
                .insert(k.as_slice(), v.as_slice())
                .map_err(|e| TreeError::from(e))
                .map(|_v| k.clone())
        })
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        self.use_db_mut(|v| {
            v.unwrap()
                .remove(k.as_slice())
                .map_err(|e| TreeError::from(e))
                .map(|_v| {})
        })
    }
}

impl<'a> DB for TrieDB<'a> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        self.internal_db.as_ref().unwrap().get_configuration()
    }

    fn roll_back(&mut self, h: u64) -> TreeResult<()> {
        self.internal_db
            .as_mut()
            .unwrap()
            .borrow_mut()
            .roll_back_with_operation(RollBackOperation::MerkleHeight(h, Some(false)))?;
        self.replace();
        Ok(())
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        self.internal_db
            .as_mut()
            .unwrap()
            .borrow_mut()
            .roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        self.internal_db.as_ref().unwrap().borrow().height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        // mpt wont do anything about clean
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

impl<'a> Batch for TrieDB<'a> {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        let aux_ops = batch_operation(self, operations).expect("TODO: panic message");
        let root = self.use_db_mut(|tree| {
            let internal_tree = tree.unwrap();
            *internal_tree.root()
        });
        self.commit_internal(aux_ops, root);
        root
    }
}

impl<'a> TreeDB for TrieDB<'a> {
    fn prove(&self, req_enums: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        if let ProveRequestEnums::MPTProve(req) = req_enums {
            self.use_db_mut(|db| {
                let d = db.unwrap();
                let internal_db = &d.db_mut();
                generate_proof::<_, sp_trie::LayoutV0<NullHasher>, _, _>(
                    internal_db,
                    &req.root,
                    &req.query,
                )
                .map_err(|e| TreeError::from(e))
                .map(|v| ProveResponseEnums::MPTProve(MPTProveResponse::new(v)))
            })
        } else {
            unreachable!()
        }
    }

    fn verify(&self, req_enums: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        if let VerifyRequestEnums::MPTVerify(req) = req_enums {
            Ok(verify_proof::<sp_trie::LayoutV0<NullHasher>, _, _, _>(
                &req.root,
                req.proof.as_slice(),
                &req.query,
            )
            .map_or(VerifyResponse::default(), |_| VerifyResponse {
                valid: true,
            }))
        } else {
            unreachable!()
        }
    }

    fn root_hash(&self) -> RootHash {
        self.use_db_mut(|db| *db.unwrap().root())
    }
}

impl<'a> TrieDB<'a> {
    pub fn call_internal(&self, _any: Box<dyn Any>) {}

    fn commit_internal(&mut self, mut aux: Vec<Operation>, root: RootHash) {
        let key = build_root_key(self.get_slice_root_prefix());
        aux.push(Operation::Set(key, root.as_slice().to_vec()));
        self.internal_db.as_mut().unwrap().borrow_mut().commit(aux);
    }
}

unsafe impl Send for MPTreeDB {}

unsafe impl Sync for MPTreeDB {}

fn get_mpt_root(merk: SharedDB<Box<dyn DB>>, root_prefix: Vec<u8>) -> [u8; 32] {
    let key = build_root_key(root_prefix.as_slice());
    let root: [u8; 32] = {
        merk.get(key.as_slice())
            .expect("fail to get root")
            .map_or_else(|| Default::default(), |v| *slice_to_array_32(v.as_slice()))
    };
    root
}

pub fn new_mpt_from_cfg<'a>(m: MerkleRocksDBConfiguration, root_prefix: Vec<u8>) -> TrieDB<'a> {
    let db: Box<dyn DB> = Box::new(MerkleRocksDB::new(m));
    let m = SharedDB::new(db);
    new_mpt_with_wrap_db(m, root_prefix)
}

pub fn new_mpt_with_wrap_db<'a>(
    internal_db: SharedDB<Box<dyn DB>>,
    root_prefix: Vec<u8>,
) -> TrieDB<'a> {
    let couple = new_mpt_db(internal_db.clone(), root_prefix.clone());
    let mpt = couple.0;
    let binding = Box::new(couple.1);
    let leak = Box::leak(binding);
    let leak_db = Box::leak(Box::new(mpt));
    let internal = TrieDB::new(leak_db, internal_db.clone(), leak, root_prefix.clone());
    internal
}

fn new_mpt_db(merk: SharedDB<Box<dyn DB>>, root_prefix: Vec<u8>) -> (MPTreeDB, [u8; 32]) {
    let root = get_mpt_root(merk.clone(), root_prefix.clone());
    let mpt = MPTreeDB::new_from_null_node(merk.clone(), root_prefix.clone());
    return (mpt, root);
}

#[cfg(test)]
mod test {

    use crate::merkle::{MerkleRocksDB, MerkleRocksDBConfiguration};
    use crate::mpt::{new_mpt_with_wrap_db, TrieDB};
    use crate::tree::{Batch, Read, Write, DB};

    use crate::shared::SharedDB;
    use std::path::Path;

    fn mpt<'a, P: AsRef<Path>>(p: P) -> TrieDB<'a> {
        new_mpt_with_wrap_db(new_merk_db(p), vec![])
    }

    fn new_merk_db<P: AsRef<Path>>(p: P) -> SharedDB<Box<dyn DB>> {
        let cfg = MerkleRocksDBConfiguration::new(p);
        SharedDB::new(Box::new(MerkleRocksDB::new(cfg)))
    }

    #[test]
    pub fn test_st() {
        let mut mpt = mpt("./merk.db");
        mpt.set(vec![10, 11, 12], vec![13, 14, 15])
            .expect("fail to set");
        mpt.set(vec![20, 21, 22], vec![23, 24, 25])
            .expect("fail to set");
        mpt.set(vec![1, 2], vec![4, 5, 6]).expect(" fail to set");
        let ret = mpt
            .get_next(vec![10].as_slice())
            .expect("fail to get next")
            .unwrap();
        assert_eq!(ret.0, vec![10, 11, 12]);
        assert_eq!(ret.1, vec![13, 14, 15]);

        std::fs::remove_dir_all(mpt.get_configuration().clone().home).unwrap();
    }

    #[test]
    pub fn test_internal() {
        let mut mpt = mpt("./merk.db");
        mpt.set(vec![10, 11, 12], vec![13, 14, 15])
            .expect("fail to set");
        mpt.set(vec![20, 21, 22], vec![23, 24, 25])
            .expect("fail to set");
        mpt.set(vec![1, 2], vec![4, 5, 6]).expect(" fail to set");
        mpt.commit(vec![]);

        std::fs::remove_dir_all(mpt.get_configuration().clone().home).unwrap();
    }

    #[test]
    pub fn test_roll_back() {
        let mut db = mpt("./merk.db");
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

        db.roll_back(1).expect("fail to rollback");
        let ret = db.get(key).expect("fail to get").unwrap();
        println!("{:?}", String::from_utf8(ret.clone()));
        assert_eq!(value, ret.clone().as_slice());

        std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
    }
}
