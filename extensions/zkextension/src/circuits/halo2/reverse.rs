use crate::circuits::halo2::wrapper::CellWrapper;
use halo2_proofs::circuit::{AssignedCell, Layouter, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Selector};
use halo2_proofs::poly::Rotation;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct ReverseConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub bit: Column<Advice>,
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
    // poly: r-b=bit*(a-b)
    // let l=a if s=0 {a} else {b}
    // poly: l-a=bit*(a-b)
    pub fn configure(meta: &mut ConstraintSystem<F>) -> ReverseConfig {
        let a_col = meta.advice_column();
        let b_col = meta.advice_column();
        let bit_col = meta.advice_column();
        let s = meta.selector();

        meta.create_gate("reverse", |meta| {
            let a = meta.query_advice(a_col, Rotation::cur());
            let b = meta.query_advice(b_col, Rotation::cur());
            let bit = meta.query_advice(bit_col, Rotation::cur());
            let s = meta.query_selector(s);

            let l = meta.query_advice(a_col, Rotation::next());
            let r = meta.query_advice(b_col, Rotation::next());

            vec![
                s.clone()
                    * (r.clone() - b.clone() - bit.clone() * (a.clone() - b.clone()) + l.clone()
                        - a.clone()
                        - bit.clone() * (a.clone() - b.clone())),
            ]
        });

        ReverseConfig {
            a: a_col,
            b: b_col,
            bit: bit_col,
            s,
        }
    }
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        index: Value<F>,
        a: Value<CellWrapper<F>>,
        b: Value<F>,
        offset: usize,
    ) -> Result<(CellWrapper<F>, CellWrapper<F>), Error> {
        layout.assign_region(
            || "reverse assign",
            |mut region| {
                index.map(|v| {
                    if v == F::ONE {
                        self.config.s.enable(&mut region, offset).unwrap();
                    }
                });
                // FIXME: 这里的代码有问题，需要修改
                let mut internal = Value::known(F::ONE);
                a.clone().map(|mut v| {
                    let cell = v.cell();
                    internal = cell.value().cloned()
                });
                let l: AssignedCell<F, F> =
                    region.assign_advice(|| "lhs", self.config.a, 0, || internal)?;
                let r = region.assign_advice(|| "rhs", self.config.b, 0, || b)?;

                Ok((CellWrapper::new(l), CellWrapper::new(r)))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    pub struct MockCircut {}
    #[test]
    pub fn test_asd() {}
}
