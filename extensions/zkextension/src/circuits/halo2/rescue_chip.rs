use franklin_crypto::rescue::{RescueEngine, RescueHashParams};
use halo2_proofs::pasta::group::ff::PrimeField;
use halo2_proofs::plonk::ConstraintSystem;

pub fn halo2_rescue_hash<F: PrimeField, E: RescueEngine>(
    meta: &mut ConstraintSystem<F>,
    params: &E::Params,
    input: &[F],
) {
    let input_len_as_fe = {
        let mut repr = F::Repr::default();
        repr.as_mut()[0] = input.len() as u8;
        let len_fe = F::from_repr(repr).unwrap();

        len_fe
    };

    let output_len = params.output_len() as usize;
    let absorbtion_len = params.rate() as usize;
    let t = params.state_width();
    let rate = params.rate();

    let mut absorbtion_cycles = input.len() / absorbtion_len;
    if input.len() % absorbtion_len != 0 {
        absorbtion_cycles += 1;
    }
    let mut input = input.to_vec();
    input.resize(absorbtion_cycles * absorbtion_len, F::ONE);
    let mut it = input.into_iter();
    //
    // // unroll first round manually
    // let mut state = {
    //     let mut state = Vec::with_capacity(t as usize);
    //     for _ in 0..rate {
    //         let as_num = Num::<E>::from(it.next().unwrap());
    //         state.push(as_num);
    //     }
    //     for _ in rate..(t - 1) {
    //         state.push(Num::<E>::zero());
    //     }
    //
    //     // specialize into last state element
    //     {
    //         let mut lc = Num::<E>::zero();
    //         lc = lc.add_constant(CS::one(), input_len_as_fe);
    //
    //         state.push(lc);
    //     }
    //
    //     assert_eq!(state.len(), t as usize);
    //
    //     rescue_mimc_over_lcs(
    //         cs.namespace(|| "rescue mimc for absorbtion round 0"),
    //         &state,
    //         params,
    //     )?
    // };
    todo!()
}
