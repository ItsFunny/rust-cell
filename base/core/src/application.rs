use core::any::Any;
use core::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use crossbeam::channel::{bounded, Select, Sender};
use flo_stream::Publisher;
use rocket::build;
use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::mpsc;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::bus::{EventBus, publish_application_events, subscribe_application_events};
use crate::command::Command;
use crate::event::{ApplicationEnvironmentPreparedEvent, ApplicationInitEvent, ApplicationReadyEvent, ApplicationStartedEvent, Event, NextStepEvent};
use crate::extension::{ExtensionFactory, ExtensionManager, ExtensionManagerBuilder, step_0, step_1, step_2, step_3, step_4};
use crate::module::ModuleEnumsStruct;

const Application: &'static str = "application";

pub struct CellApplication {
    bus: EventBus<Box<dyn Event>>,
    tx: mpsc::Sender<u8>,
    manager: ExtensionManager,
    runtime: Arc<Runtime>,
}


impl CellApplication {
    pub fn run(self, args: Vec<String>) {
        let runtime = self.runtime.clone();
        runtime.block_on(async {
            self.async_start(args).await
        })
    }
    async fn async_start(self, args: Vec<String>) {
        // start
        self.bus.clone().start();
        self.manager.clone().start();
        self.step0(args).await;

        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => {
                cerror!(ModuleEnumsStruct::CELL_APPLICATION,"Unable to listen for shutdown signal: {}", err);
            }
        }
    }
    async fn step0(&self, args: Vec<String>) {
        let app_bus = self.bus.clone();
        let sub = subscribe_application_events(app_bus.clone(), Application, None);
        let arc_bus = Arc::new(app_bus.clone());
        let mut sel = Select::new();
        sel.recv(&sub);

        // send event
        let msg = ApplicationEnvironmentPreparedEvent::new(args);
        publish_application_events(arc_bus.clone(), Box::new(msg), None);

        loop {
            let index = sel.ready();
            let res = sub.try_recv();
            if let Err(e) = res {
                if e.is_empty() {
                    continue;
                }
            }
            let msg = res.unwrap();
            cinfo!(ModuleEnumsStruct::CELL_APPLICATION,"收到msg:{}",msg.clone());

            let any = msg.as_any();
            {
                let mut actual = any.downcast_ref::<NextStepEvent>();
                match actual {
                    Some(v) => {
                        if v.current == step_0 {
                            publish_application_events(arc_bus.clone(), Box::new(ApplicationInitEvent::new()), None);
                        } else if v.current == step_1 {
                            publish_application_events(arc_bus.clone(), Box::new(ApplicationStartedEvent::new()), None);
                        } else if v.current == step_2 {
                            publish_application_events(arc_bus.clone(), Box::new(ApplicationReadyEvent::new()), None);
                        } else if v.current == step_3 {
                            cinfo!(ModuleEnumsStruct::EXTENSION,"step:3")
                        } else if v.current == step_4 {
                            cinfo!(ModuleEnumsStruct::EXTENSION,"step:4")
                        }
                    }
                    None => {}
                }
            }
        }
    }

    pub fn new(mut builders: Vec<Box<dyn ExtensionFactory>>) -> Self {
        let mut components = collect_components(&builders);
        let rt = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let mut bus = EventBus::new(rt);
        let mut manage_builder = ExtensionManagerBuilder::default();
        manage_builder = manage_builder
            .with_components(components.clone())
            .with_bus(bus.clone());

        while builders.len() > 0 {
            let builder = builders.remove(0);
            if let Some(extension) = builder.build_extension(components.clone()) {
                manage_builder = manage_builder.with_extension(extension);
            }
        }
        let (txx, rxx) = mpsc::channel::<u8>(1);
        manage_builder = manage_builder
            .with_close_notifyc(rxx);
        let extension_manager = manage_builder.build();
        let runtime = extension_manager.get_ctx().clone().borrow().tokio_runtime.clone();
        runtime.spawn(async {
            println!("1")
        });


        CellApplication {
            bus: bus.clone(),
            tx: txx,
            manager: extension_manager,
            runtime,
        }
    }
}


fn collect_components(mut builders: &Vec<Box<dyn ExtensionFactory>>) -> Vec<Arc<Box<dyn Any>>> {
    let mut ret: Vec<Arc<Box<dyn Any>>> = Vec::new();
    for i in 0..builders.len() {
        let ext = builders.get(i).unwrap();
        let components_opt = ext.components();
        match components_opt {
            Some(components) => {
                if components.len() > 0 {
                    let iter = components.iter();
                    for com in iter {
                        ret.push(com.clone());
                    }
                }
            }
            None => {}
        }
    }
    ret
}

// fn collect_commands(mut builders: &Vec<Box<dyn ExtensionFactory>>) -> Vec<Command<'static>> {
//     let mut ret: Vec<Command<'static>> = Vec::new();
//     for i in 0..builders.len() {
//         let ext = builders.get(i).unwrap();
//         let cmds_opt = ext.commands();
//         match cmds_opt {
//             Some(cmds) => {
//                 if cmds.len() > 0 {
//                     let iter = cmds.iter();
//                     for cmd in iter {
//                         ret.push(cmd.clone());
//                     }
//                 }
//             }
//             None => {}
//         }
//     }
//     return ret;
// }


#[cfg(test)]
mod tests {
    use core::any::Any;
    use core::cell::RefCell;
    use std::env;
    use std::sync::Arc;
    use clap::Arg;
    use logsdk::common::LogLevel;
    use logsdk::module::CellModule;
    use crate::application::CellApplication;
    use crate::command::Command;
    use crate::extension::{ExtensionFactory, NodeExtension};


    //////
    pub struct DemoExtensionFactory {}

    impl ExtensionFactory for DemoExtensionFactory {
        fn build_extension(&self, components: Vec<Arc<Box<dyn Any>>>) -> Option<Arc<RefCell<dyn NodeExtension>>> {
            let mut compo1: Option<DemoComponent1> = None;
            for com in components {
                if let Some(v) = com.downcast_ref::<DemoComponent1>() {
                    compo1 = Some(v.clone());
                }
            }
            // panic
            let ret = DemoExtension { com1: compo1.unwrap() };

            Some(Arc::new(RefCell::new(ret)))
        }
    }


    pub struct DemoExtension {
        // pub com1:,
        pub com1: DemoComponent1,
    }

    pub struct DemoComponent1 {}

    impl Clone for DemoComponent1 {
        fn clone(&self) -> Self {
            DemoComponent1 {}
        }
    }

    impl NodeExtension for DemoExtension {
        fn module(&self) -> CellModule {
            CellModule::new(1, "asd", &LogLevel::Info)
        }
        fn get_options<'a>(&self) -> Option<Vec<Arg<'a>>> {
            Some(vec![Arg::default().name("demo").long("long").required(false),
                      Arg::default().name("demo2").long("long2").required(false)])
        }
    }

    pub struct ExtensionFactory2 {}

    impl ExtensionFactory for ExtensionFactory2 {
        fn components(&self) -> Option<Vec<Arc<Box<dyn Any>>>> {
            let mut ret: Vec<Arc<Box<dyn Any>>> = Vec::new();
            ret.push(Arc::new(Box::new(DemoComponent1 {})));
            Some(ret)
        }
    }

    #[test]
    fn test_application() {
        let mut factories: Vec<Box<dyn ExtensionFactory>> = Vec::new();
        factories.push(Box::new(DemoExtensionFactory {}));
        factories.push(Box::new(ExtensionFactory2 {}));
        let app = CellApplication::new(factories);
        app.run(vec![])
    }
}