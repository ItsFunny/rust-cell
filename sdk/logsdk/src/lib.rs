pub mod module;
pub mod common;
pub mod log4rs;

use std::collections::HashSet;
use std::iter::Map;
use backtrace::Backtrace;
use chrono::Local;
use lazy_static::lazy_static;
use crate::common::{get_simple_loglevel, LogLevel};
use crate::module::Module;

const DATE_FORMAT_STR: &'static str = "%Y/%m/%d %H:%M:%S";
const SKIP_CALLER: usize = 3;

lazy_static! {
    static ref validList:  Vec<&'static str>={
        let mut  ret=Vec::new();
        ret.push("rust-cell/sdk/logsdk/src/lib.rs");
        ret
    };
static    ref  blackList:  Vec<&'static str>={
        let mut  ret=Vec::new();
        ret.push("rust-cell/sdk/logsdk/src/lib.rs");
        ret
    };
}

pub fn AppendBlackList(str: &'static str) {
    unsafe {
        // BlackListMap.insert(str);
        // println!("{:?}", BlackListMap);
    }
}

pub mod log {
    use std::collections::HashSet;
    use std::fmt::format;
    use crate::common::{get_simple_loglevel, LogEntry, LogLevel};
    use crate::{DATE_FORMAT_STR, default_format_msg, get_current_time_str};
    use crate::module::Module;

    pub trait MLogger {
        fn log(&self, entry: LogEntry);
    }

    pub struct Logger {}


    pub struct LoggerEntryContext {}

    impl LoggerEntryContext {
        pub fn create_log_entry<'a>(m: &'static dyn Module,
                                    l: LogLevel,
                                    fileStr: &str,
                                    lineNo: u32,
                                    format_msg: &'a str) -> LogEntry {
            let ret = LogEntry { msg: default_format_msg(m, fileStr, lineNo, l, format_msg), log_level: l, module: m };
            return ret;
        }
    }
}

// TODO awful
fn default_format_msg<'a>(m: &'static dyn Module,
                          fileStr: &str,
                          lineNo: u32,
                          l: LogLevel, format_msg: &'a str) -> String {
    // date (code) [module][loglevel]info
    let now = get_current_time_str();
    let mut arrs: Vec<&str> = fileStr.split("/").collect();
    let mut file_info = fileStr;
    let r;
    if arrs.len() > SKIP_CALLER {
        arrs.drain(0..=(arrs.len() - SKIP_CALLER));
        r = arrs.join("/");
        file_info = r.as_str();
    }
    format!("{} ({}:{}) [{}][{}]{}", now, file_info, lineNo, m.name(), get_simple_loglevel(l), format_msg)
}

fn stack_trace<'a>(bt: &'a Backtrace) -> (&str, u32) {
    let fileStr;
    let lineNo;
    let (file, line) = get_log_info(&bt);
    match file {
        None => {
            fileStr = file!();
            lineNo = line!();
        }
        Some(v) => {
            fileStr = v;
            lineNo = line.unwrap();
        }
    }
    return (fileStr, lineNo);
}

// TODO bad code
// FIXME use macro s
fn get_log_info<'a>(bt: &'a Backtrace) -> (Option<&'a str>, Option<u32>) {
    let mut find = false;
    let it = bt.frames().iter();
    for f in it {
        for s in f.symbols() {
            match s.filename() {
                None => continue,
                Some(f) => {
                    let str = f.as_os_str().to_str().unwrap();
                    if !find {
                        for v in validList.iter() {
                            if !str.contains(v) {
                                continue;
                            }
                            find = true;
                            break;
                        }
                        if !find {
                            continue;
                        }
                    }

                    for v in blackList.iter() {
                        if str.contains(v) {
                            continue;
                        } else {
                            return (f.as_os_str().to_str(), s.lineno());
                        }
                    }
                }
            }
        }
    }

    return (None, None);
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
        let msg = default_format_msg(m1, file!(), line!(), LogLevel::Info, "msg");
        println!("{}", msg);
    }

    #[test]
    fn test_create_entry() {
        static m2: &CellModule = &CellModule::new(1, "M2", &LogLevel::Info);
        let entry = LoggerEntryContext::create_log_entry(m2, LogLevel::Info, file!(), line!(), "asdd");
        println!("{:?}", entry)
    }
}
