use crate::circuits::halo2::chip2::{RescueHashChip, RescueHashConfig};
use crate::circuits::halo2::index_to_bool::{IndexToBoolChip, IndexToBoolConfig};
use crate::circuits::halo2::reverse::{ReverseChip, ReverseConfig};
use crate::circuits::halo2::wrapper::CellWrapper;
use halo2_proofs::circuit::{Layouter, Value};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::{
    Advice, Column, ConstraintSystem, Error, Expression, Instance, Selector,
};
use halo2_proofs::poly::Rotation;
use merkle_tree::primitives::BitIteratorLe;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct MerkleConfig<const D: usize> {
    pub reverse: ReverseConfig,
    pub rescue: RescueHashConfig,
    pub index_bool: IndexToBoolConfig<D>,

    pub input: Column<Instance>,
    pub output: Column<Advice>,
    pub s: Selector,
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
        let input = meta.instance_column();
        let output = meta.advice_column();
        let s = meta.selector();
        meta.create_gate("output check", |meta| {
            let input = meta.query_instance(input, Rotation::cur());
            let output = meta.query_advice(output, Rotation::cur());
            let s = meta.query_selector(s);
            vec![s * (input - output)]
        });
        MerkleConfig {
            reverse: reverse_config,
            rescue: rescue_config,
            index_bool: index_to_bool_config,
            input,
            output,
            s,
        }
    }
    pub fn assign(
        &self,
        mut layout: impl Layouter<F>,
        leaf: Option<Vec<bool>>,
        index: Value<F>,
        audit_path: &Vec<Value<F>>,
    ) -> Result<(), Error> {
        let index_chip = IndexToBoolChip::new(self.config.index_bool.clone());
        let index = index_chip.assign(layout.namespace(|| "index"), index)?;

        let cur_hash = self.assign_fist_leaf(layout.namespace(|| "assign first leaf"), leaf)?;
        let mut cur_hash = Value::known(cur_hash);
        let a = index.map(|v| {
            for (i, direction_bit) in v.iter().enumerate() {
                let path_element = &audit_path[i];
                // Swap the two if the current subtree is on the right
                let index = direction_bit.value().cloned();
                let reverse_chip = ReverseChip::new(self.config.reverse.clone());
                // get the current hash
                let path_element = audit_path[i].clone();
                let v = reverse_chip
                    .assign(
                        layout.namespace(|| "reverse"),
                        index,
                        cur_hash.clone(),
                        path_element,
                        i,
                    )
                    .unwrap();
                let lhs = v.0;
                let rhs = v.1;
                let hash_chip = RescueHashChip::construct(self.config.rescue.clone());

                let hash = hash_chip.assign_rescue(
                    layout.namespace(|| "assign rescue"),
                    lhs.cell(),
                    rhs.cell(),
                );
                cur_hash = Value::known(hash);
            }
            cur_hash
        });

        layout
            .assign_region(
                || "assign merkle region",
                |mut region| {
                    a.clone().map(|vv| {
                        vv.map(|v| {
                            let cell = v.cell();
                            let value = cell.value().cloned();
                            region
                                .assign_advice(|| "assign root", self.config.output, 0, || value)
                                .unwrap();
                        })
                    });
                    // cur_hash.clone().map(|v| {
                    //     let cell = v.cell();
                    //     let value = cell.value().cloned();
                    //     region
                    //         .assign_advice(|| "assign root", self.config.output, 0, || value)
                    //         .unwrap();
                    // });
                    Ok(())
                },
            )
            .unwrap();

        Ok(())
    }
    pub fn enforce() {}
    fn assign_fist_leaf(
        &self,
        mut layout: impl Layouter<F>,
        leaf: Option<Vec<bool>>,
    ) -> Result<CellWrapper<F>, Error> {
        let hash_chip = RescueHashChip::construct(self.config.rescue.clone());
        let hash = hash_chip.assign(layout.namespace(|| "assign hash"), leaf)?;

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

#[cfg(test)]
mod tests {
    use crate::circuits::halo2::merkle::{MerkleChip, MerkleConfig};
    use crate::utils::fr_to_fq;
    use halo2_proofs::circuit::{Layouter, SimpleFloorPlanner, Value};
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::pasta::{Fp, Fq};
    use halo2_proofs::plonk::{Circuit, ConstraintSystem, Error};
    use merkle_tree::primitives::GetBits;
    use merkle_tree::smt::parallel_smt;
    use merkle_tree::smt::rescue_hasher::RescueHasher;
    use merkle_tree::smt::storage::DefaultMemorySMTStorage;
    use merkle_tree::{Engine, Fr, SparseMerkleTree};

    struct MockCircuit {
        leaf: TestNode,
        index: u64,
        audit_path: Vec<Fp>,
    }

    impl Circuit<Fp> for MockCircuit {
        type Config = MerkleConfig<64usize>;
        type FloorPlanner = SimpleFloorPlanner;

        fn without_witnesses(&self) -> Self {
            todo!()
        }

        fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
            MerkleChip::configure(meta)
        }

        fn synthesize(
            &self,
            config: Self::Config,
            mut layouter: impl Layouter<Fp>,
        ) -> Result<(), Error> {
            let leaf = self.leaf.get_bits_le();
            let index = self.index;
            let merkle_paths = self
                .audit_path
                .clone()
                .into_iter()
                .map(|v| Value::known(v))
                .collect();
            let chip: MerkleChip<Fp, 64> = MerkleChip::new(config);
            chip.assign(
                layouter.namespace(|| "assign "),
                Some(leaf),
                Value::known(Fp::from(index)),
                &merkle_paths,
            )
        }
    }

    #[derive(Default, Clone)]
    pub struct TestNode {
        data: u64,
    }

    impl TestNode {
        pub fn new(data: u64) -> Self {
            Self { data }
        }
    }

    impl GetBits for TestNode {
        fn get_bits_le(&self) -> Vec<bool> {
            let mut bits = Vec::new();
            let mut data = self.data;
            for _ in 0..64 {
                bits.push(data & 1 == 1);
                data >>= 1;
            }
            bits
        }
    }
    pub type TestMerkleTree = parallel_smt::SparseMerkleTree<
        TestNode,
        Fr,
        RescueHasher<Engine>,
        DefaultMemorySMTStorage<TestNode>,
    >;
    #[test]
    pub fn test_merkle() {
        let mut tree = TestMerkleTree::new(64);
        for i in 0..10 {
            tree.insert(i, TestNode::new(i as u64));
        }
        let root = tree.root_hash();
        let path: Vec<Fr> = tree.merkle_path(1).into_iter().map(|e| e.0).collect();
        println!("{:?}", root);
        println!("{:?}", path);

        let path: Vec<Fp> = path.into_iter().map(|v| fr_to_fq(&v)).collect();
        let leaf = TestNode::new(120);
        let circuit = MockCircuit {
            leaf: leaf.clone(),
            index: 120,
            audit_path: path.clone(),
        };
        tree.insert(120, leaf);
        let root = tree.root_hash();
        let root: Fp = fr_to_fq(&root);
        let public_inputs: Vec<Vec<Fp>> = vec![vec![root.clone()]];
        let prover = MockProver::run(10, &circuit, public_inputs).unwrap();
    }
}
