use crate::circuits::halo2::temp::temp_a;
use crate::utils::fq_to_fr;
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::pasta::Fp;
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use merkle_tree::primitives::BitIteratorLe;
use merkle_tree::Fr;
use std::marker::PhantomData;

// value|..|selector
// ..   |..| 1

#[derive(Debug, Clone)]
pub struct IndexToBoolConfig<F: PrimeField, const D: usize> {
    _ph: PhantomData<F>,
    points: [Column<Advice>; D],
    constants: [Column<Fixed>; D],
    count: Column<Fixed>,
    s: Selector,
}
pub struct IndexToBoolChip<F: PrimeField, const D: usize> {
    pub config: IndexToBoolConfig<F, D>,
}

impl<F: PrimeField, const D: usize> IndexToBoolChip<F, D> {
    pub fn new(config: IndexToBoolConfig<F, D>) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<F>) -> IndexToBoolConfig<F, D> {
        let select = meta.selector();
        let count = meta.fixed_column();
        meta.enable_equality(count);

        let mut advices = vec![];
        let mut instances = vec![];
        for _ in 0..D {
            let adl = meta.advice_column();
            let constants = meta.fixed_column();
            // meta.enable_equality(adl);
            // meta.enable_constant(constants);
            advices.push(adl);
            instances.push(constants);
        }

        meta.create_gate("index", |meta| {
            let s = meta.query_selector(select);
            let adl = meta.query_advice(advices[0], Rotation::cur());
            let instance = meta.query_fixed(instances[0]);

            let count = meta.query_fixed(count);
            let mut ret = instance * adl;
            for i in 1..D {
                let adl = meta.query_advice(advices[i], Rotation::cur());
                let inst = meta.query_fixed(instances[i]);
                ret = ret + (adl * inst);
            }

            vec![s * (ret - count)]
        });
        IndexToBoolConfig {
            _ph: Default::default(),
            points: <[Column<Advice>; D]>::try_from(advices).unwrap(),
            constants: <[Column<Fixed>; D]>::try_from(instances).unwrap(),
            count,
            s: select,
        }
    }
    pub fn assign(&self, mut layout: impl Layouter<F>, index: Value<F>) -> Result<(), Error> {
        let index = index.as_ref();
        let values: Value<Vec<Option<bool>>> = index.map(|value| {
            println!("assign f_modules:{:?}", F::MODULUS);
            let mut tmp: Vec<Option<bool>> = Vec::with_capacity(F::CAPACITY as usize);
            for b in BitIteratorLe::new(value.to_repr()) {
                tmp.push(Some(b));
            }
            println!(
                "z {:?},{:?},{:?},{:?}",
                F::CAPACITY,
                F::MODULUS,
                F::NUM_BITS,
                tmp.len()
            );
            tmp
        });
        layout.assign_region(
            || "assign",
            |mut region| {
                let mut count = 0;
                let cells = values.clone().map(|bools| {
                    bools
                        .into_iter()
                        .enumerate()
                        .take(D)
                        .map(|(i, b)| {
                            let point = self.config.points[i];
                            let const_point = self.config.constants[i];
                            // FIXME
                            let value = b.map_or(Value::known(F::ZERO), |v| {
                                if v {
                                    count = count + 1;
                                    // FIXME : unwrap
                                    region
                                        .assign_fixed(
                                            || format!("assign:{}", i),
                                            const_point,
                                            0,
                                            || Value::known(F::ONE),
                                        )
                                        .unwrap();
                                    Value::known(F::ONE)
                                } else {
                                    Value::known(F::ZERO)
                                }
                            });

                            // FIXME : unwrap
                            let b = region
                                .assign_advice(|| format!("assign:{}", i), point, 0, || value)
                                .unwrap();
                            b
                        })
                        .collect::<Vec<AssignedCell<F, F>>>()
                });

                region
                    .assign_fixed(
                        || format!("assign count"),
                        self.config.count,
                        0,
                        || Value::known(F::from(count)),
                    )
                    .unwrap();

                self.config.s.enable(&mut region, 0)?;
                Ok(())
            },
        )?;
        Ok(())
    }
}

struct MockCircuit {
    index: Value<Fp>,
}
impl Circuit<Fp> for MockCircuit {
    type Config = IndexToBoolConfig<Fp, 1>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        todo!()
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        IndexToBoolChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        let chip = IndexToBoolChip::new(config);
        chip.assign(layouter.namespace(|| "assign"), self.index.clone())?;
        Ok(())
    }
}
#[test]
pub fn test_index() {
    println!("{:?}", Fp::ONE + Fp::ONE);
    let circuit = MockCircuit {
        index: Value::known(Fp::from(5u64)),
    };
    let prover = MockProver::run(8, &circuit, vec![]).unwrap();
    assert_eq!(prover.verify(), Ok(()))
}

#[test]
pub fn test_to_bool() {
    // assign f_modules:"0x40000000000000000000000000000000224698fc094cf91b992d30ed00000001"
    println!(
        "aaaaa {:?}",
        Fp::from_raw([1u64, 0u64, 0u64, 0x0800_0000_0000_0011])
    );
    // [Some(true), Some(false), Some(true), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false)]
    // [Some(true), Some(false), Some(true), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false), Some(false)]
    let fr = fq_to_fr(&Fp::from(5u64));
    temp_a(Some(fr))
}

#[cfg(test)]
#[cfg(feature = "dev-graph")]
mod tests {
    use super::*;
    use halo2_proofs::dev::circuit_dot_graph;

    #[test]
    pub fn test_print() {
        use plotters::prelude::*;

        let root =
            BitMapBackend::new("example-circuit-layout.png", (1024, 768)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root
            .titled("Example Circuit Layout", ("sans-serif", 60))
            .unwrap();

        let circuit = MockCircuit {
            index: Value::known(Fp::from(5u64)),
        };
        let a = circuit_dot_graph(&circuit);
        println!("xxxxxx {:?}", a);
        halo2_proofs::dev::CircuitLayout::default()
            .show_labels(true)
            .show_equality_constraints(true)
            .mark_equality_cells(true)
            .render(5, &circuit, &root)
            .unwrap();
    }
}
