use keccak_hasher::KeccakHasher;
use std::collections::HashMap;
use trie_db::{DBValue, TrieHash};

#[derive(Default)]
pub struct SimpleProveRequest {
    pub query: Vec<Vec<u8>>,
}

#[derive(Default)]
pub struct MPTProveRequest {
    pub root: TrieHash<sp_trie::LayoutV0<KeccakHasher>>,
    pub query: Vec<Vec<u8>>,
}
pub enum ProveRequestEnums {
    SimpleProve(SimpleProveRequest),
    MPTProve(MPTProveRequest),
}

pub enum ProveResponseEnums {
    SimpleProve(SimpleProveResponse),
    MPTProve(MPTProveResponse),
}

impl SimpleProveRequest {
    pub fn insert(&mut self, k: Vec<u8>) {
        self.query.push(k)
    }
}

pub struct SimpleProveResponse {
    pub proof: Vec<u8>,
}
pub struct MPTProveResponse {
    pub proof: Vec<Vec<u8>>,
}

impl MPTProveResponse {
    pub fn new(proof: Vec<Vec<u8>>) -> Self {
        Self { proof }
    }
}
pub enum VerifyRequestEnums {
    SimpleVerify(SimpleVerifyRequest),
    MPTVerify(MPTVerifyRequest),
}

pub struct SimpleVerifyRequest {
    pub proof: Vec<u8>,
    pub expected_root: [u8; 32],
    pub kv: HashMap<Vec<u8>, Vec<u8>>,
}

pub struct MPTVerifyRequest {
    pub proof: Vec<Vec<u8>>,
    pub root: TrieHash<sp_trie::LayoutV0<KeccakHasher>>,
    pub query: Vec<(Vec<u8>, Option<DBValue>)>,
}

impl SimpleVerifyRequest {
    pub fn new(proof: Vec<u8>, expected_root: [u8; 32]) -> Self {
        Self {
            proof,
            expected_root,
            kv: Default::default(),
        }
    }

    pub fn insert(&mut self, k: Vec<u8>, v: Vec<u8>) {
        self.kv.insert(k, v);
    }
}

#[derive(Default)]
pub struct VerifyResponse {
    pub valid: bool,
}
