use core::cell::RefCell;
use std::sync::Arc;
use flo_stream::Publisher;
use rocket::build;
use tokio::sync::mpsc;
use crate::event::Event;
use crate::extension::{ExtensionBuilder, ExtensionManager, ExtensionManagerBuilder};

pub struct CellApplication {
    publisher: Arc<RefCell<Publisher<Arc<dyn Event>>>>,
    tx: mpsc::Sender<u8>,
    manager: ExtensionManager,
}


pub struct CellApplicationBuilder {
    builders: Vec<Box<dyn ExtensionBuilder>>,
}

impl CellApplicationBuilder {
    pub fn with_builders(mut self, builders: Vec<Box<dyn ExtensionBuilder>>) -> Self {
        self.builders = builders;
        self
    }
    pub fn new(builders: Vec<Box<dyn ExtensionBuilder>>) -> Self {
        Self { builders }
    }
    pub fn build(mut self) -> CellApplication {
        let mut manage_builder = ExtensionManagerBuilder::default();
        let mut i = 0;
        while i != self.builders.len() {
            let builder = self.builders.remove(i);
            let extension = builder.build_extension();
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
}


