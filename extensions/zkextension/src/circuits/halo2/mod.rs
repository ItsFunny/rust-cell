mod chip;
mod chip2;
mod hasher;
mod mimc;
pub mod rescue_chip;

use crate::circuits::halo2::chip::{MerkleChip, MerkleChipTrait};
use crate::traces::TraceTableCircuit;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{Layouter, Region, SimpleFloorPlanner};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::pasta::Fq;
use halo2_proofs::plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Fixed, Instance};
use halo2_proofs::poly::Rotation;

#[derive(Clone, Debug)]
pub struct BlockHalo2Config {
    old_root: Column<Fixed>,
    traces: [Column<Advice>; 10],
    new_root: Column<Instance>,
}
pub struct BlockHalo2Circuit<F: PrimeField> {
    trace_table: TraceTableCircuit<F>,
}

impl<F: PrimeField> Circuit<F> for BlockHalo2Circuit<F> {
    type Config = BlockHalo2Config;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        todo!()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let traces = [meta.advice_column(); 10];
        let old_root = meta.fixed_column();
        let new_root = meta.instance_column();
        meta.enable_equality(new_root);
        meta.enable_equality(old_root);
        for trace in traces {
            meta.enable_equality(trace);
        }

        meta.create_gate("merkle root", |meta| {
            let ret = meta.query_advice(traces[0], Rotation::cur());
            // | trace_0  | trace_1  | trace_2 |trace_3 |trace_4 |trace_5 |trace_6 |trace_7 |trace_8 |trace_9 |
            // |-----|-----|-------|-------|-------|-------|-------|-------|-------|-------|-------|-------|
            // | lhs |.....
            // | out |     |       |
            for i in 0..10 {
                let advice = meta.query_advice(traces[i], Rotation::cur());
            }
            let out = meta.query_advice(traces[0], Rotation::next());

            vec![ret]
        });
        BlockHalo2Config {
            old_root,
            traces,
            new_root,
        }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        // let chip = MerkleChip::<F>::new(config.clone());
        // let root = chip.allocate_merkle_root(layouter.namespace(|| "roots"), &self.trace_table);
        // layouter
        //     .constrain_instance(root.cell(), config.new_root.clone(), 0)
        //     .unwrap();
        // Ok(())
        todo!()
    }
}
