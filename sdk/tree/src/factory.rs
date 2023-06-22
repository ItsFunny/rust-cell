use crate::cache::CacheWrapper;
use crate::merkle::{MerkleRocksDB, MerkleRocksDBConfiguration};
use crate::mpt::new_mpt_with_wrap_db;
use crate::prefix::PrefixWrapper;
use crate::rocksdb::RocksDB;
use crate::shared::{SharedDB, SharedWrapper};
use crate::smt::new_smt_with_wrap_db;
use crate::tree::{TreeDB, DB};
use std::path::Path;

pub fn new_rocks_db<P: AsRef<Path>>(p: P) -> Box<dyn DB> {
    Box::new(RocksDB::new(MerkleRocksDBConfiguration::new(p)))
}

pub fn new_default_merkle<P: AsRef<Path>>(p: P) -> Box<dyn DB> {
    Box::new(MerkleRocksDB::new(MerkleRocksDBConfiguration::new(p)))
}

pub fn new_smt_tree(
    prefix: Vec<u8>,
    db: SharedDB<Box<dyn DB>>,
    root_prefix: Vec<u8>,
) -> SharedWrapper<Box<dyn TreeDB>> {
    SharedWrapper::new(Box::new(PrefixWrapper::new(
        prefix,
        CacheWrapper::new(new_smt_with_wrap_db(db, root_prefix)),
    )))
}

pub fn new_mpt_tree(
    prefix: Vec<u8>,
    db: SharedDB<Box<dyn DB>>,
    root_prefix: Vec<u8>,
) -> SharedWrapper<Box<dyn TreeDB>> {
    SharedWrapper::new(Box::new(PrefixWrapper::new(
        prefix,
        CacheWrapper::new(new_mpt_with_wrap_db(db, root_prefix)),
    )))
}

#[cfg(test)]
mod test {
    use crate::factory::{new_default_merkle, new_mpt_tree, new_smt_tree};
    use crate::shared::SharedDB;
    use crate::tree::{Batch, Read, TreeDB, Write, DB};

    // TODO, release the rocksdb
    fn new_merk() -> Box<dyn DB> {
        Box::new(new_default_merkle("./merk.db"))
    }

    fn new_trees(internal: SharedDB<Box<dyn DB>>) -> (impl TreeDB, impl TreeDB) {
        let smt_prefix = b"smt".to_vec();
        let mpt_prefix = b"mpt".to_vec();
        let smt = new_smt_tree(smt_prefix.clone(), internal.clone(), vec![]);
        let mpt = new_mpt_tree(mpt_prefix.clone(), internal.clone(), vec![]);
        return (smt, mpt);
    }

    #[test]
    pub fn test_roll_back() {
        let mut internal = SharedDB::new(new_merk());
        let trees = new_trees(internal.clone());
        let mut smt = trees.0;
        let mut mpt = trees.1;
        let root1 = smt.root_hash();
        let root2 = mpt.root_hash();
        println!("{:?},{:?}", hex::encode(root1), hex::encode(root2));

        smt.set(b"key1".to_vec(), b"value1".to_vec())
            .expect("fail to set");
        mpt.set(b"key2".to_vec(), b"value2".to_vec())
            .expect("fail to set");
        smt.flush().unwrap();
        mpt.flush().unwrap();
        let root11 = smt.root_hash();
        let root22 = mpt.root_hash();
        println!("{:?},{:?}", hex::encode(root11), hex::encode(root22));
        assert_ne!(root11, root1);
        assert_ne!(root22, root2);

        // failed for some reason
        // roll back
        internal.clean().expect("fail to clean");
        let trees = new_trees(internal.clone());
        let smt = trees.0;
        let mpt = trees.1;
        let root111 = smt.root_hash();
        let root222 = mpt.root_hash();
        println!("{:?},{:?}", hex::encode(root1), hex::encode(root2));

        assert_eq!(root111, root1);
        assert_eq!(root222, root2);

        std::fs::remove_dir_all(internal.clone().get_configuration().clone().home).unwrap();
    }

