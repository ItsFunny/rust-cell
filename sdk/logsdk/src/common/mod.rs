use crate::module::Module;
use std::fmt::{write, Display, Formatter};

static log_level_simple: [&'static str; 5] = ["TRACE", "DEBUG", "INFO", "WARN", "ERROR"];

#[derive(Debug)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
}

impl LogLevel {
    pub fn get_value(self) -> usize {
        self as usize
    }

    pub fn gt(self, o: LogLevel) -> bool {
        return self.get_value() > o.get_value();
    }
}

impl Clone for LogLevel {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for LogLevel {}

pub fn get_simple_loglevel(l: LogLevel) -> &'static str {
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
        write!(
            f,
            "msg:{:?},loglevel:{:?},module:{}",
            self.msg, self.log_level, self.module
        )
    }
}
