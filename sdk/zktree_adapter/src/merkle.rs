use crate::db::MerkleDB;
use crate::error::{MerkleError, MerkleErrorCode, MerkleResult};
use crate::{default_merkle_hash, MerkleProof, MerkleRecord};
use crate::MERKLE_HASHER;

// In default_hash vec, it is from leaf to root.
// For example, suppose that the height of merkle tree is 20.
// DEFAULT_HASH_VEC[0] represents the default leaf hash.
// DEFAULT_HASH_VEC[20] is root default hash.
// It has 21 layers including the leaf layer and root layer.
lazy_static::lazy_static! {
    pub static ref DEFAULT_HASH_VEC: Vec<[u8; 32]> = {
        let mut leaf_hash = MerkleAdapter::<64>::empty_leaf(0).hash;
        let mut default_hash = vec![leaf_hash];
        for _ in 0..(MongoMerkle::<64>::height()) {
            leaf_hash = MerkleAdapter::<64>::hash(&leaf_hash, &leaf_hash);
            default_hash.push(leaf_hash);
        }
        default_hash
    };
}

#[derive(Clone)]
pub struct MerkleAdapter<T: MerkleDB, const DEPTH: usize> {
    pub store: T,
    default_hash: Vec<[u8; 32]>,
}

unsafe impl<T: MerkleDB, const DEPTH: usize> Sync for MerkleAdapter<T, DEPTH> {}

unsafe impl<T: MerkleDB, const DEPTH: usize> Send for MerkleAdapter<T, DEPTH> {}

impl<T: MerkleDB, const DEPTH: usize> MerkleAdapter<T, DEPTH> {
    pub fn new(store: T) -> Self {
        MerkleAdapter {
            store,
            default_hash: (*DEFAULT_HASH_VEC).clone(),
        }
    }

    async fn get_or_generate_node(
        &self,
        index: u64,
        hash: &[u8; 32],
    ) -> MerkleResult<(MerkleRecord, bool)> {
        let mut exist = false;
        let node = self
            .store
            .get_record(index, hash)
            .await
            .expect("Unexpected DB Error")
            .map_or_else(
                || -> MerkleResult<MerkleRecord> {
                    Ok(self.check_generate_default_node(index, hash)?)
                },
                |x| -> MerkleResult<MerkleRecord> {
                    exist = true;
                    assert_eq!(x.index(), index);
                    Ok(x)
                },
            )?;
        Ok((node, exist))
    }

    pub(crate) fn get_root_hash(&self) -> [u8; 32] {
        self.get_root_hash()
    }

    pub(crate) fn update_root_hash(&mut self, hash: &[u8; 32]) {
        self.update_root_hash(hash)
    }

    pub async fn set_parent(
        &mut self,
        index: u64,
        hash: &[u8; 32],
        left: &[u8; 32],
        right: &[u8; 32],
    ) -> MerkleResult<()> {
        self.boundary_check(index)?;
        let record = MerkleRecord {
            index,
            data: [0; 32],
            left: *left,
            right: *right,
            hash: *hash,
        };

        self.store
            .update_record(record)
            .await
            .expect("Unexpected DB Error");
        Ok(())
    }

    async fn set_leaf_and_parents(
        &mut self,
        leaf: &MerkleRecord,
        parents: [(u64, [u8; 32], [u8; 32], [u8; 32]); DEPTH],
    ) -> MerkleResult<()> {
        self.leaf_check(leaf.index())?;
        let mut records: Vec<MerkleRecord> = parents
            .map(|(index, hash, left, right)| MerkleRecord {
                index,
                data: [0; 32],
                left,
                right,
                hash,
            })
            .to_vec();

        records.push(leaf.clone());
        self.update_leaf_path_records(&records)
            .await
            .expect("Unexpected DB Error when update records.");

        Ok(())
    }

