mod db;
mod kv;
mod memory;
mod merkle;
mod error;

use std::fmt::Debug;
use mongodb::bson::Bson;
use mongodb::bson::de::Error as MongoError;
use mongodb::bson::spec::BinarySubtype;
use pairing::bn256::Fr;
use pairing::group::ff::PrimeField;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{Error, Unexpected};
pub use db::*;
pub use kv::*;
pub use memory::*;
pub use merkle::*;
use poseidon::Poseidon;
use poseidon::Spec;

pub const DEFAULT_ROOT_HASH: [u8; 32] = [
    88, 20, 228, 30, 38, 148, 94, 78, 241, 191, 176, 207, 85, 243, 219, 203, 191, 117, 59, 232, 19,
    48, 22, 59, 100, 89, 52, 250, 124, 119, 152, 40,
];

pub const DEFAULT_ROOT_HASH64: [u64; 4] = [
    5647113874217112664,
    14689602159481241585,
    4257643359784105407,
    2925219336634521956,
];


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DataHashRecord {
    #[serde(serialize_with = "self::serialize_bytes_as_binary")]
    #[serde(deserialize_with = "self::deserialize_u256_from_binary")]
    pub hash: [u8; 32],
    #[serde(serialize_with = "self::serialize_bytes_as_binary")]
    #[serde(deserialize_with = "self::deserialize_bytes_from_binary")]
    pub data: Vec<u8>,
}

lazy_static::lazy_static! {
    pub static ref POSEIDON_HASHER: poseidon::Poseidon<Fr, 9, 8> = Poseidon::<Fr, 9, 8>::new(8, 63);
    pub static ref MERKLE_HASHER: poseidon::Poseidon<Fr, 3, 2> = Poseidon::<Fr, 3, 2>::new(8, 57);
    pub static ref POSEIDON_HASHER_SPEC: poseidon::Spec<Fr, 9, 8> = Spec::new(8, 63);
    pub static ref MERKLE_HASHER_SPEC: poseidon::Spec<Fr, 3, 2> = Spec::new(8, 57);
}

pub(crate) fn default_merkle_hash(a: &[u8; 32], b: &[u8; 32]) -> [u8; 32] {
    let mut hasher = MERKLE_HASHER.clone();
    let a = Fr::from_repr(*a).unwrap();
    let b = Fr::from_repr(*b).unwrap();
    hasher.update_exact(&[a, b]).to_repr()
}

pub(crate) fn default_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = POSEIDON_HASHER.clone();
    let batchdata = data
        .chunks(16)
        .into_iter()
        .map(|x| {
            let mut v = x.clone().to_vec();
            v.extend_from_slice(&[0u8; 16]);
            let f = v.try_into().unwrap();
            Fr::from_repr(f).unwrap()
        })
        .collect::<Vec<Fr>>();
    hasher.update(&batchdata.as_slice());
    hasher.squeeze().to_repr()
}

impl DataHashRecord {
    pub fn new(&mut self, data: &Vec<u8>) -> Self {
        let hash = default_hash(data.as_slice());
        DataHashRecord {
            data: data.clone().try_into().unwrap(),
            hash,
        }
    }
    pub fn data_as_u64(&self) -> [u64; 4] {
        [
            u64::from_le_bytes(self.data[0..8].try_into().unwrap()),
            u64::from_le_bytes(self.data[8..16].try_into().unwrap()),
            u64::from_le_bytes(self.data[16..24].try_into().unwrap()),
            u64::from_le_bytes(self.data[24..32].try_into().unwrap()),
        ]
    }
}


#[derive(Debug, Serialize, Deserialize, Clone,PartialEq)]
pub struct MerkleRecord {
    #[serde(serialize_with = "self::serialize_u64_as_binary")]
    #[serde(deserialize_with = "self::deserialize_u64_as_binary")]
    pub index: u64,
    #[serde(serialize_with = "self::serialize_bytes_as_binary")]
    #[serde(deserialize_with = "self::deserialize_u256_as_binary")]
    pub hash: [u8; 32],
    #[serde(serialize_with = "self::serialize_bytes_as_binary")]
    #[serde(deserialize_with = "self::deserialize_u256_as_binary")]
    pub left: [u8; 32],
    #[serde(serialize_with = "self::serialize_bytes_as_binary")]
    #[serde(deserialize_with = "self::deserialize_u256_as_binary")]
    pub right: [u8; 32],
    #[serde(serialize_with = "self::serialize_bytes_as_binary")]
    #[serde(deserialize_with = "self::deserialize_u256_as_binary")]
    pub data: [u8; 32],
}

