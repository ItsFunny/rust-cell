use std::ops::{Bound, RangeBounds};

use crate::couple::{ProveRequestEnums, ProveResponseEnums, VerifyRequestEnums, VerifyResponse};
use hash256_std_hasher::Hash256StdHasher;
use hash_db::Hasher;
use keccak_hasher::KeccakHasher;

use crate::error::TreeResult;
use crate::iter::Iter;
use crate::merkle::MerkleRocksDBConfiguration;
use crate::operation::{Operation, RollBackOperation};

pub type RootHash = [u8; 32];
pub(crate) type KV = (Vec<u8>, Vec<u8>);

pub trait KeyHasher: Hasher {}

pub struct NullHasher {}

//
impl Hasher for NullHasher {
    type Out = [u8; 32];
    type StdHasher = Hash256StdHasher;
    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Self::Out {
        KeccakHasher::hash(x)
        // slice_to_array(x)
    }
}

// fn slice_to_array(slice: &[u8]) -> ([u8; 32]) {
// TODO ,add debug feature :panic when the x'len is not eq 32
// if slice.len() == 32 {
//     let ptr = slice.as_ptr() as *const [u8; 32];
//     unsafe { (*ptr) }
// } else {
// TODO
// KeccakHasher::hash(slice)
// }
// }

impl KeyHasher for NullHasher {}

pub trait Read {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>>;
    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>>;
    fn get_next_inclusive(&self, key: &[u8]) -> TreeResult<Option<KV>> {
        match self.get(key)? {
            Some(value) => Ok(Some((key.to_vec(), value))),
            None => self.get_next(key),
        }
    }

    fn key_decorator(&self, key: Vec<u8>) -> Vec<u8> {
        key
    }
    #[inline]
    fn into_iter<'a, B: RangeBounds<Vec<u8>>>(
        &'a self,
        bounds: B,
    ) -> TreeResult<Box<dyn Iterator<Item = TreeResult<KV>> + 'a>>
    where
        Self: Sized,
    {
        Ok(Box::new(Iter::new(
            self,
            (
                clone_bound(bounds.start_bound()),
                clone_bound(bounds.end_bound()),
            ),
        )))
    }
}

pub trait Batch {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash;
}

pub trait Write: Read {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>>;

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()>;
}

pub trait DB: Write + Batch {
    // fn batch_operation(&mut self, ops: Vec<Operation>) -> Result<()>;
    fn get_configuration(&self) -> MerkleRocksDBConfiguration;

    fn roll_back(&mut self, h: u64) -> TreeResult<()> {
        self.roll_back_with_operation(RollBackOperation::MerkleHeight(h, None))
    }
    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()>;

    fn height(&self) -> TreeResult<u64>;

    fn clean(&mut self) -> TreeResult<()>;
    // flush the data to the next db:
    // depends on the next db,it may not write the data to the disk ,
    // it is useful with the SharedWrapper to merge the child prefix trees
    fn flush(&mut self) -> TreeResult<()>;
}

pub trait TreeDB: DB {
    fn prove(&self, req: ProveRequestEnums) -> TreeResult<ProveResponseEnums>;
    fn verify(&self, req: VerifyRequestEnums) -> TreeResult<VerifyResponse>;
    fn root_hash(&self) -> RootHash;
}

fn clone_bound<T: Clone>(bound: Bound<&T>) -> Bound<T> {
    match bound {
        Bound::Unbounded => Bound::Unbounded,
        Bound::Included(key) => Bound::Included(key.clone()),
        Bound::Excluded(key) => Bound::Excluded(key.clone()),
    }
}

pub(crate) fn batch_operation<T: Write>(
    t: &mut T,
    ops: Vec<Operation>,
) -> TreeResult<Vec<Operation>> {
    let mut aux = Vec::new();
    for op in ops {
        match op {
            Operation::Set(k, v) => {
                // TODO
                t.set(k, v)?;
            }
            Operation::Delete(k) => {
                t.delete(k)?;
            }
            Operation::Aux(v) => {
                aux.push(Operation::Aux(v));
            }
        }
    }
    Ok(aux)
}

