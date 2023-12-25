use crate::{DataHashRecord, MerkleRecord};

#[async_trait::async_trait]
pub trait MerkleDB: Clone {
    async fn get_record(&self, index: u64, hash: &[u8; 32])
                        -> anyhow::Result<Option<MerkleRecord>>;

    async fn update_record(&self, record: MerkleRecord) -> anyhow::Result<()>;

    async fn batch_update_records(&self, records: &Vec<MerkleRecord>) -> anyhow::Result<()>;

    async fn get_data_by_hash(&self, hash: &[u8; 32]) -> anyhow::Result<Option<DataHashRecord>>;

    async fn update_data_hash(&self, record: DataHashRecord) -> anyhow::Result<()>;
}


#[test]
pub fn test_asd() {}