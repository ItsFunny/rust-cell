use crate::store::trace::{TraceTable, WriteTrace};
use crate::traces::TraceTableCircuit;
use halo2_proofs::pasta::Fq;
use merkle_tree::primitives::GetBits;
use tree::tree::NullHasher;

#[derive(Default, Clone)]
pub struct InstanceMerkleNode {
    pub table: TraceTable<NullHasher>,
}

impl GetBits for InstanceMerkleNode {
    fn get_bits_le(&self) -> Vec<bool> {
        let table: TraceTableCircuit<Fq> = self.table.clone().into();
        table.get_bits_le()
    }
}

pub fn u8_array_to_bool_vec(array: &[u8; 32]) -> Vec<bool> {
    let mut bool_vec = Vec::new();

    for &byte in array.iter() {
        for i in (0..8).rev() {
            bool_vec.push((byte >> i) & 1 == 1);
        }
    }

    bool_vec
}

pub fn u32_to_bool_vec(num: u32) -> Vec<bool> {
    let mut bool_vec = Vec::new();

    for i in (0..32).rev() {
        bool_vec.push((num >> i) & 1 == 1);
    }

    bool_vec
}
