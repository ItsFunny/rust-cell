use crate::circuits::halo2::chip2::{RescueHashChip, RescueHashConfig};
use crate::circuits::halo2::index_to_bool::{IndexToBoolChip, IndexToBoolConfig};
use crate::circuits::halo2::reverse::{ReverseChip, ReverseConfig};
use halo2_proofs::circuit::{Layouter, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{ConstraintSystem, Error, Expression};
use merkle_tree::primitives::BitIteratorLe;
use std::marker::PhantomData;

pub struct MerkleConfig<const D: usize> {
    pub reverse: ReverseConfig,
    pub rescue: RescueHashConfig,
    pub index_bool: IndexToBoolConfig<D>,
}
pub struct MerkleChip<F: PrimeField, const D: usize> {
    config: MerkleConfig<D>,
    _ph: PhantomData<F>,
}

impl<F: PrimeField, const D: usize> MerkleChip<F, D> {
    pub fn new(config: MerkleConfig<D>) -> Self {
        Self {
            config,
            _ph: Default::default(),
        }
    }
    pub fn configure(meta: &mut ConstraintSystem<F>) -> MerkleConfig<D> {
        let reverse_config = ReverseChip::configure(meta);
        let rescue_config = RescueHashChip::configure(meta);
        let index_to_bool_config = IndexToBoolChip::configure(meta);
        MerkleConfig {
            reverse: reverse_config,
            rescue: rescue_config,
            index_bool: index_to_bool_config,
        }
    }
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        leaf: Option<Vec<bool>>,
        index: Value<F>,
        audit_path: &Vec<Value<F>>,
    ) -> Result<(), Error> {
        let first_leaf = self.assign_fist_leaf(layout.namespace(|| "assign first leaf"), leaf)?;

        let index = self.index_to_bools(layout.namespace(|| "get index"), index);

        Ok(())
    }
    pub fn assign_fist_leaf(
        &self,
        mut layout: impl Layouter<F>,
        leaf: Option<Vec<bool>>,
    ) -> Result<Expression<F>, Error> {
        let hash_chip = RescueHashChip::construct(self.config.rescue.clone());
        let hash: Expression<F> = hash_chip.assign(layout.namespace(|| "assign hash"), leaf)?;

        Ok(hash)
    }

    // TODO: add constraints
    fn index_to_bools(
        &self,
        layout: impl Layouter<F>,
        index: Value<F>,
    ) -> Value<Vec<Option<bool>>> {
        let index = index.as_ref();
        index.map(|value| {
            let mut field_char = BitIteratorLe::new(F::MODULUS);

            let mut tmp: Vec<Option<bool>> = Vec::with_capacity(F::NUM_BITS as usize);

            let mut found_one = false;
            for b in BitIteratorLe::new(value.to_repr()) {
                // Skip leading bits
                found_one |= field_char.next().unwrap();
                if !found_one {
                    continue;
                }

                tmp.push(Some(b));
            }

            assert_eq!(tmp.len(), F::NUM_BITS as usize);

            tmp
        })
    }
}

#[test]
pub fn test_asd() {
    let u64 = [
        2208438750497609656u64,
        15320337363210015184,
        15995090123793024847,
        2117741280830857960,
        10431444199689328394,
        2354366303523504534,
        6239201738238234818,
        985956890050594503,
        49900000000,
        0,
    ];
    let mut data = vec![];
    u64.into_iter().for_each(|v| data.extend(v.to_le_bytes()));
    println!("{:?}", data);
}
