use franklin_crypto::bellman::{
    from_hex, to_hex, Field as OtherField, PrimeField as OtherPrimeField,
};
use franklin_crypto::rescue::bn256::Bn256RescueParams;
use franklin_crypto::rescue::RescueHashParams;
use franklin_crypto::rescue::SBox;
use franklin_crypto::{circuit::multipack, rescue::RescueEngine};
use halo2_proofs::arithmetic::Field;
use halo2_proofs::pasta::group::ff::{BitViewSized, PrimeField, PrimeFieldBits};
use halo2_proofs::pasta::Fq;
use merkle_tree::params::RESCUE_PARAMS;
use merkle_tree::smt::hasher::Hasher;
use merkle_tree::{Engine, Fr, RescueParams};
use std::io;
use std::io::{Read, Write};
use std::marker::PhantomData;

pub fn fq_to_fr<F: PrimeField>(f: &F) -> Fr {
    use byteorder::WriteBytesExt;
    let rep = f.to_repr();
    let data = rep.as_ref();

    let mut buf: Vec<u8> = Vec::with_capacity(data.len() * 8);
    for digit in data.iter().rev() {
        buf.write_u8(*digit).unwrap();
    }
    let binding = hex::encode(&buf);
    let str = binding.as_str();
    println!("{:?}", str);
    let fr = from_hex(str).unwrap();
    fr
}
pub fn fr_to_fq<F: PrimeField>(f: &Fr) -> F {
    let hex_fr = to_hex(f);
    fq_from_hex(hex_fr.as_str())
}

pub fn fq_from_hex<F: PrimeField>(value: &str) -> F {
    let value = if value.starts_with("0x") {
        &value[2..]
    } else {
        value
    };
    if value.len() % 2 != 0 {
        panic!("wrong");
    }
    let mut buf = hex::decode(&value).unwrap();
    let mut repr = F::Repr::default();
    let required_length = repr.as_ref().len() * 8;
    buf.reverse();
    buf.resize(required_length, 0);

    read_rep::<&[u8], F>(&mut repr, &buf[..]);

    F::from_repr(repr).unwrap()
}
fn read_rep<R: Read, F: PrimeField>(repr: &mut F::Repr, mut buf: R) {
    use byteorder::{LittleEndian, ReadBytesExt};
    for digit in repr.as_mut().iter_mut() {
        *digit = buf.read_u8().unwrap();
    }
}

#[test]
pub fn test_fq_to_fr() {
    let a = Fq::zero();
    let fr = fq_to_fr(&a);
    println!("{:?}", fr);
}
#[test]
pub fn test_fr_to_fq() {
    let a = Fr::one();
    let fq: Fq = fr_to_fq(&a);
    println!("{:?}", fq);
}
// pub fn to_hex<F: PrimeField>(el: &F) -> String {
//     let repr = el.to_repr();
//     let required_length = repr.as_ref().len() * 8;
//     let mut buf: Vec<u8> = Vec::with_capacity(required_length);
//     repr.write_be(&mut buf).unwrap();
//     hex_ext::encode(&buf)
// }

//
// pub fn compute_multipacking<F: PrimeField>(bits: &[bool]) -> Vec<F> {
//     let mut result = vec![];
//
//     for bits in bits.chunks(250 as usize) {
//         let mut cur = F::ZERO;
//         let mut coeff = F::ONE;
//
//         for bit in bits {
//             if *bit {
//                 cur = cur + coeff;
//             }
//
//             coeff = coeff.double();
//         }
//
//         result.push(cur);
//     }
//
//     result
// }
//
// pub fn rescue_hash<F: PrimeField>(params: &Bn256RescueParams, input: &[F]) -> Vec<F> {
//     sponge_fixed_length(params, input)
// }
//
// fn sponge_fixed_length<F: PrimeField>(params: &Bn256RescueParams, input: &[F]) -> Vec<F> {
//     assert!(input.len() > 0);
//     assert!(input.len() < 256);
//     let input_len = input.len() as u8;
//     let mut state = vec![F::ZERO; params.state_width() as usize];
//     // specialized for input length
//
//     let mut repr = F::Repr::default();
//     repr.as_mut()[0] = input_len;
//     let len_fe = F::from_repr(repr).unwrap();
//     let last_state_elem_idx = state.len() - 1;
//     state[last_state_elem_idx] = len_fe;
//
//     let rate = params.rate() as usize;
//     let mut absorbtion_cycles = input.len() / rate;
//     if input.len() % rate != 0 {
//         absorbtion_cycles += 1;
//     }
//     let padding_len = absorbtion_cycles * rate - input.len();
//     let padding = vec![F::ONE; padding_len];
//
//     let mut it = input.iter().chain(&padding);
//     for _ in 0..absorbtion_cycles {
//         for i in 0..rate {
//             state[i].add_assign(it.next().unwrap());
//         }
//         state = rescue_mimc(params, &state);
//     }
//
//     debug_assert!(it.next().is_none());
//
//     state[..(params.capacity() as usize)].to_vec()
// }
//
// pub fn rescue_mimc<F: PrimeField>(params: &Bn256RescueParams, old_state: &[F]) -> Vec<F> {
//     let mut state = old_state.to_vec();
//     let mut mds_application_scratch = vec![F::ZERO; state.len()];
//     assert_eq!(state.len(), params.state_width() as usize);
//     // add round constatnts
//     for (s, c) in state.iter_mut().zip(params.round_constants(0).iter()) {
//         let cc = to_hex(&c);
//         let another = F::from_str_vartime(cc).unwrap();
//         s.add_assign(another);
//     }
//
//     // parameters use number of rounds that is number of invocations of each SBox,
//     // so we double
//     for round_num in 0..(2 * params.num_rounds()) {
//         // apply corresponding sbox
//         if round_num & 1u32 == 0 {
//             params.sbox_0().apply(&mut state);
//         } else {
//             params.sbox_1().apply(&mut state);
//         }
//
//         // add round keys right away
//         mds_application_scratch.copy_from_slice(params.round_constants(round_num + 1));
//
//         // mul state by MDS
//         for (row, place_into) in mds_application_scratch.iter_mut().enumerate() {
//             let tmp = scalar_product(&state[..], params.mds_matrix_row(row as u32));
//             place_into.add_assign(&tmp);
//             // *place_into = scalar_product::<E>(& state[..], params.mds_matrix_row(row as u32));
//         }
//
//         // place new data into the state
//         state.copy_from_slice(&mds_application_scratch[..]);
//     }
//
//     state
// }
//
// fn scalar_product<F: PrimeField>(input: &[F], by: &[F]) -> F {
//     assert!(input.len() == by.len());
//     let mut result = F::ZERO;
//     for (a, b) in input.iter().zip(by.iter()) {
//         let mut tmp = *a;
//         tmp.mul_assign(b);
//         result.add_assign(&tmp);
//     }
//
//     result
// }

#[test]
pub fn testasd() {}
