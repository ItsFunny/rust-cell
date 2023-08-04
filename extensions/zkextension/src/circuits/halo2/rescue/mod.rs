use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RescueHashConfig {
    lhs: Column<Advice>,
    rhs: Column<Advice>,
}
pub struct RescueHashChip<F: PrimeField> {
    config: RescueHashConfig,
    _ph: PhantomData<F>,
}

impl<F: PrimeField> RescueHashChip<F> {
    pub fn configure(meta: &mut ConstraintSystem<F>) -> RescueHashConfig {
        let lhs = meta.advice_column();
        let rhs = meta.advice_column();
        RescueHashConfig { lhs, rhs }
    }
}
