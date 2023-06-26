use crate::store::trace::{TraceTable, WriteTrace};
use merkle_tree::primitives::GetBits;
use std::mem::take;
use tree::tree::NullHasher;

pub struct InstanceMerkleNode {
    pub table: TraceTable<NullHasher>,
}

impl GetBits for InstanceMerkleNode {
    fn get_bits_le(&self) -> Vec<bool> {
        let mut ret = vec![];
        let write = &self.table.write;
        for trace in &write.traces {
            match trace {
                WriteTrace::Set(index, k, v) => {
                    ret.extend(u32_to_bool_vec(*index));
                    ret.extend(u8_array_to_bool_vec(k));
                    ret.extend(u8_array_to_bool_vec(v));
                }
                WriteTrace::Delete(index, k) => {
                    ret.extend(u32_to_bool_vec(*index));
                    ret.extend(u8_array_to_bool_vec(k));
                }
            }
        }
        ret
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
