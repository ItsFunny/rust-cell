use core::cell::RefCell;
use core::future::Future;
use core::iter::Map;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;
use clap::{App, Arg, arg, ArgMatches};
use shaku::{module, Component, Interface, HasComponent};
use stopwatch::Stopwatch;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::task::JoinHandle;
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

    close_notify: tokio::sync::mpsc::Receiver<u8>,
}

unsafe impl Send for ExtensionManager {}

unsafe impl Sync for ExtensionManager {}

pub struct ExtensionManagerBuilder {
    tokio_runtime: Option<Runtime>,
    close_notifyc: Option<tokio::sync::mpsc::Receiver<u8>>,
    extensions: Vec<Arc<RefCell<dyn NodeExtension>>>,
}

impl Default for ExtensionManagerBuilder {
    fn default() -> Self {
        ExtensionManagerBuilder {
            tokio_runtime: None,
            close_notifyc: None,
            extensions: Vec::new(),
        }
    }
}

impl ExtensionManagerBuilder {
    pub fn with_tokio(mut self, r: Runtime) -> Self {
        self.tokio_runtime = Some(r);
        self
    }
    pub fn with_extension(mut self, e: Arc<RefCell<dyn NodeExtension>>) -> Self {
        self.extensions.push(e);
        self
    }
    pub fn with_close_notifyc(mut self, c: tokio::sync::mpsc::Receiver<u8>) -> Self {
        self.close_notifyc = Some(c);
        self
    }
    pub fn build(self) -> ExtensionManager {
        let mut ctx = NodeContext::default();
        if let Some(v) = self.tokio_runtime {
            ctx.set_tokio(v);
        }
        let (_, mut rx) = tokio::sync::mpsc::channel(1);
        if let Some(v) = self.close_notifyc {
            rx = v;
        }
        ExtensionManager {
            extension: self.extensions,
            ctx: Arc::new(RefCell::new(ctx)),
            short_ops: Default::default(),
            long_ops: Default::default(),
            close_notify: rx,
        }
    }
}

// impl ExtensionManagerTrait for ExtensionManager {}

impl ExtensionManager {
    pub fn set_close_notify(&mut self, c: tokio::sync::mpsc::Receiver<u8>) {
        mem::replace(&mut self.close_notify, c);
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
        let matchers = app.get_matches_from(args);
        self.ctx.clone().borrow_mut().set_matchers(matchers);
        Ok(())
    }

    fn has_arg(&self, arg: Arg) -> bool {
        self.short_ops.contains(&String::from(arg.get_name())) || self.long_ops.contains(&String::from(arg.get_name()))
    }

    pub fn start(mut self) {
        // self.ctx.clone().borrow().tokio_runtime.spawn(self.async_start());
        self.ctx.clone().borrow().tokio_runtime.spawn(async move {
            self.async_start().await
        });
    }
    async fn async_start(&mut self) {
        cinfo!(ModuleEnumsStruct::EXTENSION,"extension start");
        loop {
            tokio::select! {
                _=self.close_notify.recv()=>{
                    cinfo!(ModuleEnumsStruct::EXTENSION,"extension received exit signal,closing extensions");
                    self.on_close();
                },
            }
        }
    }

    // async fn async_start(mut e: ExtensionManager) {
    //     cinfo!(ModuleEnumsStruct::EXTENSION,"extension start");
    //     loop {
    //         tokio::select! {
    //             _=e.close_notify.recv()=>{
    //                 cinfo!(ModuleEnumsStruct::EXTENSION,"extension received exit signal,closing extensions");
    //                 e.on_close();
    //             },
    //         }
    //     }
    // }

