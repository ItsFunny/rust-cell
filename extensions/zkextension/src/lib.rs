pub mod blocks;
pub mod circuits;
pub mod db;
pub mod instance;
pub mod store;
pub mod suite;
pub mod types;

use crate::instance::merkle::InstanceMerkleNode;
use merkle_tree::smt::rescue_hasher::RescueHasher;
use merkle_tree::{Engine, Fr, SparseMerkleTree};

pub type InstanceTree = SparseMerkleTree<InstanceMerkleNode, Fr, RescueHasher<Engine>>;
