use core::cell::RefCell;
use core::iter::Map;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use clap::{App, Arg, arg, ArgMatches};
use shaku::{module, Component, Interface, HasComponent};
use stopwatch::Stopwatch;
use tokio::runtime::Runtime;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};


module_enums!(
        (EXTENSION,1,&logsdk::common::LogLevel::Info);
    );

// pub trait ExtensionManagerTrait: Interface {}

// #[derive(Component)]
// #[shaku(interface = ExtensionManagerTrait)]
pub struct ExtensionManager {
    extension: Vec<Arc<RefCell<dyn NodeExtension>>>,
    ctx: Arc<RefCell<NodeContext>>,

    short_ops: HashSet<String>,
    long_ops: HashSet<String>,
}

impl Default for ExtensionManager {
    fn default() -> Self {
        ExtensionManager {
            extension: vec![],
            ctx: Arc::new(RefCell::new(NodeContext::default())),
            short_ops: Default::default(),
            long_ops: Default::default(),
        }
    }
}

// impl ExtensionManagerTrait for ExtensionManager {}

impl ExtensionManager {
    pub fn add_extension(&mut self, e: Arc<RefCell<dyn NodeExtension>>) {
        self.extension.push(e);
    }
    pub fn init_command_line(&mut self, args: Vec<String>) -> CellResult<()> {
        let mut i = 0;
        let mut app = App::new("rust-cell").author("itsfunny");
        while i != self.extension.len() {
            let v = self.extension.get_mut(i).unwrap();
            if let Some(opt) = v.clone().borrow_mut().get_options() {
                for o in opt {
                    if self.has_arg(o.clone()) {
                        return Err(CellError::from(ErrorEnumsStruct::DUPLICATE_OPTION));
                    }
                    let long = o.clone().get_long();
                    match long {
                        Some(v) => {
                            self.long_ops.insert(String::from(v));
                        }
                        None => {}
                    }
                    let short = o.clone().get_short();
                    match short {
                        Some(v) => {
                            self.short_ops.insert(String::from(v));
                        }
                        None => {}
                    }
                    app = app.arg(o.clone());
                }
            }
            i += 1;
        }
        self.ctx.clone().borrow_mut().set_matchers(app.get_matches_from(args));
        Ok(())
    }
    fn has_arg(&self, arg: Arg) -> bool {
        self.short_ops.contains(&String::from(arg.get_name())) || self.long_ops.contains(&String::from(arg.get_name()))
    }
    pub fn on_init(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let e = self.extension.get_mut(i).unwrap();
            e.clone().borrow_mut().init(self.ctx.clone())?;
            i += 1;
        }
        Ok(())
    }
    pub fn on_start(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let e = self.extension.get_mut(i).unwrap();
            e.clone().borrow_mut().start(self.ctx.clone())?;
            i += 1;
        }
        Ok(())
    }
    pub fn on_ready(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let e = self.extension.get_mut(i).unwrap();
            e.clone().borrow_mut().ready(self.ctx.clone())?;
            i += 1;
        }
        Ok(())
    }
    pub fn on_close(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let e = self.extension.get_mut(i).unwrap();
            e.clone().borrow_mut().close(self.ctx.clone())?;
            i += 1;
        }
        Ok(())
    }
}


pub struct NodeContext {
    pub tokio_runtime: Runtime,
    pub matchers: ArgMatches,
}

impl NodeContext {
    pub fn set_matchers(&mut self, m: ArgMatches) {
        self.matchers = m
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        NodeContext {
            tokio_runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build().expect("failed"),
            matchers: Default::default(),
        }
    }
}


pub trait NodeExtension {
    fn module(&self) -> CellModule;
    fn get_options<'a>(&self) -> Option<Vec<Arg<'a>>> {
        None
    }
    fn init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_init(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} init end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
    fn start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_start(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} start end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
    fn ready(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_ready(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} ready end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_ready(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
    fn close(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        let wh = Stopwatch::start_new();
        self.on_close(ctx.clone())?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{} close end,cost:{}s",self.module().get_name(),wh.elapsed().as_secs());
        Ok(())
    }
    fn on_close(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
}

// pub struct ExtensionProxy {
//     extension: Arc<RefCell<dyn NodeExtension>>,
// }
//
// impl ExtensionProxy {
//     pub fn new(extension: Arc<RefCell<dyn NodeExtension>>) -> Self {
//         Self { extension }
//     }
// }
//
//
// impl NodeExtension for ExtensionProxy {
//     fn module(&self) -> CellModule {
//         // let e: &RefCell<dyn NodeExtension> = self.extension.borrow();
//         let e = self.extension.into_inner().borrow();
//         e.module()
//     }
// }

//////
pub struct DemoExtension {}

impl NodeExtension for DemoExtension {
    fn module(&self) -> CellModule {
        CellModule::new(1, "asd", &LogLevel::Info)
    }
    fn get_options<'a>(&self) -> Option<Vec<Arg<'a>>> {
        Some(vec![Arg::default().name("demo").long("long").required(false),
                  Arg::default().name("demo2").long("long2").required(false)])
    }
}


#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::Arc;
    use std::{thread, time};
    use stopwatch::Stopwatch;
    use crate::extension::{DemoExtension, ExtensionManager, NodeContext, NodeExtension};


    #[test]
    fn test_extension() {
        // let demo = DemoExtension {};
        // let mut proxy = ExtensionProxy::new(Arc::new(RefCell::new(demo)));
        // println!("{}", proxy.module());
        // proxy.start(Arc::new(NodeContext::default()));
    }

    #[test]
    fn test_extension_manager() {
        let mut m = ExtensionManager::default();
        let demo = DemoExtension {};
        m.add_extension(Arc::new(RefCell::new(demo)));
        m.on_init();
    }

    #[test]
    fn test_init_command_line() {
        let mut m = ExtensionManager::default();
        let demo = DemoExtension {};
        m.add_extension(Arc::new(RefCell::new(demo)));
        let mut args = Vec::<String>::new();
        m.init_command_line(args);
    }
}