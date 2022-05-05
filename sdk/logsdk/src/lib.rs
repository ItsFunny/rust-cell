pub mod module;
pub mod common;
pub mod log4rs;

use chrono::Local;
use crate::common::{get_simple_loglevel, LogLevel};
use crate::module::Module;

const DATE_FORMAT_STR: &'static str = "%Y/%m/%d %H:%M:%S";

pub mod log {
    use std::fmt::format;
    use crate::common::{get_simple_loglevel, LogEntry, LogLevel};
    use crate::{DATE_FORMAT_STR, default_format_msg, get_current_time_str};
    use crate::module::Module;

    pub trait MLogger {
        fn log(&self, entry: &LogEntry);
    }

    pub struct Logger {}


    pub struct LoggerEntryContext {}

    impl LoggerEntryContext {
        pub fn create_log_entry<'a>(m: &'static dyn Module, format_msg: &'a str) -> LogEntry {
            let msg = default_format_msg(m, format_msg);
            let ret = LogEntry { msg: msg.as_str(), log_level: LogLevel::Trace, module: m };
            return ret;
        }
    }
}

fn default_format_msg<'a>(m: &'static dyn Module, format_msg: &'a str) -> String {
// date (code) [module][loglevel]info
    let file = get_log_file();
    let line = line!();
    let now = get_current_time_str();
    let l = m.log_level();
    let ret = format!("{} ({}:{}) [{}][{}]{}", now, file, line, m.name(), get_simple_loglevel(l), format_msg);
    ret
}

fn get_log_file<'a>() -> &'a str {
    return file!();
}

fn get_current_time_str() -> String {
    let date = Local::now();
    date.format(DATE_FORMAT_STR).to_string()
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;
    use crate::{DATE_FORMAT_STR, default_format_msg, LogLevel, Module};
    use chrono::Local;
    use crate::log::LoggerEntryContext;
    use crate::module::CellModule;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_date() {
        let date = Local::now();
        let str = date.format(DATE_FORMAT_STR).to_string();
        println!("{}", str);
    }

    #[test]
    fn test_format() {
        static m1: &CellModule = &CellModule::new(1, "M", &LogLevel::Info);
        let msg = default_format_msg(m1, "msg");
        println!("{}", msg);
    }

    #[test]
    fn test_create_entry() {
        static m2: &CellModule = &CellModule::new(1, "M2", &LogLevel::Info);
        let entry = LoggerEntryContext::create_log_entry(m2, "asdd");
        println!("{:?}", entry)
    }
}