impl Batch for Box<dyn TreeDB> {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        (**self).commit(operations)
    }
}

impl DB for Box<dyn TreeDB> {
    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        (**self).get_configuration()
    }

    fn roll_back(&mut self, h: u64) -> TreeResult<()> {
        (**self).roll_back(h)
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        (**self).roll_back_with_operation(ops)
    }

    fn height(&self) -> TreeResult<u64> {
        (**self).height()
    }

    fn clean(&mut self) -> TreeResult<()> {
        (**self).clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        (**self).flush()
    }
}

impl Write for Box<dyn TreeDB> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        (**self).set(k, v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        (**self).delete(k)
    }
}

impl Read for Box<dyn TreeDB> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        (**self).get(k)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        (**self).get_next(start)
    }
}

impl TreeDB for Box<dyn TreeDB> {
    fn prove(&self, req: ProveRequestEnums) -> TreeResult<ProveResponseEnums> {
        (**self).prove(req)
    }

    fn verify(&self, req: VerifyRequestEnums) -> TreeResult<VerifyResponse> {
        (**self).verify(req)
    }

    fn root_hash(&self) -> RootHash {
        (**self).root_hash()
    }
}

impl Write for Box<dyn DB> {
    fn set(&mut self, k: Vec<u8>, v: Vec<u8>) -> TreeResult<Vec<u8>> {
        (**self).set(k, v)
    }

    fn delete(&mut self, k: Vec<u8>) -> TreeResult<()> {
        (**self).delete(k)
    }
}

impl Read for Box<dyn DB> {
    fn get(&self, k: &[u8]) -> TreeResult<Option<Vec<u8>>> {
        (**self).get(k)
    }

    fn get_next(&self, start: &[u8]) -> TreeResult<Option<KV>> {
        (**self).get_next(start)
    }
}

impl Batch for Box<dyn DB> {
    fn commit(&mut self, operations: Vec<Operation>) -> RootHash {
        (**self).commit(operations)
    }
}

impl DB for Box<dyn DB> {
    fn height(&self) -> TreeResult<u64> {
        (**self).height()
    }

    fn get_configuration(&self) -> MerkleRocksDBConfiguration {
        (**self).get_configuration()
    }

    fn roll_back_with_operation(&mut self, ops: RollBackOperation) -> TreeResult<()> {
        (**self).roll_back_with_operation(ops)
    }

    fn clean(&mut self) -> TreeResult<()> {
        (**self).clean()
    }

    fn flush(&mut self) -> TreeResult<()> {
        (**self).flush()
    }

    fn roll_back(&mut self, h: u64) -> TreeResult<()> {
        (**self).roll_back(h)
    }
}

#[cfg(test)]
mod test {
    use crate::merkle::{MerkleRocksDB, MerkleRocksDBConfiguration};
    use crate::tree::{Batch, Read, Write, DB};

    use std::thread;

    #[test]
    pub fn test_iterator() {
        let path = thread::current().name().unwrap().to_owned();
        println!("{:?}", path);
        let mut db = MerkleRocksDB::new(MerkleRocksDBConfiguration::new(path));
        db.set(vec![1, 2, 3], vec![4, 56]).expect("fail to set");
        db.set(vec![4, 5, 6], vec![7, 8, 9]).expect("fail to set");
        db.set(vec![9, 9, 9], vec![1, 2, 5]).expect("fail to set");
        db.commit(vec![]);
        let res = db.get_next(vec![1, 2, 3].as_slice()).expect("fail to get");
        let values = res.unwrap();
        assert_eq!(values.0, vec![4, 5, 6]);
        assert_eq!(values.1, vec![7, 8, 9]);

        std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
    }
}
