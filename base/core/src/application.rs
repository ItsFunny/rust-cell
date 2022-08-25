use core::any::Any;
use core::cell::RefCell;
use std::sync::{Arc, Mutex};
use flo_stream::Publisher;
use rocket::build;
use tokio::sync::mpsc;
use crate::command::Command;
use crate::event::Event;
use crate::extension::{ ExtensionFactory, ExtensionManager, ExtensionManagerBuilder};

pub struct CellApplication {
    publisher: Arc<RefCell<Publisher<Arc<dyn Event>>>>,
    tx: mpsc::Sender<u8>,
    manager: ExtensionManager,
}


pub struct CellApplicationBuilder {
    builders: Vec<Arc<RefCell<dyn ExtensionFactory>>>,
}

impl CellApplicationBuilder {
    pub fn with_builders(mut self, builders: Vec<Arc<RefCell<dyn ExtensionFactory>>>) -> Self {
        self.builders = builders;
        self
    }
    pub fn new(builders: Vec<Arc<RefCell<dyn ExtensionFactory>>>) -> Self {
        Self { builders }
    }
    pub fn build(mut self) -> CellApplication {
        let mut components = self.collect_components();
        let commands = self.collect_commands();

        let mut manage_builder = ExtensionManagerBuilder::default();
        manage_builder = manage_builder
            .with_components(components.clone())
            .with_commands(commands);
        let mut i = 0;

        while i != self.builders.len() {
            let builder = self.builders.remove(i);
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
            manager: extension_manager,
        }
    }

    fn collect_components(&mut self) -> Vec<Arc<Box<dyn Any>>>{
        let mut ret: Vec<Arc<Box<dyn Any>>>= Vec::new();
        for i in 0..self.builders.len() {
            let ext = self.builders.get_mut(i).unwrap();
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
    fn collect_commands(&mut self) -> Vec<Command<'static>> {
        let mut ret: Vec<Command<'static>> = Vec::new();
        for i in 0..self.builders.len() {
            let ext = self.builders.get_mut(i).unwrap();
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
}


