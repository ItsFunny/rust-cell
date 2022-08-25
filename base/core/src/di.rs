// TODO ,use macro

use shaku::Module;
use crate::command::Command;
use crate::extension::ExtensionBuilder;

pub struct ModuleWrapper<T> {
    pub commands: Option<Vec<Command<'static>>>,

    pub module: Box<dyn CellModule<Submodules=T>>,
}

pub trait CellModule: Module {}