use crate::store::trace::{TraceTable, WriteTable, WriteTrace};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::pasta::Fq;
use merkle_tree::primitives::GetBits;
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
impl<F:PrimeField>From<TraceTable<NullHasher>> for TraceTableCircuit<F>{
    fn from(value: TraceTable<NullHasher>) -> Self {
        let alloc=F::from(value.alloc as u64);
    }
}
pub struct WriteTableCircuit<F: PrimeField> {
    pub traces: Vec<WriteTraceCircuit<F>>,
}
impl<F:PrimeField> From<WriteTable<NullHasher>> for WriteTableCircuit<F>{
    fn from(value: WriteTable<NullHasher>) -> Self {
        value.traces
            .into_iter()
            .map(|v|{
                let a=match v {
                    WriteTrace::Set(index, k, v) => {
                        WriteTraceCircuit::Set(index,F::)
                    }
                    WriteTrace::Delete(index, k) => {}
                };
                a
            }).collect();
    }
}

#[derive(Clone)]
pub enum WriteTraceCircuit<F: PrimeField> {
    Set(u32, F, F),
    Delete(u32, F),
}

