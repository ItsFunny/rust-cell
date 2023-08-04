// use franklin_crypto::circuit::multipack;
// use crate::smt::hasher::Hasher;
// use pasta_curves::group::ff::PrimeField;
// use pasta_curves::Fp;
//
// pub struct Halo2RescueHasher {}
// impl Hasher<Fp> for Halo2RescueHasher {
//     fn hash_bits<I: IntoIterator<Item = bool>>(&self, input: I) -> Fp {
//         let bits: Vec<bool> = input.into_iter().collect();
//         let packed = multipack::compute_multipacking::<E>(&bits);
//         let sponge_output = rescue_hash::<E>(self.params, &packed);
//         assert_eq!(sponge_output.len(), 1);
//         sponge_output[0]
//     }
//
//     fn hash_elements<I: IntoIterator<Item = Fp>>(&self, elements: I) -> Fp {
//         todo!()
//     }
//
//     fn compress(&self, lhs: &Fp, rhs: &Fp, i: usize) -> Fp {
//         todo!()
//     }
// }
