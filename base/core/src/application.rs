use core::any::Any;
use core::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use flo_stream::Publisher;
use rocket::build;
use tokio::signal;
use tokio::sync::mpsc;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::command::Command;
use crate::event::Event;
use crate::extension::{ExtensionFactory, ExtensionManager, ExtensionManagerBuilder};

pub struct CellApplication {
    publisher: Arc<RefCell<Publisher<Arc<dyn Event>>>>,
    tx: mpsc::Sender<u8>,
    pubsub: Arc<RefCell<Publisher<Arc<dyn Event>>>>,
    manager: ExtensionManager,
}

module_enums!(
        (CELL_APPLICATION,1,&logsdk::common::LogLevel::Info);
    );



impl CellApplication {
    pub fn run(self, args: Vec<String>) {
        let runtime = Arc::new(tokio::runtime::Builder::new_multi_thread().build().unwrap());
        runtime.block_on(async {
            self.async_start().await
        })
    }
    async fn async_start(self) {
        self.step0().await;

        match signal::ctrl_c().await {
            Ok(()) => {}
            Err(err) => {
                cerror!(ModuleEnumsStruct::CELL_APPLICATION,"Unable to listen for shutdown signal: {}", err);
            }
        }
    }
    async fn step0(&self) {}
    async fn grace_exist(self) {}
    pub fn new(mut builders: Vec<Arc<RefCell<dyn ExtensionFactory>>>) -> Self {
        let mut components = collect_components(builders.clone());
        let commands = collect_commands(builders.clone());

        let mut manage_builder = ExtensionManagerBuilder::default();
        let arc_pub = Arc::new(RefCell::new(Publisher::new(10)));
        manage_builder = manage_builder
            .with_components(components.clone())
            .with_subscriber(arc_pub.clone())
            .with_commands(commands);
        let mut i = 0;

        while i != builders.len() {
            let builder = builders.remove(i);
            let extension = builder.borrow_mut().build_extension(components.clone());
            manage_builder = manage_builder.with_extension(extension);
            i += 1;
        }
        let (txx, rxx) = mpsc::channel::<u8>(1);
        let arc_pub = Arc::new(RefCell::new(Publisher::new(10)));
        manage_builder = manage_builder
            .with_subscriber(arc_pub.clone())
            .with_close_notifyc(rxx);
        let extension_manager = manage_builder.build();

        CellApplication {
            publisher: arc_pub.clone(),
            tx: txx,
            pubsub: arc_pub.clone(),
            manager: extension_manager,
        }
    }
}


fn collect_components(mut builders: Vec<Arc<RefCell<dyn ExtensionFactory>>>) -> Vec<Arc<Box<dyn Any>>> {
    let mut ret: Vec<Arc<Box<dyn Any>>> = Vec::new();
    for i in 0..builders.len() {
        let ext = builders.get_mut(i).unwrap();
        let components_opt = ext.clone().borrow_mut().components();
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

fn collect_commands(mut builders: Vec<Arc<RefCell<dyn ExtensionFactory>>>) -> Vec<Command<'static>> {
    let mut ret: Vec<Command<'static>> = Vec::new();
    for i in 0..builders.len() {
        let ext = builders.get_mut(i).unwrap();
        let cmds_opt = ext.clone().borrow_mut().commands();
        match cmds_opt {
            Some(cmds) => {
                if cmds.len() > 0 {
                    let iter = cmds.iter();
                    for cmd in iter {
                        ret.push(cmd.clone());
                    }
                }
            }
            None => {}
        }
    }
    return ret;
}


#[cfg(test)]
mod tests {
    use crate::application::CellApplication;

    #[test]
    fn test_application() {
        let app = CellApplication::new(vec![]);
    }
}