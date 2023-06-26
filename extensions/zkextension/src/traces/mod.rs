use crate::store::trace::{TraceTable, WriteTable, WriteTrace};
use crate::utils::{f_get_bits, fr_to_fq, u32_array_to_f};
use franklin_crypto::bellman::bn256::Bn256;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::pasta::Fq;
use merkle_tree::primitives::GetBits;
use merkle_tree::{params, Fr};
use tree::tree::NullHasher;

pub struct TraceTableCircuit<F: PrimeField> {
    pub alloc: F,
    pub write: WriteTableCircuit<F>,
}

impl<F: PrimeField> GetBits for TraceTableCircuit<F> {
    fn get_bits_le(&self) -> Vec<bool> {
        let mut ret = vec![];
        ret.extend(f_get_bits(&self.alloc, params::FR_BIT_WIDTH));
        for trace in &self.write.traces {
            match trace {
                WriteTraceCircuit::Set(index, k, v) => {
                    ret.extend(f_get_bits(index, params::FR_BIT_WIDTH));
                    ret.extend(f_get_bits(k, params::FR_BIT_WIDTH));
                    ret.extend(f_get_bits(v, params::FR_BIT_WIDTH));
                }
                WriteTraceCircuit::Delete(index, k) => {
                    ret.extend(f_get_bits(index, params::FR_BIT_WIDTH));
                    ret.extend(f_get_bits(k, params::FR_BIT_WIDTH));
                }
            }
        }
        ret
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
                        WriteTraceCircuit::Set(F::from(index as u64), k_f, k_v)
                    }
                    WriteTrace::Delete(index, k) => {
                        let k_f = u32_array_to_f(&k);
                        WriteTraceCircuit::Delete(F::from(index as u64), k_f)
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
    Set(F, F, F),
    Delete(F, F),
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn tesae() {}
}
