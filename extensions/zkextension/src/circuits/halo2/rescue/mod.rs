pub mod mimc;

use crate::circuits::halo2::wrapper::CellWrapper;
use crate::utils::{fq_to_fr, fr_to_fq};
use franklin_crypto::bellman::bn256::{Bn256, Fr};
use franklin_crypto::rescue::bn256::Bn256RescueParams;
use halo2_proofs::circuit::{Layouter, Region, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error};
use std::marker::PhantomData;

#[derive(Clone)]
pub struct RescueHashConfig {
    // lhs: Column<Advice>,
    // rhs: Column<Advice>,
    cur: Column<Advice>,
}
pub struct RescueHashChip<F: PrimeField> {
    config: RescueHashConfig,
    _ph: PhantomData<F>,
}

impl<F: PrimeField> RescueHashChip<F> {
    pub fn configure(meta: &mut ConstraintSystem<F>) -> RescueHashConfig {
        // FIXME ,need to implement the rescue chip
        // let lhs = meta.advice_column();
        // let rhs = meta.advice_column();
        // let cur = meta.advice_column();
        // RescueHashConfig { lhs, rhs, cur }

        let cur = meta.advice_column();
        RescueHashConfig { cur }
    }
    pub fn construct(config: RescueHashConfig) -> Self {
        Self {
            config,
            _ph: Default::default(),
        }
    }
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        traces: Option<Vec<bool>>,
    ) -> Result<CellWrapper<F>, Error> {
        layout.assign_region(
            || "assign layout",
            |mut region| {
                let witness = self.pack_into_witness(&mut region, traces.clone().unwrap());
                let hash = self.rescue_hash(&mut region, witness);
                let ret = region.assign_advice(
                    || "assign hash",
                    self.config.cur,
                    0,
                    || Value::known(hash),
                )?;
                Ok(CellWrapper::new(ret))
            },
        )
    }
    pub fn assign_rescue(
        &self,
        mut layout: impl Layouter<F>,
        lhs_value: Value<F>,
        rhs_value: Value<F>,
    ) -> CellWrapper<F> {
        let mut leaf = vec![];
        lhs_value.map(|v| leaf.push(v));
        rhs_value.map(|v| leaf.push(v));
        let ret = layout
            .assign_region(
                || "assign rescue",
                |mut region| {
                    let hash = self.rescue_hash(&mut region, leaf.clone());
                    let ret = region.assign_advice(
                        || "assign hash",
                        self.config.cur,
                        0,
                        || Value::known(hash),
                    )?;
                    Ok(CellWrapper::new(ret))
                },
            )
            .unwrap();
        ret
    }

    // FIXME: return assigned cell
    pub(crate) fn rescue_hash(&self, region: &mut Region<F>, leaf: Vec<F>) -> F {
        // TODO ,add constraints
        let RESCUE_PARAMS: Bn256RescueParams = Bn256RescueParams::new_checked_2_into_1();
        let leaf_data: Vec<Fr> = leaf.into_iter().map(|v| fq_to_fr(&v)).collect();
        let hash: Vec<Fr> =
            franklin_crypto::rescue::rescue_hash::<Bn256>(&RESCUE_PARAMS, leaf_data.as_slice());
        let hash: Vec<F> = hash.into_iter().map(|v| fr_to_fq(&v)).collect();
        let hash = hash.get(0).unwrap().clone();
        hash
    }
    // FIXME: return operation
    fn pack_into_witness(&self, region: &mut Region<F>, bits: Vec<bool>) -> Vec<F> {
        let mut results: Vec<F> = vec![];

        // TODO ,size
        for (_, bits) in bits.chunks(253 as usize).enumerate() {
            let mut coeff = F::ONE;
            let mut value = Some(F::ZERO);
            for bit in bits {
                let newval: Option<F> = match (value, Some(bit.clone())) {
                    (Some(mut curval), Some(bval)) => {
                        if bval {
                            curval.add_assign(&coeff);
                        }

                        Some(curval)
                    }
                    _ => None,
                };
                value = newval;

                coeff = coeff.double();
            }
            // TODO, add constraint
            let fr: F = value.unwrap();
            results.push(fr);
        }

        results
    }
}
