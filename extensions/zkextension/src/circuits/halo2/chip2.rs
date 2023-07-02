use halo2_proofs::arithmetic::Field;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Instance};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

// hash|lhs|rhs|root_cur
pub struct MerkleConfig {
    hash: Column<Advice>,
    lhs: Column<Advice>,
    rhs: Column<Advice>,
    cur_root: Column<Advice>,

    new_root: Column<Instance>,
}

pub struct MerkleChip<F: Field> {
    config: MerkleConfig,
    _ph: PhantomData<F>,
}

impl<F: Field> MerkleChip<F> {
    pub fn new(config: MerkleConfig) -> Self {
        Self {
            config,
            _ph: Default::default(),
        }
    }

    // let l=a if flag==0 {a} else {b}
    // ploy: l-a=flag * (a-b)
    // let r=b if flag==0 {b} else {a}
    // poly: r-b=flag*(a-b)
    // flag*(a-b) - (l-a)+ flag*(a-b)-(r-b)
    pub fn configure(meta: &mut ConstraintSystem<F>) -> MerkleConfig {
        let lhs = meta.advice_column();
        let rhs = meta.advice_column();
        let flag = meta.advice_column();
        let cur_root = meta.advice_column();
        let new_root = meta.instance_column();
        let hash = meta.advice_column();
        meta.create_gate("reverse", |meta| {
            let a = meta.query_advice(lhs, Rotation::cur());
            let b = meta.query_advice(rhs, Rotation::cur());
            let flag = meta.query_advice(flag, Rotation::cur());
            let left = meta.query_advice(lhs, Rotation::next());
            let right = meta.query_advice(rhs, Rotation::next());

            vec![
                flag.clone() * (a.clone() - b.clone()) - (left.clone() - a.clone())
                    + flag.clone() * (a.clone() - b.clone())
                    - (right - b.clone()),
            ]
        });
        MerkleConfig {
            hash,
            lhs,
            rhs,
            cur_root,
            new_root,
        }
    }
}