    pub async fn update_leaf_path_records(
        &mut self,
        records: &Vec<MerkleRecord>,
    ) -> Result<(), mongodb::error::Error> {
        // sort records by index to ensure the parent node is processed before its child nodes.
        let mut sort_records: Vec<MerkleRecord> = records.clone();
        sort_records.sort_by(|r1, r2| r1.index.cmp(&r2.index));

        let mut new_records: Vec<MerkleRecord> = vec![];
        let mut check = true;
        for record in sort_records {
            // figure out whether a parent node had been added or not,
            // to save the cache/db reading for its child nodes for optimizing.
            if check {
                match self.store.get_record(record.index, &record.hash).await {
                    Ok(Some(_)) => {}
                    _ => {
                        check = false;
                        new_records.push(record);
                    }
                }
            } else {
                new_records.push(record);
            }
        }

        if new_records.len() > 0 {
            self.store
                .batch_update_records(&new_records)
                .await
                .map_err(|e| mongodb::error::Error::custom(e))?;
        }
        Ok(())
    }
    pub async fn get_node_with_hash(
        &self,
        index: u64,
        hash: &[u8; 32],
    ) -> MerkleResult<MerkleRecord> {
        let (node, _) = self.get_or_generate_node(index, hash).await?;
        Ok(node)
    }

    pub async fn set_leaf(&mut self, leaf: &MerkleRecord) -> MerkleResult<()> {
        self.leaf_check(leaf.index())?;
        self.store
            .update_record(leaf.clone())
            .await
            .expect("Unexpected DB Error");
        Ok(())
    }

    pub(crate) async fn get_leaf_with_proof(
        &self,
        index: u64,
    ) -> MerkleResult<(MerkleRecord, MerkleProof<[u8; 32], DEPTH>)> {
        self.leaf_check(index)?;
        let paths = self.get_path(index)?.to_vec();
        // We push the search from the top
        let hash = self.get_root_hash();
        let mut acc = 0;
        let (mut acc_node, mut parent_exist) = self.get_or_generate_node(acc, &hash).await?;

        let mut assist: Vec<[u8; 32]> = Vec::new();
        for child in &paths {
            let child = child.clone();
            let (hash, sibling_hash) = if (acc + 1) * 2 == child + 1 {
                // left child
                (acc_node.left().unwrap(), acc_node.right().unwrap())
            } else {
                assert_eq!((acc + 1) * 2, child);
                (acc_node.right().unwrap(), acc_node.left().unwrap())
            };
            acc = child;
            if parent_exist {
                (acc_node, parent_exist) = self.get_or_generate_node(acc, &hash).await?
            } else {
                acc_node = self.check_generate_default_node(acc, &hash)?
            }
            assist.push(sibling_hash);
        }

        let hash = acc_node.hash();
        Ok((
            acc_node,
            MerkleProof {
                source: hash,
                root: self.get_root_hash(),
                assist: assist.try_into().unwrap(),
                index,
            },
        ))
    }

    pub fn boundary_check(&self, index: u64) -> MerkleResult<()> {
        if index >= (2_u64.pow(DEPTH as u32 + 1) - 1) {
            Err(MerkleError::WithErrorCode([0; 32], index, MerkleErrorCode::InvalidIndex))
        } else {
            Ok(())
        }
    }
    pub fn check_generate_default_node(
        &self,
        index: u64,
        hash: &[u8; 32],
    ) -> Result<MerkleRecord, MerkleError> {
        let node = self.generate_default_node(index)?;
        if node.hash() == *hash {
            Ok(node)
        } else {
            Err(MerkleError::new(*hash, index, MerkleErrorCode::InvalidHash))
        }
    }

    fn get_default_hash(&self, depth: usize) -> Result<[u8; 32], MerkleError> {
        if depth <= Self::height() {
            Ok(self.default_hash[Self::height() - depth])
        } else {
            Err(MerkleError::new(
                [0; 32],
                depth as u64,
                MerkleErrorCode::InvalidDepth,
            ))
        }
    }

    pub fn generate_default_node(&self, index: u64) -> Result<MerkleRecord, MerkleError> {
        let height = (index + 1).ilog2();
        let default = self.get_default_hash(height as usize)?;
        let child_hash = if height == Self::height() as u32 {
            [0; 32]
        } else {
            self.get_default_hash((height + 1) as usize)?
        };

        Ok(MerkleRecord {
            index,
            hash: default,
            data: [0; 32],
            left: child_hash,
            right: child_hash,
        })
    }

    pub fn leaf_check(&self, index: u64) -> Result<(), MerkleError> {
        if (index) >= (2_u64.pow(DEPTH as u32) - 1)
            && (index) < (2_u64.pow((DEPTH as u32) + 1) - 1)
        {
            Ok(())
        } else {
            Err(MerkleError::new(
                [0; 32],
                index,
                MerkleErrorCode::InvalidLeafIndex,
            ))
        }
    }

