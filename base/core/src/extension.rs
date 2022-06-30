use core::cell::RefCell;
use core::future::Future;
use core::iter::Map;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::sync::Arc;
use clap::{App, Arg, arg, ArgMatches};
use flo_stream::{MessagePublisher, Publisher, Subscriber};
use futures::future::ok;
use futures::StreamExt;
use shaku::{module, Component, Interface, HasComponent};
use stopwatch::Stopwatch;
use tokio::runtime::Runtime;
use tokio::select;
use tokio::task::JoinHandle;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::banner::{BLESS, INIT, START};
use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};
use crate::event::{ApplicationCloseEvent, ApplicationEnvironmentPreparedEvent, ApplicationInitEvent, ApplicationReadyEvent, ApplicationStartedEvent, Event};


module_enums!(
        (EXTENSION,1,&logsdk::common::LogLevel::Info);
    );


pub const step_0: u8 = 1 << 0;
pub const step_1: u8 = 1 << 1;
pub const step_2: u8 = 1 << 2;
pub const step_3: u8 = 1 << 3;
pub const step_4: u8 = 1 << 3;
// pub trait ExtensionManagerTrait: Interface {}

// #[derive(Component)]
// #[shaku(interface = ExtensionManagerTrait)]
pub struct ExtensionManager {
    extension: Vec<Arc<RefCell<dyn NodeExtension>>>,
    ctx: Arc<RefCell<NodeContext>>,

    short_ops: HashSet<String>,
    long_ops: HashSet<String>,

    close_notify: tokio::sync::mpsc::Receiver<u8>,
    subscriber: Subscriber<Arc<dyn Event>>,
    step: u8,
}

unsafe impl Send for ExtensionManager {}

unsafe impl Sync for ExtensionManager {}

pub struct ExtensionManagerBuilder {
    tokio_runtime: Option<Arc<Runtime>>,
    close_notifyc: Option<tokio::sync::mpsc::Receiver<u8>>,
    extensions: Vec<Arc<RefCell<dyn NodeExtension>>>,
    publisher: Option<Arc<RefCell<Publisher<Arc<dyn Event>>>>>,
}

impl Default for ExtensionManagerBuilder {
    fn default() -> Self {
        ExtensionManagerBuilder {
            tokio_runtime: None,
            close_notifyc: None,
            extensions: Vec::new(),
            publisher: None,
        }
    }
}

