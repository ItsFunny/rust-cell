use crate::circuits::halo2::BlockHalo2Config;
use crate::circuits::traces::TraceTableCircuit;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Chip, Layouter};
use std::marker::PhantomData;

pub struct MerkleChip<F: Field> {
    config: BlockHalo2Config,
    _marker: PhantomData<F>,
}

impl<F: Field> MerkleChip<F> {
    pub fn new(config: BlockHalo2Config) -> Self {
        Self {
            config,
            _marker: Default::default(),
        }
    }
}

pub trait MerkleChipTrait<F: Field>: Chip<F> {
    fn allocate_merkle_root(
        &self,
        layouter: impl Layouter<F>,
        traces: &TraceTableCircuit<F>,
    ) -> AssignedCell<F, F>;
}

impl<F: Field> Chip<F> for MerkleChip<F> {
    type Config = BlockHalo2Config;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: Field> MerkleChipTrait<F> for MerkleChip<F> {
    fn allocate_merkle_root(
        &self,
        layouter: impl Layouter<F>,
        traces: &TraceTableCircuit<F>,
    ) -> AssignedCell<F, F> {
        todo!()
    }
}

fn do_allocate_merkle_root(leaf_bits: Vec<bool>) {
    
}
