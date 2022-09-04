use core::any::Any;
use core::cell::RefCell;
use core::future::Future;
use core::iter::Map;
use std::collections::{HashMap, HashSet};
use std::{mem, thread};

use derive_builder::Builder;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use clap::{App, Arg, arg, ArgMatches, command};
use crossbeam::channel::{bounded, Receiver, Select, Sender};
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
use crate::banner::{BLESS, CLOSE, INIT, START};
use crate::bus::{DefaultRegexQuery, EventBus, publish_application_events, subscribe_application_events};
use crate::cerror::{CellError, CellResult, ErrorEnumsStruct};
use crate::command::Command;
use crate::core::conv_protocol_to_string;
use crate::event::{ApplicationCloseEvent, ApplicationEnvironmentPreparedEvent, ApplicationInitEvent, ApplicationReadyEvent, ApplicationStartedEvent, CallBackEvent, Event, NextStepEvent};
use crate::module::ModuleEnumsStruct;


pub const step_0: u8 = 1 << 0;
pub const step_1: u8 = 1 << 1;
pub const step_2: u8 = 1 << 2;
pub const step_3: u8 = 1 << 3;
pub const step_4: u8 = 1 << 4;

pub const default_orderer: i32 = 0;
pub const max_orderer: i32 = i32::MIN;

const extension_manager:&'static str="extension_manager";


// pub trait  Component: Any + Clone{}

// #[derive(Component)]
// #[shaku(interface = ExtensionManagerTrait)]
pub struct ExtensionManager {
    extension: Vec<Arc<RefCell<dyn NodeExtension>>>,
    ctx: Arc<RefCell<NodeContext>>,

    short_ops: HashSet<String>,
    long_ops: HashSet<String>,

    subscriber: Arc<Receiver<Arc<Box<dyn Event>>>>,
    bus: Arc<EventBus<Box<dyn Event>>>,

    close_notify: tokio::sync::mpsc::Receiver<u8>,
    step: u8,

    components: Vec<Arc<Box<dyn Any>>>,
    commands: Vec<Command<'static>>,
}

pub trait ExtensionFactory {
    fn build_extension(&self, compoents: Vec<Arc<Box<dyn Any>>>) -> Option<Arc<RefCell<dyn NodeExtension>>> {
        None
    }
    // TODO ,maybe it should wrapped by refcell
    fn components(&self) -> Option<Vec<Arc<Box<dyn Any>>>> {
        None
    }
    fn commands(&self) -> Option<Vec<Command<'static>>> {
        None
    }
}

unsafe impl Send for ExtensionManager {}

unsafe impl Sync for ExtensionManager {}

pub struct ExtensionManagerBuilder {
    tokio_runtime: Option<Arc<Runtime>>,
    close_notifyc: Option<tokio::sync::mpsc::Receiver<u8>>,
    extensions: Vec<Arc<RefCell<dyn NodeExtension>>>,
    // publisher: Option<Sender<Arc<dyn Event>>>,
    // subscriber: Option<Receiver<Arc<dyn Event>>>,
    bus: Option<EventBus<Box<dyn Event>>>,

    components: Option<Vec<Arc<Box<dyn Any>>>>,
    commands: Option<Vec<Command<'static>>>,
}

