use crate::common::LogLevel;

pub trait Module {
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
    pub const  fn new(index: i16, name: &'static str, log_level: &'static LogLevel) -> CellModule {
        CellModule { index, name, log_level }
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