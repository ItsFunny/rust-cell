use core::cell::RefCell;
use std::sync::Arc;
use shaku::{module, Component, Interface, HasComponent};
use stopwatch::Stopwatch;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::cerror::CellResult;

pub trait ExtensionManagerTrait: Interface {}

#[derive(Component)]
#[shaku(interface = ExtensionManagerTrait)]
pub struct ExtensionManager {}

impl ExtensionManagerTrait for ExtensionManager {}


pub struct NodeContext {}

module_enums!(
        (EXTENSION,1,&logsdk::common::LogLevel::Info);
    );

pub trait NodeExtension {
    fn module(&self) -> CellModule;
    fn init(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_init(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} init end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_init(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        Ok(())
    }
    fn start(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_start(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} start end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_start(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        Ok(())
    }
    fn ready(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_ready(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} ready end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_ready(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        Ok(())
    }
    fn close(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_close(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} close end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_close(&mut self, ctx: Arc<NodeContext>) -> CellResult<()> {
        Ok(())
    }
}

pub struct ExtensionProxy {
    extension: Arc<RefCell<dyn NodeExtension>>,
}

impl ExtensionProxy {
    pub fn new(extension: Arc<RefCell<dyn NodeExtension>>) -> Self {
        Self { extension }
    }
}


impl NodeExtension for ExtensionProxy {
    fn module(&self) -> CellModule {
        self.extension.borrow().module()
    }
}

//////
pub struct DemoExtension {}

impl NodeExtension for DemoExtension {
    fn module(&self) -> CellModule {
        CellModule::new(1, "asd", &LogLevel::Info)
    }
}


#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::Arc;
    use std::{thread, time};
    use stopwatch::Stopwatch;
    use crate::extension::{DemoExtension, ExtensionProxy, NodeExtension};


    #[test]
    fn test_extension() {
        let demo = DemoExtension {};
        let proxy = ExtensionProxy::new(Arc::new(RefCell::new(demo)));
        println!("{}", proxy.module())
    }
}