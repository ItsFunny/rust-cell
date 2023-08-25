// use franklin_crypto::bellman::{
//     BitIterator, ConstraintSystem, PrimeField as PlonkPF, SynthesisError,
// };
// use franklin_crypto::circuit::test::TestConstraintSystem;
// use franklin_crypto::rescue::{RescueHashParams, SBox};
//
// pub fn temp_a<F: PlonkPF>(value: Option<F>) {
//     let values = match value {
//         Some(ref value) => {
//             println!("temp_a char {:?}", F::char());
//             let mut field_char = BitIterator::new(F::char());
//
//             let mut tmp = Vec::with_capacity(F::NUM_BITS as usize);
//
//             let mut found_one = false;
//             for b in BitIterator::new(value.into_repr()) {
//                 // Skip leading bits
//                 found_one |= field_char.next().unwrap();
//                 if !found_one {
//                     continue;
//                 }
//
//                 tmp.push(Some(b));
//             }
//
//             assert_eq!(tmp.len(), F::NUM_BITS as usize);
//
//             tmp
//         }
//         None => vec![None; F::NUM_BITS as usize],
//     };
//
//     // Allocate in little-endian order
//     let bits: Vec<Option<bool>> = values
//         .into_iter()
//         .rev()
//         .enumerate()
//         .take(64)
//         .map(|(i, b)| b)
//         .collect::<Vec<Option<bool>>>();
//     println!("{:?}", bits);
// }