    // fn get_sibling_index(&self, index: u64) -> u64 {
    //     self.merkle.get_sibling_index(index)
    // }

    pub fn get_path(&self, index: u64) -> Result<[u64; DEPTH], MerkleError> {
        self.leaf_check(index)?;
        let mut height = (index + 1).ilog2();
        let round = height;
        let full = (1u64 << height) - 1;
        let mut p = index - full;
        let mut path = vec![];
        for _ in 0..round {
            let full = (1u64 << height) - 1;
            // Calculate the index of current node
            let i = full + p;
            path.insert(0, i);
            height = height - 1;
            // Caculate the offset of parent
            p = p / 2;
        }
        assert!(p == 0);
        Ok(path.try_into().unwrap())
    }

    pub async fn update_leaf_data_with_proof(
        &mut self,
        index: u64,
        data: &Vec<u8>,
    ) -> Result<MerkleProof<[u8; 32], DEPTH>, MerkleError> {
        let (mut leaf, _) = self.get_leaf_with_proof(index).await?;
        leaf.set(data);
        self.set_leaf_with_proof(&leaf).await
    }

    // async fn restore_root_from_data(
    //     &mut self,
    //     data: Vec<u8>,
    //     index: u64,
    //     assists: [[u8; 32]; DEPTH],
    // ) -> [u8; 32] {
    //     let mut hash = self.cal_hash(&data).await;
    //     let mut p = get_offset(index);
    //     for i in 0..DEPTH {
    //         let cur_hash = hash;
    //         let depth = DEPTH - i - 1;
    //         let (left, right) = if p % 2 == 1 {
    //             (&assists[depth], &cur_hash)
    //         } else {
    //             (&cur_hash, &assists[depth])
    //         };
    //         hash = MongoMerkle::<DEPTH>::hash(left, right);
    //         p = p / 2;
    //     }
    //     hash
    // }
    // async fn cal_hash(&self, data: &[u8]) -> [u8; 32] {
    //     let mut hasher = POSEIDON_HASHER.clone();
    //     let batchdata = data
    //         .chunks(16)
    //         .into_iter()
    //         .map(|x| {
    //             let mut v = x.clone().to_vec();
    //             v.extend_from_slice(&[0u8; 16]);
    //             let f = v.try_into().unwrap();
    //             Fr::from_repr(f).unwrap()
    //         })
    //         .collect::<Vec<Fr>>();
    //     let values: [Fr; 2] = batchdata.try_into().unwrap();
    //     hasher.update(&values);
    //     let hash = hasher.squeeze().to_repr();
    //     hash
    // }

    // 1. 先根据data 算出leaf的hash
    // 2. 然后根据index 获取得到对应的proof
    // 3. 根据index 获取 p
    // 3. 获取得到proof 也就能拿到assist
    async fn set_leaf_with_proof(
        &mut self,
        leaf: &MerkleRecord,
    ) -> Result<MerkleProof<[u8; 32], DEPTH>, MerkleError> {
        let index = leaf.index();
        let mut hash = leaf.hash();
        let (_, mut proof) = self.get_leaf_with_proof(index).await?;
        proof.source = hash.clone();
        let mut p = get_offset(index);
        //self.set_leaf(leaf)?;
        let mut parents = vec![];
        for i in 0..DEPTH {
            let cur_hash = hash;
            let depth = DEPTH - i - 1;
            let (left, right) = if p % 2 == 1 {
                (&proof.assist[depth], &cur_hash)
            } else {
                (&cur_hash, &proof.assist[depth])
            };
            hash = default_merkle_hash(left, right);
            p = p / 2;
            let index = p + (1 << depth) - 1;
            //self.set_parent(index, &hash, left, right)?;
            parents.push((index, hash.clone(), left.clone(), right.clone()));
        }
        self.set_leaf_and_parents(leaf, parents.try_into().unwrap())
            .await?;
        self.update_root_hash(&hash);
        proof.root = hash;

        Ok(proof)
    }

    pub fn height() -> usize {
        return DEPTH;
    }
    fn empty_leaf(index: u64) -> MerkleRecord {
        let mut leaf = MerkleRecord::new(index);
        leaf.set(&[0; 32].to_vec());
        leaf
    }

    fn hash(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
        default_merkle_hash(a,b)
    }
}

pub fn get_offset(index: u64) -> u64 {
    let height = (index + 1).ilog2();
    let full = (1u64 << height) - 1;
    index - full
}