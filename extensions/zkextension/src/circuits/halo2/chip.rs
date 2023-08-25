use crate::circuits::halo2::BlockHalo2Config;
use crate::traces::TraceTableCircuit;
use crate::utils::{fr_from, fr_to_fq};
use franklin_crypto::bellman::bn256::Bn256;
use franklin_crypto::bellman::Field as FranklinField;
use franklin_crypto::bellman::{ConstraintSystem as PlonkConstraintSystem, SynthesisError};
use franklin_crypto::circuit::boolean::Boolean;
use franklin_crypto::circuit::num::{AllocatedNum, Num};
use franklin_crypto::circuit::test::TestConstraintSystem;
use franklin_crypto::circuit::{multipack, rescue, Assignment};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::circuit::{AssignedCell, Chip, Layouter};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{Advice, Column, ConstraintSystem, Instance};
use merkle_tree::params::RESCUE_PARAMS;
use merkle_tree::{Engine, Fr, RescueParams};
use rescue_poseidon::rescue_hash;
use std::marker::PhantomData;
use std::vec;

// lhs|rhs|root_cur
pub struct MerkleConfig {
    merkle_path: [Column<Advice>; 64],
    lhs: Column<Advice>,
    rhs: Column<Advice>,
    cur_root: Column<Advice>,

    new_root: Column<Instance>,
}
pub struct MerkleChip<F: PrimeField> {
    config: BlockHalo2Config,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> MerkleChip<F> {
    pub fn new(config: BlockHalo2Config) -> Self {
        Self {
            config,
            _marker: Default::default(),
        }
    }
}

pub trait MerkleChipTrait<F: PrimeField>: Chip<F> {
    fn allocate_merkle_root(
        &self,
        layouter: impl Layouter<F>,
        traces: &TraceTableCircuit<F>,
    ) -> AssignedCell<F, F>;
}

impl<F: PrimeField> Chip<F> for MerkleChip<F> {
    type Config = BlockHalo2Config;
    type Loaded = ();

    fn config(&self) -> &Self::Config {
        &self.config
    }

