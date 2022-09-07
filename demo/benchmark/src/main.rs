use std::any::Any;
use std::cell::RefCell;
use std::fmt::format;
use std::sync::Arc;
use bytes::Bytes;
use cell_core::application::CellApplication;
use cell_core::command::{ClosureFunc, Command};
use cell_core::constants::{EnumsProtocolStatus, ProtocolStatus};
use cell_core::core::{ProtocolID, runTypeHttp};
use cell_core::extension::{ExtensionFactory, NodeExtension};
use cell_core::wrapper::ContextResponseWrapper;
use cellhttp::extension::HttpExtensionFactory;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use pipeline2::pipeline2::ClosureExecutor;

fn create_cmd(path: &'static str) -> Command<'static> {
    let ret = Command::default()
        .with_executor(Arc::new(ClosureFunc::new(Arc::new(|ctx, v| {
            let resp = ContextResponseWrapper::default()
                .with_body(Bytes::from(path.clone().as_bytes()))
                .with_status(ProtocolStatus::SUCCESS);
            ctx.response(resp);
        }))))
        .with_protocol_id(path.clone())
        .with_run_type(runTypeHttp);
    ret
}

pub struct BenchMarkExtension {}

pub struct BenchMarkFactory {}

impl ExtensionFactory for BenchMarkFactory {
    fn build_extension(&self, compoents: Vec<Arc<Box<dyn Any>>>) -> Option<Arc<RefCell<dyn NodeExtension>>> {
        Some(Arc::new(RefCell::new(BenchMarkExtension {})))
    }
}

impl NodeExtension for BenchMarkExtension {
    fn module(&self) -> CellModule {
        CellModule::new(1, "BENCH_MARK", &LogLevel::Info)
    }

    fn commands(&mut self) -> Option<Vec<Command<'static>>> {
        let mut ret: Vec<Command> = Vec::new();
        for i in 0..100 {
            let path = format!("/bench/{}", i);
            let cmd = create_cmd( );
            ret.push(cmd);
        }

        Some(ret)
    }
}

fn main() {
    let mut factories: Vec<Box<dyn ExtensionFactory>> = Vec::new();
    factories.push(Box::new(HttpExtensionFactory {}));
    factories.push(Box::new(BenchMarkFactory {}));
    let mut app = CellApplication::new(factories);
    app.run(vec![]);
}
