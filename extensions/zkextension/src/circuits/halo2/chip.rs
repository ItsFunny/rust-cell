use crate::circuits::halo2::merkle::{MerkleChip, MerkleConfig};
use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};
use std::marker::PhantomData;

pub struct MerkleCircuit<F: PrimeField> {
    _ph: PhantomData<F>,
    index: u64,
    audit_path: Vec<F>,
    leaf: Vec<bool>,
}

impl<F: PrimeField> MerkleCircuit<F> {
    pub fn new(index: u64, audit_path: Vec<F>, leaf: Vec<bool>) -> Self {
        Self {
            _ph: Default::default(),
            index,
            audit_path,
            leaf,
        }
    }
}

impl<F: PrimeField> Circuit<F> for MerkleCircuit<F> {
    type Config = MerkleConfig<64>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        unreachable!()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        MerkleChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let leaf = self.leaf.clone();
        let index = self.index;
        let merkle_paths = self
            .audit_path
            .clone()
            .into_iter()
            .map(|v| Value::known(v))
            .collect();
        let chip: MerkleChip<F, 64> = MerkleChip::new(config);
        chip.assign(
            layouter.namespace(|| "assign "),
            Some(leaf),
            Value::known(F::from(index)),
            &merkle_paths,
        )
    }
}
