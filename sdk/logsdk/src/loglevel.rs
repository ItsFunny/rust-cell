pub enum LogLevel {
    TRACE = 1,
    DEBUG = 2,
    INFO = 3,
    WARN = 4,
    ERROR = 5,
}

impl LogLevel {
    pub fn is_bigger(&self, l: LogLevel) -> bool {
        false
    }
    pub fn get_value(&self) -> i32 {
        self as i32
    }
}