extern crate core;

use color_eyre::Result;
use std::fmt::Error;
use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::{generate_random_parameters, prepare_verifying_key, verify_proof, create_random_proof as prove};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal,
    SynthesisError, SynthesisMode,
};
use ark_std::rand::thread_rng;
use color_eyre::eyre::ErrReport;

fn main() {
}

#[test]
fn groth16_proof() {
    // Load the WASM and R1CS for witness and proof generation
    let cfg = CircomConfig::<Bn254>::new(
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.wasm",
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.r1cs",
    ).unwrap();

// Insert our public inputs as key value pairs
    let mut builder = CircomBuilder::new(cfg);
    builder.push_input("a", 3);
    builder.push_input("b", 11);

// Create an empty instance for setting it up
    let circom = builder.setup();

// Run a trusted setup
    let mut rng = thread_rng();
    let params = generate_random_parameters::<Bn254, _, _>(circom, &mut rng).unwrap();

// Get the populated instance of the circuit with the witness
    let circom = builder.build().unwrap();

    let inputs = circom.get_public_inputs().unwrap();

// Generate the proof
    let proof = prove(circom, &params, &mut rng).unwrap();


// Check that the proof is valid
    let pvk = prepare_verifying_key(&params.vk);
    let verified = verify_proof(&pvk, &proof, &inputs).unwrap();
    assert!(verified);
}