impl MerkleRecord {
    fn index(&self) -> u64 {
        self.index
    }
    fn hash(&self) -> [u8; 32] {
        self.hash
    }
    fn set(&mut self, data: &Vec<u8>) {
        let mut hasher = POSEIDON_HASHER.clone();
        self.data = data.clone().try_into().unwrap();
        let batchdata = data
            .chunks(16)
            .into_iter()
            .map(|x| {
                let mut v = x.clone().to_vec();
                v.extend_from_slice(&[0u8; 16]);
                let f = v.try_into().unwrap();
                Fr::from_repr(f).unwrap()
            })
            .collect::<Vec<Fr>>();
        let values: [Fr; 2] = batchdata.try_into().unwrap();
        hasher.update(&values);
        self.hash = hasher.squeeze().to_repr();
    }
    fn right(&self) -> Option<[u8; 32]> {
        Some(self.right)
    }
    fn left(&self) -> Option<[u8; 32]> {
        Some(self.left)
    }
}

impl MerkleRecord {
    fn new(index: u64) -> Self {
        MerkleRecord {
            index,
            hash: [0; 32],
            data: [0; 32],
            left: [0; 32],
            right: [0; 32],
        }
    }

    pub fn data_as_u64(&self) -> [u64; 4] {
        [
            u64::from_le_bytes(self.data[0..8].try_into().unwrap()),
            u64::from_le_bytes(self.data[8..16].try_into().unwrap()),
            u64::from_le_bytes(self.data[16..24].try_into().unwrap()),
            u64::from_le_bytes(self.data[24..32].try_into().unwrap()),
        ]
    }
}

#[derive(Debug)]
pub struct MerkleProof<H: Debug + Clone + PartialEq, const D: usize> {
    pub source: H,
    pub root: H,
    // last is root
    pub assist: [H; D],
    pub index: u64,
}

fn deserialize_u64_as_binary<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: Deserializer<'de>,
{
    match Bson::deserialize(deserializer) {
        Ok(Bson::Binary(bytes)) => Ok({
            let c: [u8; 8] = bytes.bytes.try_into().unwrap();
            u64::from_le_bytes(c)
        }),
        Ok(..) => Err(Error::invalid_value(Unexpected::Enum, &"Bson::Binary")),
        Err(e) => Err(e),
    }
}

fn serialize_bytes_as_binary<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let binary = Bson::Binary(mongodb::bson::Binary {
        subtype: BinarySubtype::Generic,
        bytes: bytes.into(),
    });
    binary.serialize(serializer)
}

fn deserialize_u256_from_binary<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
{
    match Bson::deserialize(deserializer) {
        Ok(Bson::Binary(bytes)) => Ok(bytes.bytes.try_into().unwrap()),
        Ok(..) => Err(Error::invalid_value(Unexpected::Enum, &"Bson::Binary")),
        Err(e) => Err(e),
    }
}

fn deserialize_bytes_from_binary<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
{
    match Bson::deserialize(deserializer) {
        Ok(Bson::Binary(bytes)) => Ok(bytes.bytes.to_vec()),
        Ok(..) => Err(Error::invalid_value(Unexpected::Enum, &"Bson::Binary")),
        Err(e) => Err(e),
    }
}

fn serialize_u64_as_binary<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
{
    let binary = Bson::Binary(mongodb::bson::Binary {
        subtype: BinarySubtype::Generic,
        bytes: value.to_le_bytes().to_vec(),
    });
    binary.serialize(serializer)
}

fn deserialize_u256_as_binary<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
{
    match Bson::deserialize(deserializer) {
        Ok(Bson::Binary(bytes)) => Ok(bytes.bytes.try_into().unwrap()),
        Ok(..) => Err(Error::invalid_value(Unexpected::Enum, &"Bson::Binary")),
        Err(e) => Err(e),
    }
}