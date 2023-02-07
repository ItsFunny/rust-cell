#[macro_use]
extern crate rocket;

use std::borrow::Cow;
use std::io::Bytes;
use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig};
use ark_ff::to_bytes;
use ark_groth16::{generate_random_parameters, prepare_verifying_key, verify_proof, create_random_proof as prove, ProvingKey, Proof};
use ark_std::rand::thread_rng;
use num_bigint::BigInt;
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem, OptimizationGoal,
    Result as R1CSResult,
};
use lazy_static::lazy_static;
use rocket::serde::{Serialize, Deserialize};
use rocket::figment::map;
use rocket::futures::future::ok;
use rocket::http::hyper::body::to_bytes;
use rocket::Request;
use rocket::response::Responder;
use rocket::serde::json::{Json, Value, json};



lazy_static! {
    static ref ZKPInstance: ZKP = init_zkp();
}

fn init_zkp() -> ZKP {
    ZKP::new()
}


pub struct ZKP {
    pub zkp_config: CircomConfig<Bn254>,
    pub zkp_params: ProvingKey<Bn254>,
    pub circom: CircomCircuit<Bn254>,
}


impl ZKP {
    pub fn new() -> ZKP {
        let mut rng = thread_rng();
        let cfg = CircomConfig::<Bn254>::new(
            "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.wasm",
            "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.r1cs",
        ).unwrap();

        let mut builder = CircomBuilder::new(cfg.clone());
        let circom = builder.setup();
        let cir = builder.build().unwrap();
        let params = generate_random_parameters::<Bn254, _, _>(cir, &mut rng).unwrap();
        ZKP {
            zkp_config: cfg.clone(),
            zkp_params: params.clone(),
            circom: circom,
        }
    }

    pub fn generate_prove(&self, a: BigInt, b: BigInt) -> R1CSResult<Proof<Bn254>> {
        let mut builder = CircomBuilder::new(self.zkp_config.clone());
        builder.push_input("a", a);
        builder.push_input("b", b);
        let mut rng = thread_rng();
        let circom = builder.build().unwrap();
        let mut rng = thread_rng();
        // let inputs = circom.get_public_inputs().unwrap();
        prove(circom, &self.zkp_params, &mut rng)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ProveRequest<'r> {
    a: Cow<'r, str>,
    b: Cow<'r, str>,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct ProveResponse {
    proof: String,
    vk: String,
}

#[post("/", format = "json", data = "<message>")]
fn prove_api(message: Json<ProveRequest<'_>>) -> String {
    match message.a.parse::<BigInt>() {
        Ok(v) => {
            match message.b.parse::<BigInt>() {
                Ok(vv) => {
                    match ZKPInstance.generate_prove(v, vv) {
                        Ok(data) => {
                            let proof_bytes = to_bytes!(data).unwrap();
                            let vk_bytes = to_bytes!(ZKPInstance.zkp_params.vk).unwrap();
                            let ret = ProveResponse { proof: hex::encode(proof_bytes), vk: hex::encode(vk_bytes) };
                            let ret = json!(ret);
                            format!("{}", ret.to_string())
                        }
                        Err(e) => {
                            format!("{}", "err")
                        }
                    }
                }
                Err(e) => {
                    format!("{}", "err")
                }
            }
        }
        Err(e) => {
            format!("{}", "err")
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/prove", routes![prove_api])
}

