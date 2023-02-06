mod request;

extern crate core;

use std::any::Any;
use std::cell::RefCell;
use color_eyre::Result;
use std::fmt::Error;
use std::sync::Arc;
use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomCircuit, CircomConfig};
use ark_groth16::{generate_random_parameters, prepare_verifying_key, verify_proof, create_random_proof as prove, ProvingKey};
use ark_relations::r1cs::{
    ConstraintSynthesizer, ConstraintSystem, OptimizationGoal,
    SynthesisError, SynthesisMode,
};
use ark_std::rand::rngs::ThreadRng;
use ark_std::rand::thread_rng;
use color_eyre::eyre::ErrReport;
use color_eyre::owo_colors::OwoColorize;
use cell_core::application::CellApplication;
use cell_core::cerror::CellResult;
use cell_core::command::{ClosureFunc, Command};
use cell_core::core::runTypeHttp;
use cell_core::extension::{ExtensionFactory, NodeContext, NodeExtension};
use cellhttp::extension::HttpExtensionFactory;
use logsdk::{cinfo, module_enums};
use logsdk::common::LogLevel;
use logsdk::module::CellModule;

module_enums!(
        (ZK,1,&logsdk::common::LogLevel::Info);
    );


fn main() {
    let mut factories: Vec<Box<dyn ExtensionFactory>> = Vec::new();
    factories.push(Box::new(HttpExtensionFactory {}));
    factories.push(Box::new(ZKExtensionFactory {}));
    let mut app = CellApplication::new(factories);
    app.run(vec![]);
}


pub struct ZKExtension {
    zkp: Option<ZKP>,
}

impl NodeExtension for ZKExtension {
    fn module(&self) -> CellModule {
        CellModule::new(1, "ZK", &LogLevel::Info)
    }

    fn on_init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let zkp: ZKP = ZKP::new();
        self.zkp = Some(zkp);
        Ok(())
    }

    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }


    fn commands(&mut self) -> Option<Vec<Command<'static>>> {
        let mut ret = Vec::new();
        let app = self.zkp.as_mut().unwrap();
        ret.push(prove_cmd(app.clone()));
        Some(ret)
    }
}

pub struct ZKExtensionFactory {}

impl ExtensionFactory for ZKExtensionFactory {
    fn build_extension(&self, compoents: Vec<Arc<Box<dyn Any>>>) -> Option<Arc<RefCell<dyn NodeExtension>>> {
        Some(Arc::new(RefCell::new(ZKExtension { zkp: None })))
    }
}

fn setup() -> (ProvingKey<Bn254>, ThreadRng) {
    let mut rng = thread_rng();
    let cfg = CircomConfig::<Bn254>::new(
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.wasm",
        "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.r1cs",
    ).unwrap();

// Insert our public inputs as key value pairs
    let mut builder = CircomBuilder::new(cfg);
    let cir = builder.build().unwrap();
    (generate_random_parameters::<Bn254, _, _>(cir, &mut rng).unwrap(), rng.clone())
}

fn prove_cmd(zkp: ZKP) -> Command<'static> {
    Command::default()
        .with_executor(Arc::new(ClosureFunc::new(Arc::new(|ctx, v| {
            cinfo!(ModuleEnumsStruct::ZK,"receive prove request")
        }))))
        .with_protocol_id("/prove")
        .with_run_type(runTypeHttp)
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

impl ZKP {
    pub fn new() -> ZKP {
        let mut rng = thread_rng();
        let cfg = CircomConfig::<Bn254>::new(
            "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.wasm",
            "/Users/lvcong/rust/rust-cell/demo/zk/src/test-vectors/mycircuit.r1cs",
        ).unwrap();

        let mut builder = CircomBuilder::new(cfg.clone());
        let cir = builder.build().unwrap();
        let params = generate_random_parameters::<Bn254, _, _>(cir, &mut rng).unwrap();
        ZKP { zkp_config: cfg.clone(), zkp_params: params }
    }
}


#[test]
fn groth16_proof2() {
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