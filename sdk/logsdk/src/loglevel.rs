pub enum LogLevel {
    TRACE,
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

impl LogLevel {
    pub fn is_bigger(&self, l: LogLevel) -> bool {
        false
    }
}