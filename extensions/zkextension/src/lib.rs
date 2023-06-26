extern crate core;

pub mod blocks;
pub mod circuits;
pub mod db;
pub mod instance;
pub mod store;
pub mod suite;
mod traces;
pub mod types;
pub mod utils;

use crate::instance::merkle::InstanceMerkleNode;
use merkle_tree::smt::rescue_hasher::RescueHasher;
use merkle_tree::{Engine, Fr, SparseMerkleTree};

pub type InstanceTree = SparseMerkleTree<InstanceMerkleNode, Fr, RescueHasher<Engine>>;

pub type CircuitInstanceTree = SparseMerkleTree<InstanceMerkleNode, Fr, RescueHasher<Engine>>;
