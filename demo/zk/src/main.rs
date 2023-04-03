mod request;

extern crate core;

use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig};
use ark_ff::ToBytes;
use ark_groth16::{
    create_random_proof as prove, generate_random_parameters, prepare_verifying_key, verify_proof,
    Proof, ProvingKey,
};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal, SynthesisError, SynthesisMode,
};
use ark_std::rand::rngs::ThreadRng;
use ark_std::rand::thread_rng;
use bytes::Bytes;
use cell_core::application::CellApplication;
use cell_core::cerror::{CellError, CellResult, ErrorEnums, ErrorEnumsStruct};
use cell_core::command::{ClosureFunc, Command};
use cell_core::constants::ProtocolStatus;
use cell_core::core::{runTypeHttp, runTypeHttpPost};
use cell_core::extension::{ExtensionFactory, NodeContext, NodeExtension};
use cell_core::wrapper::ContextResponseWrapper;
use cellhttp::extension::HttpExtensionFactory;
use cellhttp::request::HttpRequest;
use cellhttp::server::HttpServerBuilder;
use color_eyre::eyre::ErrReport;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::Result;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use logsdk::{cinfo, module_enums};
use num_bigint::{BigInt, ParseBigIntError};
use std::any::Any;
use std::cell::RefCell;
use std::env;
use std::fmt::Error;
use std::rc::Rc;
use std::sync::Arc;

module_enums!(
    (ZK,1,&logsdk::common::LogLevel::Info);
);

fn main() {
    let mut factories: Vec<Box<dyn ExtensionFactory>> = Vec::new();
    factories.push(Box::new(HttpExtensionFactory {}));
    factories.push(Box::new(ZKExtensionFactory {}));
    let mut app = CellApplication::new(factories);
    let args: Vec<String> = env::args().collect();
    app.run(args);
}

pub struct ZKExtension {
    command: Command<'static>,
}

impl NodeExtension for ZKExtension {
    fn module(&self) -> CellModule {
        CellModule::new(1, "ZK", &LogLevel::Info)
    }

    fn on_init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let zkp: ZKP = ZKP::new();
        self.command = prove_cmd(Arc::new(zkp));
        Ok(())
    }

    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }

    fn commands(&mut self) -> Option<Vec<Command<'static>>> {
        let mut ret = Vec::new();
        ret.push(self.command.clone());
        Some(ret)
    }
}

pub struct ZKExtensionFactory {}

impl ExtensionFactory for ZKExtensionFactory {
    fn build_extension(
        &self,
        compoents: Vec<Arc<Box<dyn Any>>>,
    ) -> Option<Arc<RefCell<dyn NodeExtension>>> {
        Some(Arc::new(RefCell::new(ZKExtension {
            command: Default::default(),
        })))
    }
}

fn setup() -> (ProvingKey<Bn254>, ThreadRng) {
    let mut rng = thread_rng();
    let cfg = CircomConfig::<Bn254>::new(
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.wasm",
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.r1cs",
    )
    .unwrap();

    // Insert our public inputs as key value pairs
    let mut builder = CircomBuilder::new(cfg);
    let cir = builder.build().unwrap();
    (
        generate_random_parameters::<Bn254, _, _>(cir, &mut rng).unwrap(),
        rng.clone(),
    )
}

fn prove_cmd(zkp: Arc<ZKP>) -> Command<'static> {
    Command::default()
        .with_executor(Arc::new(ClosureFunc::new(Arc::new(|ctx, v| {
            cinfo!(ModuleEnumsStruct::ZK, "receive prove request");
            let req = ctx.get_request();
            let any = req.as_any();
            let mut actual = any.downcast_ref::<HttpRequest>();
            let mut a: BigInt = BigInt::default();
            let mut b: BigInt = BigInt::default();
            match actual {
                Some(v) => {
                    let req = &v.request;
                    let body = req.body();
                    match "1".parse::<BigInt>() {
                        Ok(aa) => {
                            a = aa;
                        }
                        _ => {}
                    }
                    match "2".parse::<BigInt>() {
                        Ok(bb) => {
                            b = bb;
                        }
                        _ => {}
                    }
                    cinfo!(ModuleEnumsStruct::ZK, "uri:{}", req.uri().to_string());
                }
                None => {}
            }
            // let prove_resp = generate_prove(zkp.zkp_config.clone(), a, b);
            let mut resp = ContextResponseWrapper::default()
                .with_body(Bytes::from("aaa"))
                .with_status(ProtocolStatus::SUCCESS);
            // match prove_resp {
            //     Ok(ret) => {
            //
            //     }
            //     Err(e) => {
            //         resp = resp.with_status(ProtocolStatus::FAIL);
            //     }
            // }

            ctx.response(resp);
        }))))
        .with_protocol_id("/prove")
        .with_run_type(runTypeHttpPost)
}

pub struct ZKP {
    pub zkp_config: CircomConfig<Bn254>,
    pub zkp_params: ProvingKey<Bn254>,
}

impl Clone for ZKP {
    fn clone(&self) -> Self {
        ZKP {
            zkp_config: self.zkp_config.clone(),
            zkp_params: self.zkp_params.clone(),
        }
    }
}

pub fn generate_prove(zk: CircomConfig<Bn254>, a: BigInt, b: BigInt) -> CellResult<Proof<Bn254>> {
    let mut builder = CircomBuilder::new(zk.clone());
    builder.push_input("a", a);
    builder.push_input("b", b);
    let circom = builder.setup();
    let inputs = circom.get_public_inputs().unwrap();
    let mut rng = thread_rng();
    let params = generate_random_parameters::<Bn254, _, _>(circom, &mut rng).unwrap();
    let circom = builder.build().unwrap();
    let mut rng = thread_rng();
    let ret = prove(circom, &params, &mut rng);
    match ret {
        Err(e) => Err(CellError::from(ErrorEnumsStruct::IO_ERROR)),
        Ok(v) => Ok(v),
    }
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
        let cir = builder.build().unwrap();
        let params = generate_random_parameters::<Bn254, _, _>(cir, &mut rng).unwrap();
        ZKP {
            zkp_config: cfg.clone(),
            zkp_params: params,
        }
    }
}

#[test]
fn groth16_proof2() {
    // Load the WASM and R1CS for witness and proof generation
    let cfg = CircomConfig::<Bn254>::new(
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.wasm",
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.r1cs",
    )
    .unwrap();

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
