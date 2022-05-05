pub enum LogLevel {
    Trace = 1,
    Debug = 2,
    Info = 3,
    Warn = 4,
    Error = 5,
}


impl LogLevel {
    pub fn get_value(&self) -> i16 {
        1
    }
}

pub struct LogEntry<'a> {
    pub msg: &'a str,
    pub log_level: LogLevel,
}

pub struct EntryFactory {}

impl EntryFactory {
    pub fn new_log_entry(msg: &str, l: LogLevel) -> LogEntry {
        LogEntry { msg: msg, log_level: l }
    }
}