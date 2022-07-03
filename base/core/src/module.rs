use core::cell::RefCell;
use std::sync::Arc;
use shaku::{module, Component, Interface, HasComponent};
use crate::command::Command;
use crate::extension::NodeExtension;

pub trait ModuleTrait: Interface {
    fn get_commands<'a>(&self) -> Vec<Command<'a>>;
    fn get_extension(&self) -> Arc<RefCell<dyn NodeExtension>>;
}

