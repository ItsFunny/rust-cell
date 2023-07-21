use crate::circuits::halo2::wrapper::CellWrapper;
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

pub struct ReverseConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub s: Selector,
}
pub struct ReverseChip<F: PrimeField> {
    config: ReverseConfig,
    _ph: PhantomData<F>,
}

impl<F: PrimeField> ReverseChip<F> {
    pub fn new(config: ReverseConfig) -> Self {
        Self {
            config,
            _ph: Default::default(),
        }
    }

    // let r=b if s=0 {b} else {a}
    // poly: r-b=s*(a-b)
    // let l=a if s=0 {a} else {b}
    // poly: l-a=s*(a-b)
    pub fn configure(meta: &mut ConstraintSystem<F>) -> ReverseConfig {
        let a_col = meta.advice_column();
        let b_col = meta.advice_column();
        let s = meta.selector();

        meta.create_gate("reverse", |meta| {
            let a = meta.query_advice(a_col, Rotation::cur());
            let b = meta.query_advice(b_col, Rotation::cur());
            let s = meta.query_selector(s);

            let l = meta.query_advice(a_col, Rotation::next());
            let r = meta.query_advice(b_col, Rotation::next());

            vec![
                s.clone() * (a.clone() - b.clone()) - (r.clone() - b.clone())
                    + s.clone() * (a.clone() - b.clone())
                    - (l.clone() - a.clone()),
            ]
        });

        ReverseConfig {
            a: a_col,
            b: b_col,
            s,
        }
    }
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        index: Option<bool>,
        a: Value<F>,
        b: Value<F>,
        offset: usize,
    ) -> Result<(CellWrapper<F>, CellWrapper<F>), Error> {
        layout.assign_region(
            || "reverse assign",
            |mut region| {
                let index = index.unwrap();
                if index {
                    self.config.s.enable(&mut region, offset)?;
                }
                let l: AssignedCell<F, F> =
                    region.assign_advice(|| "lhs", self.config.a, 0, || a)?;
                let r = region.assign_advice(|| "rhs", self.config.b, 0, || b)?;

                Ok((CellWrapper { cell: l }, CellWrapper { cell: r }))
            },
        )
    }
}
#[test]
pub fn test_asd() {}
