use crate::store::trace::TraceTable;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::pasta::Fq;
use tree::tree::NullHasher;

pub struct TraceTableCircuit<F: Field> {
    pub alloc: F,
    pub write: WriteTableCircuit<F>,
}
pub struct WriteTableCircuit<F: Field> {
    pub traces: Vec<WriteTrace<F>>,
}

#[derive(Clone)]
pub enum WriteTrace<F: Field> {
    Set(u32, F, F),
    Delete(u32, F),
}

impl From<TraceTable<NullHasher>> for TraceTableCircuit<Fq> {
    fn from(value: TraceTable<NullHasher>) -> Self {
        todo!()
    }
}
