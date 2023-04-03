use bytes::Bytes;
use cell_core::application::CellApplication;
use cell_core::cerror::CellResult;
use cell_core::command::{ClosureFunc, Command};
use cell_core::constants::ProtocolStatus;
use cell_core::core::runTypeHttp;
use cell_core::extension::{ExtensionFactory, NodeContext, NodeExtension};
use cell_core::wrapper::ContextResponseWrapper;
use cellhttp::extension::HttpExtensionFactory;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use std::any::Any;
use std::cell::RefCell;
use std::sync::Arc;

pub struct DemoExtensionFactory {}

pub struct DemoExtension {}

impl ExtensionFactory for DemoExtensionFactory {
    fn build_extension(
        &self,
        compoents: Vec<Arc<Box<dyn Any>>>,
    ) -> Option<Arc<RefCell<dyn NodeExtension>>> {
        Some(Arc::new(RefCell::new(DemoExtension {})))
    }
}

impl NodeExtension for DemoExtension {
    fn module(&self) -> CellModule {
        CellModule::new(1, "demo", &LogLevel::Info)
    }

    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let rt = ctx.clone().borrow().tokio_runtime.clone();
        rt.spawn(async {});
        Ok(())
    }

    fn commands(&mut self) -> Option<Vec<Command<'static>>> {
        let mut ret: Vec<Command> = Vec::new();

        let cmd = Command::default()
            .with_run_type(runTypeHttp)
            .with_protocol_id("/demo")
            .with_executor(Arc::new(ClosureFunc::new(Arc::new(|ctx, v| {
                let resp = ContextResponseWrapper::default()
                    .with_body(Bytes::from("asd"))
                    .with_status(ProtocolStatus::SUCCESS);
                ctx.response(resp);
            }))));
        ret.push(cmd);
        Some(ret)
    }
}

fn main() {
    let mut factories: Vec<Box<dyn ExtensionFactory>> = Vec::new();
    factories.push(Box::new(HttpExtensionFactory {}));
    factories.push(Box::new(DemoExtensionFactory {}));
    let mut app = CellApplication::new(factories);
    app.run(vec![]);
}