impl Default for ExtensionManagerBuilder {
    fn default() -> Self {
        ExtensionManagerBuilder {
            tokio_runtime: None,
            close_notifyc: None,
            extensions: Vec::new(),
            components: None,
            commands: None,
            bus: None,
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
    // pub fn with_subscriber(mut self, sub: Arc<Publisher<Arc<dyn Event>>>) -> Self {
    //     self.publisher = Some(sub);
    //     self
    // }
    pub fn with_commands(mut self, value: Vec<Command<'static>>) -> Self {
        self.commands = Some(value);
        self
    }
    pub fn with_components(mut self, value: Vec<Arc<Box<dyn Any>>>) -> Self {
        self.components = Some(value);
        self
    }
    pub fn with_bus(mut self, bus: EventBus<Box<dyn Event>>) -> Self {
        self.bus = Some(bus);
        self
    }

    pub fn build(mut self) -> ExtensionManager {
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
        let mut bus = self.bus.unwrap();
        let clone_bus = bus.clone();

        let subsc =subscribe_application_events(clone_bus,extension_manager,None);

        ctx.set_bus(clone_bus.clone());

        // internal
        let mut inter_tokio = InternalTokioExtension::new();
        self.extensions.push(Arc::new(RefCell::new(inter_tokio)));

        ExtensionManager {
            extension: self.extensions,
            ctx: Arc::new(RefCell::new(ctx)),
            short_ops: Default::default(),
            long_ops: Default::default(),
            close_notify: rx,
            subscriber: subsc,
            step: 0,
            components: vec![],
            commands: vec![],
            bus: Arc::new(clone_bus.clone()),
        }
    }
}

// impl ExtensionManagerTrait for ExtensionManager {}

impl ExtensionManager {
    // pub fn get_publisher(&self) -> Arc<Publisher<Arc<dyn Event>>> {
    //     self.publisher.clone()
    // }
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
                        cerror!(ModuleEnumsStruct::EXTENSION,"duplicate option:{}",o.clone());
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

    // fn fill_ctx(&mut self) {
    //     let mut cmds = Vec::new();
    //     for c in &self.commands {
    //         cmds.push(c.clone());
    //     }
    //     self.ctx.clone().borrow_mut().set_commands(cmds);
    // }


    fn has_arg(&self, arg: Arg) -> bool {
        self.short_ops.contains(&String::from(arg.get_name())) || self.long_ops.contains(&String::from(arg.get_name()))
    }

    pub fn start(mut self) {
        // thread::spawn(move || {
        //     futures::executor::block_on(self.async_start())
        // });
        self.ctx.clone().borrow().tokio_runtime.clone().spawn(async move {
            self.async_start().await
        });
    }
    pub fn get_ctx(&self) -> Arc<RefCell<NodeContext>> {
        self.ctx.clone()
    }
    async fn async_start(&mut self) {
        cinfo!(ModuleEnumsStruct::EXTENSION,"extension start");


        let mut sel = Select::new();
        let clone_sub = self.subscriber.clone();
        sel.recv(&clone_sub);

        loop {
            let index = sel.ready();
            let res = clone_sub.try_recv();
            // If the operation turns out not to be ready, retry.
            if let Err(e) = res {
                if e.is_empty() {
                    continue;
                }
            }
            let v = res.unwrap();
            if let Err(v) = self.handle_msg(v) {
                cerror!(ModuleEnumsStruct::EXTENSION,"handle msg failed:{}",v);
            }
        }
        // tokio::select! {
        //     _=self.close_notify.recv()=>{
        //         cinfo!(ModuleEnumsStruct::EXTENSION,"extension received exit signal,closing extensions");
        //         self.on_close();
        //         break
        //     },
        //     msg=self.subscriber.next()=>{
        //         let res=self.handle_msg(msg.unwrap());
        //         match res {
        //             Err(e)=>{
        //                 cerror!(ModuleEnumsStruct::EXTENSION,"handle msg failed:{}",e);
        //             },
        //             _=>{}
        //         }
        //     },
        // }
    }

    fn handle_msg(&mut self, msg: Arc<Box<dyn Event>>) -> CellResult<()> {
        cinfo!(ModuleEnumsStruct::EXTENSION,"receive msg:{}",msg);
        let any = msg.as_any();
        let mut res: CellResult<()> = Ok(());
        {
            let mut actual = any.downcast_ref::<ApplicationEnvironmentPreparedEvent>();
            match actual {
                Some(v) => {
                    res = self.on_prepare(v.args.clone());
                    // notify
                    publish_application_events(self.bus.clone(), Box::new(NextStepEvent::new(self.step,
                                                                                             Arc::new(ApplicationInitEvent::new()))));
                }
                None => {}
            }
        }


        {
            let actual = any.downcast_ref::<ApplicationInitEvent>();
            match actual {
                Some(v) => {
                    res = self.on_init();
                    // notify
                    publish_application_events(self.bus.clone(), Box::new(NextStepEvent::new(self.step,
                                                                                             Arc::new(ApplicationStartedEvent::new()))));
                }
                None => {}
            }
        }

        {
            let actual = any.downcast_ref::<ApplicationStartedEvent>();
            match actual {
                Some(v) => {
                    res = self.on_start();
                    // notify
                    publish_application_events(self.bus.clone(), Box::new(NextStepEvent::new(self.step,
                                                                                             Arc::new(ApplicationReadyEvent::new()))));
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

        {
            let actual = any.downcast_ref::<CallBackEvent>();
            match actual {
                Some(v) => {
                    (v.cb)();
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
            // TODO ,async
            let e = self.extension.get_mut(i).unwrap();
            let res = e.clone().borrow_mut().ready(self.ctx.clone());
            match res {
                Err(err) => {
                    if e.clone().borrow_mut().required() {
                        panic!("{}", err.get_msg())
                    }
                    cerror!(ModuleEnumsStruct::EXTENSION,"load extension [{}] failed ,err:{}"
                ,e.clone().borrow_mut().module().get_name(),err.get_msg());
                }
                Ok(..) => {}
            }
            cinfo!(ModuleEnumsStruct::EXTENSION,"load extension [{}] successfully ,cost:{}"
                ,e.clone().borrow_mut().module().get_name(),wh.elapsed().as_secs());
            i += 1;
        }
        self.step = step_3;
        Ok(())
    }

    pub fn on_close(&mut self) -> CellResult<()> {
        self.verify_step(step_4)?;
        cinfo!(ModuleEnumsStruct::EXTENSION,"{}",CLOSE);
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
    pub commands: HashMap<String, Command<'static>>,

    pub bus: EventBus<Box<dyn Event>>,
}

impl NodeContext {
    pub fn set_matchers(&mut self, m: ArgMatches) {
        self.matchers = m
    }
    pub fn get_matchers(&self) -> ArgMatches {
        self.matchers.clone()
    }
    pub fn set_tokio(&mut self, m: Arc<Runtime>) {
        self.tokio_runtime = m
    }
    pub fn set_bus(&mut self, bus: EventBus<Box<dyn Event>>) {
        self.bus = bus
    }
    // pub fn set_publisher(&mut self, value: Sender<Arc<dyn Event>>) {
    //     self.publisher = value
    // }
    // pub fn set_subscriber(&mut self, value: Receiver<Arc<dyn Event>>) {
    //     self.subscriber = value
    // }
    //
    pub fn set_commands(&mut self, cmds: Vec<Command<'static>>) {
        for cmd in cmds {
            let id = conv_protocol_to_string(cmd.protocol_id);
            self.commands.insert(id, cmd.clone());
        }
    }
}

unsafe impl Send for NodeContext {}

unsafe impl Sync for NodeContext {}

impl Default for NodeContext {
    fn default() -> Self {
        let rt = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
        NodeContext {
            tokio_runtime: rt.clone(),
            matchers: Default::default(),
            commands: Default::default(),
            bus: EventBus::<Box<dyn Event>>::new(rt.clone()),
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
    fn get_orderer(&mut self) -> i32 {
        default_orderer
    }

    // TODO ,maybe it should wrapped by refcell
    // fn components(&mut self) -> Option<Vec<Arc<Box<dyn Any>>>> {
    //     None
    // }
    // fn commands(&mut self) -> Option<Vec<Command<'static>>> {
    //     None
    // }
    // fn resolve(&mut self, any: Arc<Box<dyn Any>>) {}
}


pub const INTERNAL_TOKIO: CellModule = CellModule::new(1, "INTERNAL_TOKIO", &LogLevel::Info);

////////////// internal
pub struct InternalTokioExtension {}

impl InternalTokioExtension {
    pub fn new() -> Self {
        Self {}
    }
}

impl NodeExtension for InternalTokioExtension {
    fn module(&self) -> CellModule {
        INTERNAL_TOKIO
    }
    fn on_init(&mut self, ctx: Arc<RefCell<NodeContext>>) -> CellResult<()> {
        // TODO
        // let mut ctx_mut = ctx.borrow_mut();
        // let matchers = ctx_mut.get_matchers();
        // let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
        // ctx_mut.set_tokio(runtime);
        Ok(())
    }
    fn get_orderer(&mut self) -> i32 {
        max_orderer
    }
}
/////////


#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::{env, thread, time};
    use std::borrow::Borrow;
    use std::collections::{HashMap, HashSet};
    use std::time::Duration;
    use crossbeam::channel::{bounded, Receiver, Select, unbounded};
    use flo_stream::{MessagePublisher, Publisher, Subscriber};
    use futures::StreamExt;
    use stopwatch::Stopwatch;
    use tokio::runtime::Runtime;
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::Sender;
    use crate::event::{ApplicationCloseEvent, ApplicationEnvironmentPreparedEvent, ApplicationInitEvent, ApplicationReadyEvent, ApplicationStartedEvent, CallBackEvent, Event, NextStepEvent};
    use crate::extension::{ExtensionManager, ExtensionManagerBuilder, NodeContext, NodeExtension};
    use crate::module::ModuleEnumsStruct;
    use logsdk::common::LogLevel;
    use logsdk::module::CellModule;
    use crate::bus::{DefaultRegexQuery, EventBus, publish_application_events};


    #[test]
    fn test_extension() {}

    fn create_builder() -> (ExtensionManager, Arc<Runtime>, Sender<u8>, EventBus<Box<dyn Event>>) {
        // let demo = DemoExtension {};
        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
        let (tx, rx) = mpsc::channel::<u8>(1);

        let bus = EventBus::<Box<dyn Event>>::new(runtime.clone());
        (ExtensionManagerBuilder::default()
             // .with_extension(Arc::new(RefCell::new(demo)))
             .with_tokio(runtime.clone())
             .with_close_notifyc(rx)
             .with_bus(bus.clone())
             .build(), runtime.clone(), tx, bus.clone())
    }


    #[test]
    fn test_init_command_line() {
        let mut m = create_builder();
        let mut args = Vec::<String>::new();
        m.0.init_command_line(args);
    }

    //
    // #[test]
    // fn test_start() {
    //     let mut m = create_builder();
    //     let am = RefCell::new(m.0);
    //     am.into_inner().start();
    //     m.2.clone().block_on(async move {
    //         thread::sleep(Duration::from_secs(3));
    //         let msg = ApplicationReadyEvent::new();
    //         m.1.clone().borrow_mut().publish(Arc::new(msg)).await;
    //         thread::sleep(Duration::from_secs(10000));
    //     });
    // }


    #[test]
    fn test_extension_procedure() {
        let mut m = create_builder();
        let ctx = m.0.ctx.clone();
        let am = RefCell::new(m.0);


        let bus = Arc::new(ctx.clone().borrow_mut().bus.clone());
        let evs = HashSet::<String>::new();
        let sub = ctx.clone().borrow_mut().bus.clone().subscribe(String::from("tet_cli"),
                                                                 10, Box::new(DefaultRegexQuery::new("id", String::from("events*"), evs)), None);

        let run = m.1.clone();
        am.into_inner().start();


        run.clone().block_on(async move {
            {
                thread::sleep(Duration::from_secs(2));
                let msg = ApplicationEnvironmentPreparedEvent::new(vec![]);
                let events = HashMap::<String, Vec<String>>::new();
                publish_application_events(bus, Box::new(msg));
            }

            // {
            //     thread::sleep(Duration::from_secs(2));
            //     let msg = ApplicationInitEvent::new();
            //     m.1.clone().borrow_mut().publish(Arc::new(msg)).await;
            // }
            //
            // {
            //     thread::sleep(Duration::from_secs(1));
            //     let msg = ApplicationStartedEvent::new();
            //     m.1.clone().borrow_mut().publish(Arc::new(msg)).await;
            // }
            //
            // {
            //     thread::sleep(Duration::from_secs(1));
            //     let msg = ApplicationReadyEvent::new();
            //     m.1.clone().borrow_mut().publish(Arc::new(msg)).await;
            // }
            //
            //
            // {
            //     thread::sleep(Duration::from_secs(1));
            //     let msg = ApplicationCloseEvent::new();
            //     m.1.clone().borrow_mut().publish(Arc::new(msg)).await;
            // }

            thread::sleep(Duration::from_secs(1000));
        });
    }

    #[test]
    fn test_multi() {
        let (sender, receiver) = bounded::<u8>(10);
        let run_time = Arc::new(tokio::runtime::Runtime::new().unwrap());


        let f = move |index: u8, s: crossbeam::channel::Sender<u8>, r: Receiver<u8>| {
            run_time.clone().spawn(async move {
                let mut sel = Select::new();
                sel.recv(&r);
                loop {
                    let index = sel.ready();
                    let res = r.try_recv();
                    // If the operation turns out not to be ready, retry.
                    if let Err(e) = res {
                        if e.is_empty() {
                            continue;
                        }
                    }
                    let msg = res.unwrap();
                    println!("receive msg:{},index:{}", msg, index);
                    // cinfo!(ModuleEnumsStruct::EXTENSION,"receive msg:{},index:{}", msg, index);
                }
            });
        };


        let s1 = sender.clone();
        let r1 = receiver.clone();
        // let arc_sel = Arc::new(select);
        f(1, s1, r1);

        let s2 = sender.clone();
        let r2 = receiver.clone();
        f(2, s2, r2);


        let s3 = sender.clone();
        let r3 = receiver.clone();
        f(3, s3, r3);


        let publisher = sender.clone();
        for i in 0..5 {
            println!("send msg:{}", i);
            // cinfo!(ModuleEnumsStruct::EXTENSION,"send msg:{}",i);
            publisher.send(i).unwrap();
        }
        thread::sleep(Duration::from_secs(1000));
    }
}