use halo2_proofs::circuit::{AssignedCell, Layouter, SimpleFloorPlanner, Value};
use halo2_proofs::dev::MockProver;
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::pasta::Fp;
use halo2_proofs::plonk::{
    Advice, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use merkle_tree::primitives::BitIteratorLe;
use std::marker::PhantomData;

// value|selector
// ..   |..

#[derive(Debug, Clone)]
pub struct IndexToBoolConfig<F: PrimeField, const D: usize> {
    _ph: PhantomData<F>,
    points: [Column<Advice>; D],
    instances: [Column<Fixed>; D],
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
        let s = meta.selector();
        let mut advices = vec![];
        let mut instances = vec![];
        for _ in 0..D {
            let adl = meta.advice_column();
            let constants = meta.fixed_column();
            advices.push(adl);
            instances.push(constants);
        }

        meta.create_gate("index", |meta| {
            let adl = meta.query_advice(advices[0], Rotation::cur());
            let instance = meta.query_fixed(instances[0]);
            let s = meta.query_selector(s);
            let mut ret = instance * adl;
            for i in 1..D {
                let adl = meta.query_advice(advices[i], Rotation::cur());
                let inst = meta.query_fixed(instances[i]);
                ret = ret * (adl * inst);
            }

            vec![s * (ret - Expression::Constant(F::ONE))]
        });
        IndexToBoolConfig {
            _ph: Default::default(),
            points: <[Column<Advice>; D]>::try_from(advices).unwrap(),
            instances: <[Column<Fixed>; D]>::try_from(instances).unwrap(),
            s,
        }
    }
    pub fn assign(&self, mut layout: impl Layouter<F>, index: Value<F>) -> Result<(), Error> {
        let index = index.as_ref();
        let values: Value<Vec<Option<bool>>> = index.map(|value| {
            let mut field_char = BitIteratorLe::new(F::MODULUS);
            println!("z {:?},{:?},{:?}", F::CAPACITY, F::MODULUS, F::NUM_BITS);
            let mut tmp: Vec<Option<bool>> = Vec::with_capacity(F::CAPACITY as usize);
            let mut found_one = false;
            for b in BitIteratorLe::new(value.to_repr()) {
                // Skip leading bits
                found_one |= field_char.next().unwrap();
                if !found_one {
                    continue;
                }

                tmp.push(Some(b));
            }

            assert_eq!(tmp.len(), F::CAPACITY as usize);
            tmp
        });

        layout.assign_region(
            || "assign",
            |mut region| {
                self.config.s.enable(&mut region, 0)?;
                let cells = values.clone().map(|bools| {
                    bools
                        .into_iter()
                        .rev()
                        .enumerate()
                        .take(D)
                        .map(|(i, b)| {
                            let point = self.config.points[i];
                            let const_point = self.config.instances[i];
                            // FIXME
                            let value = b.map_or(Value::known(F::ZERO), |v| {
                                if v {
                                    // FIXME : unwrap
                                    region
                                        .assign_fixed(
                                            || format!("assign:{}", i),
                                            const_point,
                                            i,
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
                                .assign_advice(|| format!("assign:{}", i), point, i, || value)
                                .unwrap();
                            b
                        })
                        .collect::<Vec<AssignedCell<F, F>>>()
                });

                Ok(cells)
            },
        )?;
        Ok(())
    }
}

struct MockCircuit {
    index: Value<Fp>,
}
impl Circuit<Fp> for MockCircuit {
    type Config = IndexToBoolConfig<Fp, 64>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        MockCircuit {
            index: Default::default(),
        }
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
    let circuit = MockCircuit {
        index: Value::known(Fp::from(3u64)),
    };
    let prover = MockProver::run(8, &circuit, vec![]).unwrap();
    assert_ne!(prover.verify(), Ok(()))
}
