use std::fmt::{Debug, Display, Formatter};
use crate::common::LogLevel;

pub trait Module: Display + Debug +Sync{
    fn name(&self) -> &'static str;
    fn index(&self) -> i16;
    fn log_level(&self) -> &'static LogLevel;
}

pub struct CellModule {
    index: i16,
    name: &'static str,
    log_level: &'static LogLevel,
}

impl CellModule {
    pub const fn new(index: i16, name: &'static str, log_level: &'static LogLevel) -> CellModule {
        CellModule {
            index: index,
            name: name,
            log_level: log_level,
        }
    }
}


impl Display for CellModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "module_name:{},index:{}", self.name, self.index)
    }
}

impl Debug for CellModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "module_name:{},index:{}", self.name, self.index)
    }
}

impl Module for CellModule {
    fn index(&self) -> i16 {
        self.index
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn log_level(&self) -> &'static LogLevel {
        self.log_level
    }
}