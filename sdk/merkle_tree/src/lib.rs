pub mod circuit;
pub mod error;
pub mod params;
pub mod primitives;
pub mod smt;

use franklin_crypto::alt_babyjubjub::AltJubjubBn256;
use franklin_crypto::rescue::bn256::Bn256RescueParams;
use franklin_crypto::{
    bellman::{pairing::bn256, plonk::better_cs::cs::PlonkCsWidth4WithNextStepParams},
    eddsa::{PrivateKey as PrivateKeyImport, PublicKey as PublicKeyImport},
    jubjub::JubjubEngine,
};

mod crypto_exports {
    pub use franklin_crypto::{
        bellman,
        bellman::{pairing, pairing::ff},
    };
    pub use rand;
    pub use recursive_aggregation_circuit;
    pub use rescue_poseidon;
}

pub type Engine = bn256::Bn256;
pub type Fr = bn256::Fr;
pub type RescueParams = Bn256RescueParams;
pub type JubjubParams = AltJubjubBn256;
pub type Fs = <Engine as JubjubEngine>::Fs;
pub type PlonkCS = PlonkCsWidth4WithNextStepParams;

pub type PrivateKey = PrivateKeyImport<Engine>;
pub type PublicKey = PublicKeyImport<Engine>;
