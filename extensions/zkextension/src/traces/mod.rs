use crate::store::trace::{TraceTable, WriteTable, WriteTrace};
use crate::utils::{fr_to_fq, u32_array_to_f};
use franklin_crypto::bellman::bn256::Bn256;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::pasta::Fq;
use merkle_tree::primitives::GetBits;
use merkle_tree::Fr;
use tree::tree::NullHasher;

pub struct TraceTableCircuit<F: PrimeField> {
    pub alloc: F,
    pub write: WriteTableCircuit<F>,
}

impl<F: PrimeField> GetBits for TraceTableCircuit<F> {
    fn get_bits_le(&self) -> Vec<bool> {
        todo!()
    }
}
impl<F: PrimeField> From<TraceTable<NullHasher>> for TraceTableCircuit<F> {
    fn from(value: TraceTable<NullHasher>) -> Self {
        let alloc = F::from(value.alloc as u64);
        TraceTableCircuit {
            alloc,
            write: value.write.into(),
        }
    }
}
pub struct WriteTableCircuit<F: PrimeField> {
    pub traces: Vec<WriteTraceCircuit<F>>,
}
impl<F: PrimeField> From<WriteTable<NullHasher>> for WriteTableCircuit<F> {
    fn from(value: WriteTable<NullHasher>) -> Self {
        let traces: Vec<WriteTraceCircuit<F>> = value
            .traces
            .into_iter()
            .map(|v| {
                let a = match v {
                    WriteTrace::Set(index, k, v) => {
                        let k_f = u32_array_to_f(&k);
                        let k_v = u32_array_to_f(&v);
                        WriteTraceCircuit::Set(index, k_f, k_v)
                    }
                    WriteTrace::Delete(index, k) => {
                        let k_f = u32_array_to_f(&k);
                        WriteTraceCircuit::Delete(index, k_f)
                    }
                };
                a
            })
            .collect();
        WriteTableCircuit { traces }
    }
}

#[derive(Clone)]
pub enum WriteTraceCircuit<F: PrimeField> {
    Set(u32, F, F),
    Delete(u32, F),
}

#[test]
pub fn tesae() {}
