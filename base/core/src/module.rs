use crate::command::Command;
use crate::extension::NodeExtension;
use core::cell::RefCell;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use shaku::{module, Component, HasComponent, Interface};
use std::sync::Arc;

pub trait ModuleTrait: Interface {
    fn get_commands<'a>(&self) -> Vec<Command<'a>>;
    fn get_extension(&self) -> Arc<RefCell<dyn NodeExtension>>;
}

module_enums!(
    (EXTENSION,1,&logsdk::common::LogLevel::Info);
    (INTERNAL_TOKIO,2,&logsdk::common::LogLevel::Info);
    (CELL_APPLICATION,3,&logsdk::common::LogLevel::Info);
    (DISPATCHER,4,&logsdk::common::LogLevel::Info);
);
