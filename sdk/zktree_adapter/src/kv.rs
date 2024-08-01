use crate::{DataHashRecord, default_hash, DEFAULT_ROOT_HASH, DEFAULT_ROOT_HASH64, MerkleAdapter, MerkleDB, MerkleProof, POSEIDON_HASHER};

pub const MERKLE_TREE_HEIGHT: usize = 32;

#[derive(Clone)]
pub struct MerkleKv<T: MerkleDB> {
    merkle: MerkleAdapter<T, MERKLE_TREE_HEIGHT>,
}

impl<T: MerkleDB> MerkleKv<T> {
    pub fn new(store: T, root: [u8; 32]) -> Self {
        Self {
            merkle: MerkleAdapter::<T, MERKLE_TREE_HEIGHT>::new(store),
        }
    }

    pub fn set_root(&mut self, root: &[u8; 32]) {
        self.merkle.update_root_hash(root);
    }

    pub fn get_root(&self) -> [u8; 32] {
        self.merkle.get_root_hash()
    }

    pub async fn set_simple(&mut self, address: u32, v: &[u8; 32]) {
        let index = (address as u64) + (1u64 << MERKLE_TREE_HEIGHT) - 1;
        let mt = &mut self.merkle;
        let mut hash = Vec::new();
        hash.extend_from_slice(v);

        mt.update_leaf_data_with_proof(index, &hash)
            .await
            .expect("Unexpected failure: update leaf with proof fail");
    }

    pub async fn set(&mut self, address: u32, data: &[u8]) {
        let index = (address as u64) + (1u64 << MERKLE_TREE_HEIGHT) - 1;
        let mt = &mut self.merkle;

        // TODO: add hash abstraction
        let hash = default_hash(data).to_vec();
        mt.update_leaf_data_with_proof(index, &hash)
            .await
            .expect("Unexpected failure: update leaf with proof fail");
        // put data and hash into mongo_datahash if the data is binded to the merkle tree leaf
        if !data.is_empty() {
            self.merkle
                .store
                .update_data_hash({
                    DataHashRecord {
                        hash: hash.try_into().unwrap(),
                        data: data.to_vec(),
                    }
                })
                .await
                .unwrap();
        }
    }

    pub async fn get_simple(&mut self, k: u32) -> ([u64; 4], MerkleProof<[u8; 32], 32>) {
        let address = k;
        let index = (address as u64) + (1u64 << MERKLE_TREE_HEIGHT) - 1;
        let mt = &mut self.merkle;
        let (leaf, proof) = mt
            .get_leaf_with_proof(index)
            .await
            .expect("Unexpected failure: get leaf fail");

        (leaf.data_as_u64(), proof)
    }

    pub async fn get(&mut self, k: u32, data: &mut [u64]) -> (u64, MerkleProof<[u8; 32], 32>) {
        let address = k;
        let index = (address as u64) + (1u64 << MERKLE_TREE_HEIGHT) - 1;
        let mt = &mut self.merkle;
        let (leaf, proof) = mt
            .get_leaf_with_proof(index)
            .await
            .expect("Unexpected failure: get leaf fail");

        let hash = leaf.data;

        let datahashrecord = self.merkle.store.get_data_by_hash(&hash).await.unwrap();
        let d = datahashrecord.map_or(vec![], |r| {
            r.data
                .chunks_exact(8)
                .into_iter()
                .into_iter()
                .map(|x| u64::from_le_bytes(x.try_into().unwrap()))
                .collect::<Vec<u64>>()
        });
        for (i, n) in d.iter().enumerate() {
            data[i] = *n;
        }

        (d.len() as u64, proof)
    }
}


fn u64_4_to_u8_32(o: &[u64; 4]) -> &[u8; 32] {
    unsafe { &*(o.as_ptr() as *const [u8; 32]) }
}

