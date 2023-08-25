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

        meta.create_gate("reverse gate", |meta| {
            let a = meta.query_advice(a_col, Rotation::cur());
            let b = meta.query_advice(b_col, Rotation::cur());
            let bit = meta.query_advice(bit_col, Rotation::cur());
            let s = meta.query_selector(s);

            let l = meta.query_advice(a_col, Rotation::next());
            let r = meta.query_advice(b_col, Rotation::next());

            vec![
                // s.clone()
                //     * (r.clone() - b.clone() - bit.clone() * (a.clone() - b.clone()) + l.clone()
                //         - a.clone()
                //         - bit.clone() * (a.clone() - b.clone())),
                s * ((bit * F::from(2) * (b.clone() - a.clone()) - (l - a)) - (b - r)),
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
        region_name: usize,
    ) -> Result<(CellWrapper<F>, CellWrapper<F>), Error> {
        let offset = 0;
        layout.assign_region(
            || format!("reverse gate {}", region_name),
            |mut region| {
                self.config.s.enable(&mut region, offset).unwrap();
                // FIXME
                let mut lhs = Default::default();
                let mut rhs = Default::default();

                index.map(|v| {
                    if v == F::ONE {
                        region
                            .assign_advice(
                                || "bit",
                                self.config.bit,
                                offset,
                                || Value::known(F::ONE),
                            )
                            .unwrap();
                        a.clone().map(|mut v| {
                            let cell = v.cell();
                            rhs = cell.value().cloned()
                        });
                        lhs = b.clone();
                    } else {
                        region
                            .assign_advice(
                                || "bit",
                                self.config.bit,
                                offset,
                                || Value::known(F::ZERO),
                            )
                            .unwrap();
                        a.clone().map(|mut v| {
                            let cell = v.cell();
                            lhs = cell.value().cloned()
                        });
                        rhs = b.clone();
                    }
                });

                let l: AssignedCell<F, F> =
                    region.assign_advice(|| "lhs", self.config.a, offset + 1, || lhs)?;
                let r = region.assign_advice(|| "rhs", self.config.b, offset + 1, || rhs)?;

                Ok((CellWrapper::new(l), CellWrapper::new(r)))
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::circuits::halo2::reverse::{ReverseChip, ReverseConfig};
    use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner};
    use halo2_proofs::pasta::Fp;
    use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};

    pub struct MockCircut {}
    impl Circuit<Fp> for MockCircut {
        type Config = ReverseConfig;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            todo!()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            ReverseChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            // let chip = ReverseChip::new(config);
            todo!()
        }
    }
    #[test]
    pub fn test_asd() {}
}
