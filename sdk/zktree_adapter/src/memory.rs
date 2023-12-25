use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::{DataHashRecord, MerkleDB, MerkleRecord};

#[derive(Default, Clone)]
pub struct DefaultMemoryDB {
    data: Rc<RefCell<HashMap<String, MerkleRecord>>>,
    data_hash: Rc<RefCell<HashMap<String, DataHashRecord>>>,
}

unsafe impl Send for DefaultMemoryDB {}

unsafe impl Sync for DefaultMemoryDB {}

impl DefaultMemoryDB {
    pub fn new() -> Rc<RefCell<DefaultMemoryDB>> {
        Rc::new(RefCell::new(Self {
            data: Rc::new(RefCell::new(HashMap::new())),
            data_hash: Rc::new(RefCell::new(HashMap::new())),
        }))
    }
}

#[async_trait::async_trait]
impl MerkleDB for DefaultMemoryDB {
    async fn get_record(&self, index: u64, hash: &[u8; 32]) -> anyhow::Result<Option<MerkleRecord>> {
        let key = self.build_key(index, hash);
        let data = self.data.borrow();
        Ok(data.get(&key).cloned())
    }

    async fn update_record(&self, record: MerkleRecord) -> anyhow::Result<()> {
        let key = self.build_key(record.index, &record.hash);
        let mut data = self.data.borrow_mut();
        data.insert(key, record);
        Ok(())
    }

    async fn batch_update_records(&self, records: &Vec<MerkleRecord>) -> anyhow::Result<()> {
        let mut data = self.data.borrow_mut();
        for record in records.iter() {
            let key = self.build_key(record.index, &record.hash);
            data.insert(key, record.clone());
        }
        Ok(())
    }

    async fn get_data_by_hash(&self, hash: &[u8; 32]) -> anyhow::Result<Option<DataHashRecord>> {
        let key = hex::encode(hash);
        let data = self.data_hash.borrow();
        Ok(data.get(&key).cloned())
    }

    async fn update_data_hash(&self, record: DataHashRecord) -> anyhow::Result<()> {
        let key = hex::encode(&record.hash);
        let mut data = self.data_hash.borrow_mut();
        data.insert(key, record);
        Ok(())
    }
}


impl DefaultMemoryDB {
    fn build_key(&self, index: u64, hash: &[u8; 32]) -> String {
        format!("{}-{}", index, hex::encode(hash))
    }

    pub fn batch_get_merkle_records(
        &self,
        records: &Vec<MerkleRecord>,
    ) -> anyhow::Result<(Vec<MerkleRecord>, Vec<MerkleRecord>)> {
        let mut find = vec![];
        let mut not_find = records.clone();

        for rec in records {
            let got = self.get_merkle_record(rec.index, &rec.hash)?;
            if got.is_some() {
                find.push(rec.clone());
                not_find.remove(not_find.iter().position(|x| x == rec).unwrap());
            }
        }

        Ok((find, not_find))
    }
}
