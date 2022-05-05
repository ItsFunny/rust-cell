use std::fmt::{Display, Formatter, write};
use crate::module::Module;

static log_level_simple: [char; 5] = ['T', 'D', 'I', 'W', 'E'];

#[derive(Debug)]
pub enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}


impl LogLevel {
    pub fn get_value(self) -> usize {
        self as usize
    }
}

impl Clone for LogLevel {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for LogLevel {}


pub fn get_simple_loglevel(l: LogLevel) -> char {
    // let v = *l;
    log_level_simple[l.get_value()]
}

#[derive(Debug)]
pub struct LogEntry {
    pub msg: String,
    pub log_level: LogLevel,
    pub module: &'static dyn Module,
}


impl Display for LogEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "msg:{:?},loglevel:{:?},module:{}", self.msg, self.log_level, self.module)
    }
}