impl ExtensionManagerBuilder {
    pub fn with_tokio(mut self, r: Arc<Runtime>) -> Self {
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
    pub fn with_subscriber(mut self, sub: Arc<RefCell<Publisher<Arc<dyn Event>>>>) -> Self {
        self.publisher = Some(sub);
        self
    }
    pub fn build(self) -> ExtensionManager {
        let mut ctx = NodeContext::default();
        if let Some(v) = self.tokio_runtime {
            ctx.set_tokio(v);
        }
        match self.close_notifyc {
            None => {
                panic!("close notify cant be null")
            }
            _ => {}
        }
        let rx = self.close_notifyc.unwrap();
        let mut publisher = self.publisher.unwrap();
        let sub = publisher.clone().borrow_mut().subscribe();
        ExtensionManager {
            extension: self.extensions,
            ctx: Arc::new(RefCell::new(ctx)),
            short_ops: Default::default(),
            long_ops: Default::default(),
            close_notify: rx,
            subscriber: sub,
            step: 0,
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
                        return Err(CellError::from(ErrorEnumsStruct::DUPLICATE_STEP));
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
        self.ctx.clone().borrow().tokio_runtime.clone().spawn(async move {
            self.async_start().await
        });
    }
    pub fn get_ctx(&self) -> Arc<RefCell<NodeContext>> {
        self.ctx.clone()
    }
    async fn async_start(&mut self) {
        cinfo!(ModuleEnumsStruct::EXTENSION,"extension start");

        loop {
            tokio::select! {
                _=self.close_notify.recv()=>{
                    cinfo!(ModuleEnumsStruct::EXTENSION,"extension received exit signal,closing extensions");
                    self.on_close();
                    break
                },
                msg=self.subscriber.next()=>{
                    let res=self.handle_msg(msg.unwrap());
                    match res {
                        Err(e)=>{
                            cerror!(ModuleEnumsStruct::EXTENSION,"handle msg failed:{}",e);
                        },
                        _=>{}
                    }
                },
            }
        }
    }
    fn handle_msg(&mut self, msg: Arc<dyn Event>) -> CellResult<()> {
        cinfo!(ModuleEnumsStruct::EXTENSION,"receive msg:{}",msg);
        let any = msg.as_any();
        let mut res = Err(CellError::from("unknown event"));
        {
            let actual = any.downcast_ref::<ApplicationEnvironmentPreparedEvent>();
            match actual {
                Some(v) => {
                    res = self.on_prepare(v.args.clone());
                }
                None => {}
            }
        }

        {
            let actual = any.downcast_ref::<ApplicationInitEvent>();
            match actual {
                Some(v) => {
                    res = self.on_init();
                }
                None => {}
            }
        }

        {
            let actual = any.downcast_ref::<ApplicationStartedEvent>();
            match actual {
                Some(v) => {
                    res = self.on_start();
                }
                None => {}
            }
        }


        {
            let actual = any.downcast_ref::<ApplicationReadyEvent>();
            match actual {
                Some(v) => {
                    res = self.on_ready();
                }
                None => {}
            }
        }

        {
            let actual = any.downcast_ref::<ApplicationCloseEvent>();
            match actual {
                Some(v) => {
                    res = self.on_close();
                }
                None => {}
            }
        }
        res
    }

    pub fn on_prepare(&mut self, args: Vec<String>) -> CellResult<()> {
        self.verify_step(step_0)?;
        self.init_command_line(args)?;
        self.step = step_0;
        Ok(())
    }

    pub fn on_init(&mut self) -> CellResult<()> {
        self.verify_step(step_1)?;

        cinfo!(ModuleEnumsStruct::EXTENSION,"{}",INIT);
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
        self.step = step_1;
        Ok(())
    }

    pub fn on_start(&mut self) -> CellResult<()> {
        self.verify_step(step_2)?;
        let mut i = 0;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{}",START);
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
        self.step = step_2;
        Ok(())
    }

    pub fn on_ready(&mut self) -> CellResult<()> {
        self.verify_step(step_3)?;

        let mut i = 0;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{}",BLESS);
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
        self.step = step_3;
        Ok(())
    }

    pub fn on_close(&mut self) -> CellResult<()> {
        self.verify_step(step_4)?;

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
        self.step = step_4;
        Ok(())
    }

    pub fn verify_step(&mut self, to_verify: u8) -> CellResult<()> {
        if self.step == to_verify {
            return Err(CellError::from(ErrorEnumsStruct::DUPLICATE_STEP));
        } else if self.step > to_verify {
            return Err(CellError::from(ErrorEnumsStruct::ILLEGAL_STEP));
        }

        if to_verify != step_0 {
            let last_step = to_verify >> 1;
            if self.step != last_step {
                return Err(CellError::from(ErrorEnumsStruct::ILLEGAL_STEP));
            }
        }

        Ok(())
    }
}

fn async_start_manager(m: ExtensionManager) {}

pub struct NodeContext {
    pub tokio_runtime: Arc<Runtime>,
    pub matchers: ArgMatches,
    // TODO ,node context need pubsub component
}

impl NodeContext {
    pub fn set_matchers(&mut self, m: ArgMatches) {
        self.matchers = m
    }

    pub fn set_tokio(&mut self, m: Arc<Runtime>) {
        self.tokio_runtime = m
    }
}

impl Default for NodeContext {
    fn default() -> Self {
        NodeContext {
            tokio_runtime: Arc::new(tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build().expect("failed")),
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
    use flo_stream::{MessagePublisher, Publisher, Subscriber};
    use futures::StreamExt;
    use stopwatch::Stopwatch;
    use tokio::runtime::Runtime;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::Sender;
    use crate::event::{ApplicationReadyEvent, Event};
    use crate::extension::{DemoExtension, ExtensionManager, ExtensionManagerBuilder, NodeContext, NodeExtension};


    #[test]
    fn test_extension() {
        // let demo = DemoExtension {};
        // let mut proxy = ExtensionProxy::new(Arc::new(RefCell::new(demo)));
        // println!("{}", proxy.module());
        // proxy.start(Arc::new(NodeContext::default()));
    }

    fn create_builder() -> (ExtensionManager, Arc<RefCell<Publisher<Arc<dyn Event>>>>, Arc<Runtime>, Sender<u8>) {
        let demo = DemoExtension {};
        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
        let (tx, rx) = mpsc::channel::<u8>(1);
        let mut publ = Publisher::<Arc<dyn Event>>::new(10);
        let arc_pub = Arc::new(RefCell::new(Publisher::new(10)));
        (ExtensionManagerBuilder::default()
             .with_extension(Arc::new(RefCell::new(demo)))
             .with_tokio(runtime.clone())
             .with_close_notifyc(rx)
             .with_subscriber(arc_pub.clone())
             .build(), arc_pub.clone(), runtime.clone(), tx)
    }

    #[test]
    fn test_extension_manager() {
        let mut m = create_builder();
        m.0.on_init();
    }

    #[test]
    fn test_init_command_line() {
        let mut m = create_builder();
        let mut args = Vec::<String>::new();
        m.0.init_command_line(args);
    }

    #[test]
    fn test_start() {
        let mut m = create_builder();
        let am = RefCell::new(m.0);
        am.into_inner().start();
        let a = async {
            thread::sleep(Duration::from_secs(1000));
        };
        m.2.clone().block_on(async move {
            thread::sleep(Duration::from_secs(3));
            let msg = ApplicationReadyEvent::new();
            m.1.clone().borrow_mut().publish(Arc::new(msg)).await;
            thread::sleep(Duration::from_secs(10000));
        });
    }
}