pub mod circuits;
pub mod types;

use crate::types::instance::Instance;
use merkle_tree::smt::rescue_hasher::RescueHasher;
use merkle_tree::{Engine, Fr, SparseMerkleTree};

pub type InstanceTree = SparseMerkleTree<Instance, Fr, RescueHasher<Engine>>;