    fn loaded(&self) -> &Self::Loaded {
        &()
    }
}

impl<F: PrimeField> MerkleChipTrait<F> for MerkleChip<F> {
    fn allocate_merkle_root(
        &self,
        layouter: impl Layouter<F>,
        traces: &TraceTableCircuit<F>,
    ) -> AssignedCell<F, F> {
        todo!()
    }
}

pub fn get_delta_root<F: PrimeField>(
    leaf_bits: &[bool],
    index: u64,
    audit_path: &[Fr],
    length_to_root: usize,
) -> F {
    let mut cs = TestConstraintSystem::new();
    let mut bool_bits = vec![];
    for item in leaf_bits {
        bool_bits.push(Boolean::constant(*item));
    }
    let mut allocated = vec![];
    for (i, e) in audit_path.iter().enumerate() {
        let fr = e.clone();
        let path_element =
            AllocatedNum::alloc(cs.namespace(|| format!("path element{}", i)), || Ok(fr)).unwrap();
        allocated.push(path_element);
    }
    let index_fr = fr_from(index);
    let id = AllocatedNum::alloc(cs.namespace(|| "id"), || Ok(index_fr)).unwrap();
    let index = id
        .into_bits_le_fixed(cs.namespace(|| "into_bits_le_fixed"), 64)
        .unwrap();

    get_merkle_root(
        cs.namespace(|| "get root"),
        &bool_bits,
        &index.as_slice(),
        &allocated,
        length_to_root,
        &RESCUE_PARAMS,
    )
}
fn get_merkle_root<F: PrimeField, CS: PlonkConstraintSystem<Engine>>(
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

fn do_allocate_merkle_root<CS: PlonkConstraintSystem<Engine>>(
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

    let index = &index[0..length_to_root];
    let audit_path = &audit_path[0..length_to_root];

    let leaf_packed_vec = simple_pack_into_witness(leaf_bits).unwrap();

    let hash: Vec<Fr> =
        franklin_crypto::rescue::rescue_hash::<Bn256>(params, leaf_packed_vec.as_slice());
    let hash = hash.get(0).unwrap().clone();
    let mut cur_hash = AllocatedNum::alloc(cs.namespace(|| "aaaasdasd"), || Ok(hash)).unwrap();

    let mut leaf_packed = vec![];
    for (i, fr) in leaf_packed_vec.iter().enumerate() {
        let fr = fr.clone();
        leaf_packed
            .push(AllocatedNum::alloc(cs.namespace(|| format!("aaa{:?}", i)), || Ok(fr)).unwrap());
    }
    let mut account_leaf_hash = rescue::rescue_hash(
        cs.namespace(|| "account leaf content hash"),
        &leaf_packed,
        params,
    )?;

    // println!("<<<<<<<<<<<<<<<<<<<<<<<");
    // TODO, add constraint
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
        // println!(
        //     "bool:{:?},cur hash:{:?}",
        //     direction_bit.get_value(),
        //     cur_hash.get_value().as_ref().unwrap()
        // );
    }
    // println!(">>>>>>>>>>>>>>>>>>>>>>>>>");

    Ok(cur_hash)
}
pub fn simple_pack_into_witness(bits: &[Boolean]) -> Result<Vec<Fr>, SynthesisError> {
    let mut results = vec![];

    for (_, bits) in bits.chunks(253 as usize).enumerate() {
        let mut coeff = Fr::one();
        let mut value = Some(Fr::zero());
        for bit in bits {
            let newval: Option<Fr> = match (value, bit.get_value()) {
                (Some(mut curval), Some(bval)) => {
                    if bval {
                        curval.add_assign(&coeff);
                    }

                    Some(curval)
                }
                _ => None,
            };
            value = newval;

            coeff.double();
        }
        // TODO, add constraint
        let fr = value.unwrap();
        results.push(fr);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::circuits::halo2::chip::{get_delta_root, get_merkle_root};
    use crate::instance::merkle::u8_array_to_bool_vec;
    use halo2_proofs::circuit::NamespacedLayouter;
    use halo2_proofs::pasta::Fq;
    use halo2_proofs::plonk::ConstraintSystem;
    use hash_db::Hasher;
    use keccak_hasher::KeccakHasher;
    use merkle_tree::params::RESCUE_PARAMS;
    use merkle_tree::primitives::GetBits;
    use merkle_tree::smt::rescue_hasher::RescueHasher;
    use merkle_tree::{Engine, Fr, RescueParams, SparseMerkleTree};
    use std::collections::hash_map::DefaultHasher;
    use tree::tree::NullHasher;

    pub type NodeTree = SparseMerkleTree<Node, Fr, RescueHasher<Engine>>;
    #[derive(Default, Clone)]
    pub struct Node {
        data: Vec<u8>,
    }

    impl Node {
        pub fn new(data: &str) -> Self {
            Self {
                data: data.as_bytes().to_vec(),
            }
        }
    }

    impl GetBits for Node {
        fn get_bits_le(&self) -> Vec<bool> {
            let v = KeccakHasher::hash(self.data.as_slice());
            u8_array_to_bool_vec(&v)
        }
    }
    #[test]
    pub fn test_get_root() {
        let mut tree = NodeTree::new(64);
        tree.insert(1, Node::new("data1"));
        tree.insert(2, Node::new("data2"));
        let root1 = tree.root_hash();
        println!("root1 {:?}", root1);

        let tree2 = tree.clone();
        let aa: Vec<Option<Fr>> = tree2
            .merkle_path(3)
            .into_iter()
            .map(|e| Some(e.0))
            .collect();

        let node3 = Node::new("data3");
        let binding = node3.clone().get_bits_le();
        let leaf_bits = binding.as_slice();
        let merkle_path: Vec<Fr> = aa.into_iter().map(|v| v.unwrap()).collect();
        let root_delta: Fq = get_delta_root(leaf_bits, 3, merkle_path.as_slice(), 64);
        println!("root_delta:{:?}", root_delta);

        tree.insert(3, node3.clone());
        let new_root = tree.root_hash();
        println!("new root {:?}", new_root);
    }
}
