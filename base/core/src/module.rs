use core::cell::RefCell;
use std::sync::Arc;
use shaku::{module, Component, Interface, HasComponent};
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::command::Command;
use crate::extension::NodeExtension;

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