    pub fn on_init(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let e = self.extension.get_mut(i).unwrap();
            let wh = Stopwatch::start_new();
            let res = e.clone().borrow_mut().init(self.ctx.clone());
            match res {
                Err(err) => {
                    if e.clone().borrow_mut().required() {
                        panic!("{}", err.get_msg())
                    }
                    cerror!(ModuleEnumsStruct::EXTENSION,"init extension [{}] failed ,err:{}"
                ,e.clone().borrow_mut().module().get_name(),err.get_msg());
                }
                Ok(..) => {}
            }
            cinfo!(ModuleEnumsStruct::EXTENSION,"init extension [{}] successfully ,cost:{}"
                ,e.clone().borrow_mut().module().get_name(),wh.elapsed().as_secs());
            i += 1;
        }
        Ok(())
    }
    pub fn on_start(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let wh = Stopwatch::start_new();
            let e = self.extension.get_mut(i).unwrap();
            let res = e.clone().borrow_mut().start(self.ctx.clone());
            match res {
                Err(err) => {
                    if e.clone().borrow_mut().required() {
                        panic!("{}", err.get_msg())
                    }
                    cerror!(ModuleEnumsStruct::EXTENSION,"start extension [{}] failed ,err:{}"
                ,e.clone().borrow_mut().module().get_name(),err.get_msg());
                }
                Ok(..) => {}
            }
            cinfo!(ModuleEnumsStruct::EXTENSION,"start extension [{}] successfully ,cost:{}"
                ,e.clone().borrow_mut().module().get_name(),wh.elapsed().as_secs());
            i += 1;
        }
        Ok(())
    }
    pub fn on_ready(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let wh = Stopwatch::start_new();
            let e = self.extension.get_mut(i).unwrap();
            let res = e.clone().borrow_mut().ready(self.ctx.clone());
            match res {
                Err(err) => {
                    if e.clone().borrow_mut().required() {
                        panic!("{}", err.get_msg())
                    }
                    cerror!(ModuleEnumsStruct::EXTENSION,"ready extension [{}] failed ,err:{}"
                ,e.clone().borrow_mut().module().get_name(),err.get_msg());
                }
                Ok(..) => {}
            }
            cinfo!(ModuleEnumsStruct::EXTENSION,"ready extension [{}] successfully ,cost:{}"
                ,e.clone().borrow_mut().module().get_name(),wh.elapsed().as_secs());
            i += 1;
        }
        Ok(())
    }
    pub fn on_close(&mut self) -> CellResult<()> {
        let mut i = 0;
        while i != self.extension.len() {
            let wh = Stopwatch::start_new();
            let e = self.extension.get_mut(i).unwrap();
            let res = e.clone().borrow_mut().close(self.ctx.clone());
            match res {
                Err(err) => {
                    if e.clone().borrow_mut().required() {
                        panic!("{}", err.get_msg())
                    }
                    cerror!(ModuleEnumsStruct::EXTENSION,"close extension [{}] failed ,err:{}"
                ,e.clone().borrow_mut().module().get_name(),err.get_msg());
                }
                Ok(..) => {}
            }
            cinfo!(ModuleEnumsStruct::EXTENSION,"close extension [{}] successfully ,cost:{}"
                ,e.clone().borrow_mut().module().get_name(),wh.elapsed().as_secs());
            i += 1;
        }
        Ok(())
    }
}

fn async_start_manager(m: ExtensionManager) {}

pub struct NodeContext {
    pub tokio_runtime: Runtime,
    pub matchers: ArgMatches,
}

impl NodeContext {
    pub fn set_matchers(&mut self, m: ArgMatches) {
        self.matchers = m
    }

    pub fn set_tokio(&mut self, m: Runtime) {
        self.tokio_runtime = m
    }

    // pub fn spawn_task(&self, future: F) -> JoinHandle<F::Output>
    //     where
    //         F: Future + Send + 'static,
    //         F::Output: Send + 'static {
    //     self.tokio_runtime.spawn(future)
    // }
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
    fn required(&self) -> bool {
        true
    }

    fn get_options<'a>(&self) -> Option<Vec<Arg<'a>>> {
        None
    }
    fn init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        self.on_init(ctx.clone())
    }
    fn on_init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
    fn start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        self.on_start(ctx.clone())
    }
    fn on_start(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
    fn ready(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        self.on_ready(ctx.clone())
    }
    fn on_ready(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        Ok(())
    }
    fn close(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        self.on_close(ctx.clone())
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
    use std::time::Duration;
    use stopwatch::Stopwatch;
    use crate::extension::{DemoExtension, ExtensionManager, ExtensionManagerBuilder, NodeContext, NodeExtension};


    #[test]
    fn test_extension() {
        // let demo = DemoExtension {};
        // let mut proxy = ExtensionProxy::new(Arc::new(RefCell::new(demo)));
        // println!("{}", proxy.module());
        // proxy.start(Arc::new(NodeContext::default()));
    }

    #[test]
    fn test_extension_manager() {
        let demo = DemoExtension {};
        let mut m = ExtensionManagerBuilder::default()
            .with_extension(Arc::new(RefCell::new(demo)))
            .build();
        m.on_init();
    }

    #[test]
    fn test_init_command_line() {
        let demo = DemoExtension {};
        let mut m = ExtensionManagerBuilder::default()
            .with_extension(Arc::new(RefCell::new(demo)))
            .build();
        let mut args = Vec::<String>::new();
        m.init_command_line(args);
    }

    #[test]
    fn test_start() {
        let demo = DemoExtension {};
        let mut m = ExtensionManagerBuilder::default()
            .with_extension(Arc::new(RefCell::new(demo)))
            .build();
        let am = Arc::new(RefCell::new(m));
        // let b = am.clone();
        // let inner = b.into_inner();
        // let a = async {
        //     thread::sleep(Duration::from_secs(1000));
        // };

        // m.ctx.clone().borrow().tokio_runtime.block_on(a);
    }
}