    #[test]
    pub fn test_roll_back_to_height() {
        let mut internal = SharedDB::new(new_merk());

        for i in 1..10 {
            let key1 = b"key1";
            let value1 = format!("value_1_{}", i);
            let key2 = b"key2";
            let value2 = format!("value_2_{}", i);
            {
                let trees = new_trees(internal.clone());
                let mut smt = trees.0;
                let mut mpt = trees.1;

                smt.set(key1.to_vec(), value1.as_bytes().to_vec())
                    .expect("fail to set");
                mpt.set(key2.to_vec(), value2.as_bytes().to_vec())
                    .expect("fail to set");

                smt.flush().expect("fail to flush");
                mpt.flush().expect("fail to flush");
                internal.commit(vec![]);

                let h = internal.height().unwrap();
                assert_eq!(h, i);
            }
        }

        internal.roll_back(4).unwrap();
        let i = 4;
        let key1 = b"key1";
        let value1 = format!("value_1_{}", i);
        let key2 = b"key2";
        let value2 = format!("value_2_{}", i);
        let trees = new_trees(internal.clone());
        let smt = trees.0;
        let mpt = trees.1;

        let value_1 = { smt.get(key1).unwrap().unwrap() };
        let value_2 = { mpt.get(key2).unwrap().unwrap() };

        println!(
            "{:?},{:?}",
            String::from_utf8(value_1.clone()).unwrap(),
            String::from_utf8(value_2.clone()).unwrap()
        );
        assert_eq!(value1.as_bytes(), value_1.clone().as_slice());
        assert_eq!(value2.as_bytes(), value_2.clone().as_slice());

        for i in 20..30 {
            let key1 = b"key1";
            let value1 = format!("value_1_{}", i);
            let key2 = b"key2";
            let value2 = format!("value_2_{}", i);
            {
                let trees = new_trees(internal.clone());
                let mut smt = trees.0;
                let mut mpt = trees.1;

                smt.set(key1.to_vec(), value1.as_bytes().to_vec())
                    .expect("fail to set");
                mpt.set(key2.to_vec(), value2.as_bytes().to_vec())
                    .expect("fail to set");

                smt.flush().expect("fail to flush");
                mpt.flush().expect("fail to flush");
                internal.commit(vec![]);

                let h = internal.height().unwrap();
                println!("{:?}", h)
            }
        }

        std::fs::remove_dir_all(internal.clone().get_configuration().clone().home).unwrap();
    }

    #[test]
    pub fn test_commit() {
        let mut internal = SharedDB::new(new_merk());
        let trees = new_trees(internal.clone());
        let mut smt = trees.0;
        let mut mpt = trees.1;

        let root1 = smt.root_hash();
        let root2 = mpt.root_hash();
        println!("{:?},{:?}", hex::encode(root1), hex::encode(root2));

        smt.set(b"key1".to_vec(), b"value1".to_vec())
            .expect("fail to set");
        mpt.set(b"key2".to_vec(), b"value2".to_vec())
            .expect("fail to set");

        smt.flush().expect("fail to flush");
        mpt.flush().expect("fail to flush");

        internal.commit(vec![]);
        let root1 = smt.root_hash();
        let root2 = mpt.root_hash();
        println!("{:?},{:?}", hex::encode(root1), hex::encode(root2));

        std::fs::remove_dir_all(internal.clone().get_configuration().clone().home).unwrap();
    }

    // #[test]
    // pub fn test_get() {
    //     let mut internal = SharedDB::new(new_merk());
    //     let smt_prefix = b"smt".to_vec();
    //     let mpt_prefix = b"mpt".to_vec();
    //     let mut smt = new_smt_tree(smt_prefix.clone(), internal.clone());
    //     let mut mpt = new_mpt_tree(mpt_prefix.clone(), internal.clone());
    //
    //     let value1 = smt.get(b"key1").expect("fail to get").unwrap();
    //     let value2 = mpt.get(b"key2").expect("fail to get").unwrap();
    //     println!(
    //         "{:?},{:?}",
    //         String::from_utf8_lossy(value1.as_slice()),
    //         String::from_utf8_lossy(value2.as_slice())
    //     );
    //     assert_eq!(value1.as_slice(), b"value1");
    //     assert_eq!(value2.as_slice(), b"value2");
    // }
}
