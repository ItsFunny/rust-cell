#[macro_use]
extern crate rocket;

use ark_bn254::{Bn254, Fq12, Fr, G1Affine, G2Affine, Parameters};
use ark_circom::{read_zkey, CircomBuilder, CircomCircuit, CircomConfig};
use ark_ff::to_bytes;
use ark_groth16::{
    create_random_proof as prove, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, ProvingKey,
};
use ark_relations::r1cs::{
    ConstraintMatrices, ConstraintSynthesizer, ConstraintSystem, OptimizationGoal,
    Result as R1CSResult,
};
use ark_serialize::CanonicalSerialize;
use ark_std::rand::thread_rng;
use lazy_static::lazy_static;
use num_bigint::{BigInt, BigUint};
use num_traits::{Num, Zero};
use rocket::figment::map;
use rocket::futures::future::{ok, OkInto};
use rocket::http::hyper::body::to_bytes;
use rocket::response::Responder;
use rocket::serde::json::{json, serde_json, Json, Value};
use rocket::serde::{Deserialize, Serialize};
use rocket::Request;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::io::Bytes;

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
        )
        .unwrap();

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

    pub fn generate_prove(&self, a: BigInt, b: BigInt) -> (R1CSResult<Proof<Bn254>>, Vec<Fr>) {
        let mut builder = CircomBuilder::new(self.zkp_config.clone());
        builder.push_input("a", a);
        builder.push_input("b", b);
        let mut rng = thread_rng();
        let circom = builder.build().unwrap();
        let mut rng = thread_rng();
        let inputs = circom.get_public_inputs().unwrap();
        (prove(circom, &self.zkp_params, &mut rng), inputs)
    }

    pub fn simple_verify(&self) {}
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
    public_input: String,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
struct VerifyRequest<'r> {
    proof: Cow<'r, str>,
    vk: Cow<'r, str>,
    public_input: Cow<'r, str>,
}

struct VerifyResponse {}

#[post("/", format = "json", data = "<message>")]
fn simple_prove_api(message: Json<ProveRequest<'_>>) -> String {
    match message.a.parse::<BigInt>() {
        Ok(v) => match message.b.parse::<BigInt>() {
            Ok(vv) => {
                let zkp_res = ZKPInstance.generate_prove(v, vv);
                match zkp_res.0 {
                    Ok(data) => {
                        let proof_bytes = to_bytes!(data).unwrap();
                        let vk_bytes = to_bytes!(ZKPInstance.zkp_params.vk).unwrap();
                        let public_input_bytes = to_bytes!(zkp_res.1).unwrap();
                        let ret = ProveResponse {
                            proof: hex::encode(proof_bytes),
                            vk: hex::encode(vk_bytes),
                            public_input: hex::encode(public_input_bytes),
                        };
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
        },
        Err(e) => {
            format!("{}", "err")
        }
    }
}

#[post("/", format = "json", data = "<message>")]
fn verify_api(message: Json<VerifyRequest>) -> String {
    // let cfg = ZKPInstance.zkp_config.clone();
    // let mut builder = CircomBuilder::new(cfg);
    // builder.push_input("a", 3);
    // builder.push_input("b", 11);
    let proof = message.proof.clone();
    let vk = message.vk.clone();
    let pb = message.public_input.clone();

    if let Ok(proof_resp) = hex::decode(proof.to_string()) {
        if let Ok(vk_resp) = hex::decode(vk.to_string()) {
            if let Ok(pb_resp) = hex::decode(pb.to_string()) {
                // let pvk = prepare_verifying_key(&params.vk);
                // let verified = verify_proof(&pvk, &proof, &inputs).unwrap();
            }
        }
    }
    error_return("asd")
}

fn error_return(str: &'static str) -> String {
    String::from(str)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/prove", routes![simple_prove_api])
        .mount("/verify", routes![verify_api])
}
