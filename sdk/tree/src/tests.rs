#[cfg(test)]
mod test {

    use crate::merkle::{MerkleRocksDB, MerkleRocksDBConfiguration};

    use crate::operation::Operation;
    use crate::rocksdb::RocksDB;
    use crate::smt::new_smt;
    use crate::tree::{Batch, Read, Write, DB};

    struct Temp {}

    fn new_dbs() -> Vec<Box<dyn DB>> {
        let mut dbs: Vec<Box<dyn DB>> = Vec::new();
        // dbs.push(
        //     Box::new(RocksDB::new(MerkleRocksDBConfiguration::new("./merk.db")))
        // );
        // dbs.push(
        //     Box::new(MerkleRocksDB::new(MerkleRocksDBConfiguration::new("./merk2.db")))
        // );
        //
        let smt: Box<dyn DB> = Box::new(new_smt(
            MerkleRocksDBConfiguration::new("./merk3.db"),
            vec![],
        ));
        dbs.push(smt);

        // let mpt: Box<dyn DB> = Box::new(new_mpt_from_cfg(MerkleRocksDBConfiguration::new("./merk4.db"), vec![]));
        // dbs.push(mpt);
        return dbs;
    }

    #[test]
    pub fn test_commit() {
        let dbs = new_dbs();
        for f in dbs {
            let mut db = f;
            let ops = vec![Operation::Set(b"key1".to_vec(), b"value1".to_vec())];
            db.commit(ops);

            let value = db.get(b"key1").unwrap().unwrap();
            assert_eq!(value.as_slice(), b"value1");

            std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
        }
    }

    #[test]
    pub fn test_delete() {
        let dbs = new_dbs();

        for f in dbs {
            let mut db = f;

            db.set(b"key1".to_vec(), b"value1".to_vec()).unwrap();
            db.commit(vec![]);
            //
            let value = db.get(b"key1").unwrap().unwrap();
            assert_eq!(value.as_slice(), b"value1");
            //
            db.delete(b"key1".to_vec()).unwrap();
            db.commit(vec![]);
            //
            let v = db.get(b"key1").unwrap();
            assert_eq!(v, None);

            std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
        }
    }

    #[test]
    pub fn test_roll_back() {
        let dbs = new_dbs();

        for f in dbs {
            let mut db = f;
            for i in 1..10 {
                let value = format!("value_{}", i);
                db.set(b"key1".to_vec(), value.as_bytes().to_vec()).unwrap();
                db.commit(vec![]);
                let vv = db.get(b"key1").unwrap().unwrap();
                assert_eq!(vv.as_slice(), value.as_bytes())
            }

            db.roll_back(3).unwrap();
            let i = 3;
            let value = format!("value_{}", i);
            let db_value = db.get(b"key1").unwrap().unwrap();
            println!("{:?}", hex::encode(db_value.clone()));
            assert_eq!(value.as_bytes(), db_value.clone().as_slice());
            let mut start_h = 4;
            for i in 1..10 {
                let value = format!("value_{}", i);
                db.set(b"key1".to_vec(), value.as_bytes().to_vec()).unwrap();
                db.commit(vec![]);
                let h = db.height().unwrap();
                assert_eq!(h, start_h);
                start_h = start_h + 1;
            }

            std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
        }
    }

    #[test]
    pub fn test_get_next() {
        let dbs = new_dbs();

        for mut db in dbs {
            db.set(vec![0, 0], vec![0]).unwrap();
            db.set(vec![1, 0], vec![1]).unwrap();
            db.set(vec![1, 1], vec![2]).unwrap();
            db.set(vec![2, 0], vec![3]).unwrap();
            db.set(vec![4, 0], vec![4]).unwrap();
            db.set(vec![10, 0], vec![5]).unwrap();
            db.commit(vec![]);

            assert_eq!(
                (vec![0, 0], vec![0]),
                db.get_next(vec![0].as_slice()).unwrap().unwrap()
            );
            assert_eq!(
                (vec![1, 0], vec![1]),
                db.get_next(vec![1].as_slice()).unwrap().unwrap()
            );
            assert_eq!(
                (vec![10, 0], vec![5]),
                db.get_next(vec![8].as_slice()).unwrap().unwrap()
            );
            std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
        }
    }

    #[test]
    pub fn test_height() {
        let dbs = new_dbs();

        for mut db in dbs {
            for i in 1..10 {
                db.commit(vec![]);
                let h = db.height().unwrap();
                assert_eq!(i, h)
            }

            std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
        }
    }

    #[test]
    pub fn test_clean() {
        let dbs = {
            let mut ret: Vec<Box<dyn DB>> = Vec::new();
            ret.push(Box::new(RocksDB::new(MerkleRocksDBConfiguration::new(
                "./merk.db",
            ))));
            ret.push(Box::new(MerkleRocksDB::new(
                MerkleRocksDBConfiguration::new("./merk2.db"),
            )));
            ret
        };
        for mut db in dbs {
            db.set(vec![1, 2], vec![0]).unwrap();
            let v = db.get(vec![1, 2].as_slice()).unwrap();
            assert!(v.is_some());
            let value = v.unwrap();
            assert_eq!(value.as_slice(), vec![0]);

            db.clean().unwrap();

            let v = db.get(vec![1, 2].as_slice()).unwrap();
            assert!(v.is_none());

            std::fs::remove_dir_all(db.get_configuration().clone().home).unwrap();
        }
    }
}
