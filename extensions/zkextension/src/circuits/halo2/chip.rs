use crate::circuits::halo2::hasher::{fq_to_fr, fr_to_fq};
use crate::circuits::halo2::BlockHalo2Config;
use crate::circuits::traces::TraceTableCircuit;
use franklin_crypto::bellman::{ConstraintSystem, SynthesisError};
use franklin_crypto::circuit::boolean::Boolean;
use franklin_crypto::circuit::num::AllocatedNum;
use franklin_crypto::circuit::test::TestConstraintSystem;
use franklin_crypto::circuit::{multipack, rescue};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Chip, Layouter};
use halo2_proofs::pasta::group::ff::PrimeField;
use merkle_tree::{Engine, RescueParams};
use std::marker::PhantomData;

pub struct MerkleChip<F: Field> {
    config: BlockHalo2Config,
    _marker: PhantomData<F>,
}

impl<F: Field> MerkleChip<F> {
    pub fn new(config: BlockHalo2Config) -> Self {
        Self {
            config,
            _marker: Default::default(),
        }
    }
}

pub trait MerkleChipTrait<F: Field>: Chip<F> {
    fn allocate_merkle_root(
        &self,
        layouter: impl Layouter<F>,
        traces: &TraceTableCircuit<F>,
    ) -> AssignedCell<F, F>;
}

impl<F: Field> Chip<F> for MerkleChip<F> {
    type Config = BlockHalo2Config;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: Field> MerkleChipTrait<F> for MerkleChip<F> {
    fn allocate_merkle_root(
        &self,
        layouter: impl Layouter<F>,
        traces: &TraceTableCircuit<F>,
    ) -> AssignedCell<F, F> {
        todo!()
    }
}

// fn do_allocate_merkle_root<F: PrimeField>(
//     leaf_bits: Vec<bool>,
//     index: Vec<bool>,
//     audit_path: &[Option<F>],
//     length_to_root: usize,
//     params: &RescueParams,
// ) {
//     let index = &index[0..length_to_root];
//     let audit_path = &audit_path[0..length_to_root];
// }
pub fn get_root<F: PrimeField>(
    leaf_bits: &[bool],
    index: &[bool],
    audit_path: &[F],
    length_to_root: usize,
    params: &RescueParams,
) -> F {
    let mut cs = TestConstraintSystem::new();
    let leaf_bits = leaf_bits
        .into_iter()
        .map(|v| Boolean::constant(*v))
        .collect();
    let mut allocated = vec![];
    for (i, e) in audit_path.iter().enumerate() {
        let fr = fq_to_fr(v);
        let path_element =
            AllocatedNum::alloc(cs.namespace(|| format!("path element{}", i)), || Ok(fr))?;
        allocated.push(path_element);
    }
    let index = index.into_iter().map(|v| Boolean::constant(*v)).collect();

    get_merkle_root(
        cs.namespace(|| "get root"),
        &leaf_bits,
        &index,
        &allocated,
        length_to_root,
        params,
    )
}
fn get_merkle_root<F: PrimeField, CS: ConstraintSystem<Engine>>(
    mut cs: CS,
    leaf_bits: &[Boolean],
    index: &[Boolean],
    audit_path: &[AllocatedNum<Engine>],
    length_to_root: usize,
    params: &RescueParams,
) -> F {
    let alloc = do_allocate_merkle_root(
        cs.namespace(|| "allocate merkle root"),
        leaf_bits,
        index,
        audit_path,
        length_to_root,
        params,
    )
    .unwrap();
    let fr = alloc.get_value().unwrap();
    fr_to_fq(&fr)
}
fn do_allocate_merkle_root<CS: ConstraintSystem<Engine>>(
    mut cs: CS,
    leaf_bits: &[Boolean],
    index: &[Boolean],
    audit_path: &[AllocatedNum<Engine>],
    length_to_root: usize,
    params: &RescueParams,
) -> Result<AllocatedNum<Engine>, SynthesisError> {
    // only first bits of index are considered valuable
    assert!(length_to_root <= index.len());
    assert!(index.len() >= audit_path.len());

    let remaining_index_bits = AllocatedNum::pack_bits_to_element(
        cs.namespace(|| "index_bits_after_length_root_packed"),
        &index[length_to_root..],
    )?;
    remaining_index_bits.assert_zero(cs.namespace(|| "index_bits_after_length_are_zero"))?;

    let index = &index[0..length_to_root];
    let audit_path = &audit_path[0..length_to_root];

    let leaf_packed = multipack::pack_into_witness(
        cs.namespace(|| "pack leaf bits into field elements"),
        leaf_bits,
    )?;

    let mut account_leaf_hash = rescue::rescue_hash(
        cs.namespace(|| "account leaf content hash"),
        &leaf_packed,
        params,
    )?;

    assert_eq!(account_leaf_hash.len(), 1);

    let mut cur_hash = account_leaf_hash.pop().expect("must get a single element");

    // Ascend the merkle tree authentication path

    for (i, direction_bit) in index.iter().enumerate() {
        let cs = &mut cs.namespace(|| format!("from merkle tree hash {}", i));

        // "direction_bit" determines if the current subtree
        // is the "right" leaf at this depth of the tree.

        // Witness the authentication path element adjacent
        // at this depth.
        let path_element = &audit_path[i];

        // Swap the two if the current subtree is on the right
        let (xl, xr) = AllocatedNum::conditionally_reverse(
            cs.namespace(|| "conditional reversal of preimage"),
            &cur_hash,
            path_element,
            direction_bit,
        )?;

        // we do not use any personalization here cause
        // our tree is of a fixed height and hash function
        // is resistant to padding attacks
        let mut sponge_output = rescue::rescue_hash(
            cs.namespace(|| format!("hash tree level {}", i)),
            &[xl, xr],
            params,
        )?;

        assert_eq!(sponge_output.len(), 1);
        cur_hash = sponge_output.pop().expect("must get a single element");
    }

    Ok(cur_hash)
}
