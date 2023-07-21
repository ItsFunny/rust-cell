use halo2_proofs::circuit::{Layouter, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Error, Selector};
use merkle_tree::primitives::BitIteratorLe;
use std::marker::PhantomData;

pub struct IndexToBoolConfig<F: PrimeField, const D: usize> {
    _ph: PhantomData<F>,
    points: [Column<Advice>; D],
}
pub struct IndexToBoolChip<F: PrimeField, const D: usize> {
    pub config: IndexToBoolConfig<F, D>,
}

impl<F: PrimeField, const D: usize> IndexToBoolChip<F, D> {
    pub fn new(config: IndexToBoolConfig<F, D>) -> Self {
        Self { config }
    }
    pub fn configure(meta: &mut ConstraintSystem<F>) -> IndexToBoolConfig<F, D> {
        let mut advices = vec![];
        for i in 0..D {
            let adl = meta.advice_column();
            advices[i] = adl;
        }
        IndexToBoolConfig {
            _ph: Default::default(),
            points: <[Column<Advice>; D]>::try_from(advices).unwrap(),
        }
    }
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        index: Value<F>,
    ) -> Value<Vec<Option<bool>>> {
        let index = index.as_ref();
        layout.assign_region(||"assign",|mut region|{
            index.map(|value| {
                let mut field_char = BitIteratorLe::new(F::MODULUS);

                let mut tmp: Vec<Option<bool>> = Vec::with_capacity(F::NUM_BITS as usize);
                let mut offset = 0;
                let mut found_one = false;
                for b in BitIteratorLe::new(value.to_repr()) {
                    // Skip leading bits
                    found_one |= field_char.next().unwrap();
                    if !found_one {
                        continue;
                    }

                    tmp.push(Some(b));
                    offset = offset + 1;
                    
                }

                assert_eq!(tmp.len(), F::NUM_BITS as usize);

                ()
            });
            Ok(())
        });
        todo!()
        
    }
}

#[test]
pub fn test_asd() {}
