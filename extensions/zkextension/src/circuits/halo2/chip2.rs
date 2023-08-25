use crate::circuits::halo2::wrapper::CellWrapper;
use crate::utils::{fq_to_fr, fr_to_fq};
use franklin_crypto::bellman::bn256::{Bn256, Fr};
use franklin_crypto::rescue::bn256::Bn256RescueParams;
use halo2_proofs::circuit::{AssignedCell, Layouter, Region, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Expression, Instance};
use halo2_proofs::poly::Rotation;
use merkle_tree::primitives::BitIteratorLe;
use std::marker::PhantomData;

// lhs|rhs|cur_root|new_root|flag|s_flag
pub struct MerkleConfig {
    hash: Column<Advice>,
    lhs: Column<Advice>,
    rhs: Column<Advice>,
    cur_root: Column<Advice>,

    new_root: Column<Instance>,

    rescue_config: RescueHashConfig,
}

pub struct MerkleChip<F: PrimeField> {
    config: MerkleConfig,
    paths: Option<Vec<F>>,
}

impl<F: PrimeField> MerkleChip<F> {
    pub fn new(config: MerkleConfig) -> Self {
        Self {
            config,
            paths: None,
        }
    }

    // let l=a if flag==0 {a} else {b}
    // ploy: l-a=flag * (a-b)
    // let r=b if flag==0 {b} else {a}
    // poly: r-b=flag*(a-b)
    // flag*(a-b) - (l-a)+ flag*(a-b)-(r-b)
    // pub fn configure(meta: &mut ConstraintSystem<F>) -> MerkleConfig {
    //     let lhs = meta.advice_column();
    //     let rhs = meta.advice_column();
    //     let flag = meta.advice_column();
    //     let cur_root = meta.advice_column();
    //     let new_root = meta.instance_column();
    //     let hash = meta.advice_column();
    //
    //     let s_flag = meta.selector();
    //
    //     // 约束 flag 必须为1
    //     meta.create_gate("const constraint", |meta| {
    //         let flag = meta.query_advice(flag, Rotation::cur());
    //         let s_flag = meta.query_selector(s_flag);
    //         vec![s_flag * (flag - Expression::Constant(F::ONE))]
    //     });
    //
    //     meta.create_gate("reverse", |meta| {
    //         let a = meta.query_advice(lhs, Rotation::cur());
    //         let b = meta.query_advice(rhs, Rotation::cur());
    //
    //         let flag = meta.query_advice(flag, Rotation::cur());
    //         let left = meta.query_advice(lhs, Rotation::next());
    //         let right = meta.query_advice(rhs, Rotation::next());
    //
    //         vec![
    //             flag.clone() * (a.clone() - b.clone()) - (left.clone() - a.clone())
    //                 + flag.clone() * (a.clone() - b.clone())
    //                 - (right - b.clone()),
    //         ]
    //     });
    //
    //     let rescue_config = RescueHashChip::configure(meta, lhs, rhs);
    //
    //     MerkleConfig {
    //         hash,
    //         lhs,
    //         rhs,
    //         cur_root,
    //         new_root,
    //         rescue_config,
    //     }
    // }
    // pub fn assign(
    //     &self,
    //     mut layout: impl Layouter<F>,
    //     leaf: Option<Vec<bool>>,
    //     index: Option<F>,
    //     audit_path: &Vec<F>,
    // ) -> Result<(), Error> {
    //     // cur hash
    //     let first_leaf = self.assign_fist_leaf(layout.namespace(|| "assign first leaf"), leaf)?;
    //
    //     let index = self.index_to_bools(layout.namespace(|| "get index"), index);
    //
    //     for (i, direction_bit) in index.into_iter().enumerate() {
    //         let path_element = &audit_path[i];
    //         // Swap the two if the current subtree is on the right
    //     }
    //     // 计算新的root 根
    //     let mut cur_root = Expression::Constant(F::ZERO);
    //     // let hash_chip = RescueHashChip::construct(self.config.rescue_config.clone());
    //     // hash_chip.assign(layout.namespace(|| "assign hash"), lhs, rhs);
    //     // layout
    //     //     .assign_region(|| "assign merkle", |mut region| Ok(()))
    //     //     .expect("TODO: panic message");
    //     todo!()
    // }
    // pub fn assign_fist_leaf(
    //     &self,
    //     mut layout: impl Layouter<F>,
    //     leaf: Option<Vec<bool>>,
    // ) -> Result<CellWrapper<F>, Error> {
    //     let hash_chip = RescueHashChip::construct(self.config.rescue_config.clone());
    //     let hash = hash_chip.assign(layout.namespace(|| "assign hash"), leaf)?;
    //
    //     Ok(hash)
    // }

    // TODO: add constraints
    // fn index_to_bools(&self, layout: impl Layouter<F>, index: Option<F>) -> Vec<Option<bool>> {
    //     let values: Vec<Option<bool>> = match index {
    //         Some(ref value) => {
    //             let mut field_char = BitIteratorLe::new(F::MODULUS);
    //
    //             let mut tmp = Vec::with_capacity(F::NUM_BITS as usize);
    //
    //             let mut found_one = false;
    //             for b in BitIteratorLe::new(value.to_repr()) {
    //                 // Skip leading bits
    //                 found_one |= field_char.next().unwrap();
    //                 if !found_one {
    //                     continue;
    //                 }
    //
    //                 tmp.push(Some(b));
    //             }
    //
    //             assert_eq!(tmp.len(), F::NUM_BITS as usize);
    //
    //             tmp
    //         }
    //         None => vec![None; F::NUM_BITS as usize],
    //     };
    //     values
    // }
}

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
        lhs: AssignedCell<F, F>,
        rhs: AssignedCell<F, F>,
    ) -> CellWrapper<F> {
        let lhs_value = lhs.value().cloned();
        let rhs_value = rhs.value().cloned();
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

#[test]
pub fn test_asd() {}
