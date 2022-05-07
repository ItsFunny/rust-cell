pub mod module;
pub mod common;
pub mod log4rs;

use std::collections::HashSet;
use std::iter::Map;
use std::sync::atomic::{AtomicUsize, Ordering};
use backtrace::Backtrace;
use chrono::Local;
use lazy_static::lazy_static;
use cell_base_common::cellerrors::{CellError, ErrorEnum};
use crate::common::{get_simple_loglevel, LogLevel};
use crate::log::CellLoggerConfiguration;
use crate::module::Module;

const DATE_FORMAT_STR: &'static str = "%Y/%m/%d %H:%M:%S";
const SKIP_CALLER: usize = 3;
// lazy_static! {
//     static ref validList:  Vec<&'static str>={
//         let mut  ret=Vec::new();
//         ret.push("rust-cell/sdk/logsdk/src/lib.rs");
//         ret
//     };
//     static ref  blackList:  Vec<&'static str>={
//         let mut  ret=Vec::new();
//         ret.push("rust-cell/sdk/logsdk/src/lib.rs");
//         ret
//     };
// }


static STATE: AtomicUsize = AtomicUsize::new(0);
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

static ERROR_SETUP_FAILED: &ErrorEnum = &ErrorEnum::Error(1, "setup config failed");
static mut CONFIGURATION: &CellLoggerConfiguration = &CellLoggerConfiguration {
    black_list: &[
        "src/backtrace/libunwind.rs",
        "src/backtrace/mod.rs",
        "src/capture.rs",
        "rust-cell/sdk/logsdk/src/lib.rs", ],
};

pub mod log {
    use std::collections::HashSet;
    use std::fmt::format;
    use backtrace::Backtrace;
    use crate::common::{get_simple_loglevel, LogEntry, LogLevel};
    use crate::{DATE_FORMAT_STR, default_format_msg, get_current_time_str, get_log_info, stack_trace};
    use crate::module::Module;

    pub trait MLogger {
        fn log(&self, entry: LogEntry);
    }

    pub struct Logger {
        logger: Box<dyn MLogger>,
    }


    impl Logger {
        pub fn info(&self, m: &'static dyn Module, msg: String) {
            let bt = &Backtrace::new();
            let (file_str, line_no) = stack_trace(bt);
            self.log(m, LogLevel::Info, file_str, line_no, msg.as_str())
        }
        pub fn log(&self, m: &'static dyn Module,
                   l: LogLevel,
                   file_str: &str,
                   line_no: u32,
                   format_msg: &str) {
            let entry = LoggerEntryContext::create_log_entry(m, l, file_str, line_no, format_msg);
            self.logger.log(entry)
        }
        pub fn new(logger: Box<dyn MLogger>) -> Self {
            Logger { logger }
        }
    }

    pub struct LoggerEntryContext {}

    impl LoggerEntryContext {
        pub fn create_log_entry(m: &'static dyn Module,
                                l: LogLevel,
                                file_str: &str,
                                line_no: u32,
                                format_msg: &str) -> LogEntry {
            let ret = LogEntry { msg: default_format_msg(m, file_str, line_no, l, format_msg), log_level: l, module: m };
            return ret;
        }
    }

    pub struct CellLoggerConfiguration {
        pub black_list: &'static [&'static str],
    }
}

pub fn setup_logger_configuration(cfg: &'static CellLoggerConfiguration) {
    setup_logger_configuration_inner(|| cfg);
}

// #[cfg(atomic_cas)]
fn setup_logger_configuration_inner<F>(make_f: F) -> Result<(), CellError>
    where
        F: FnOnce() -> &'static CellLoggerConfiguration,
{
    let old_state = match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            unsafe {
                CONFIGURATION = make_f();
            }
            STATE.store(INITIALIZED, Ordering::SeqCst);
            Ok(())
        }
        INITIALIZING => {
            while STATE.load(Ordering::SeqCst) == INITIALIZING {
                std::sync::atomic::spin_loop_hint();
            }
            Err(CellError::new(ERROR_SETUP_FAILED))
        }
        _ => Err(CellError::new((ERROR_SETUP_FAILED))),
    }
}


// TODO awful
fn default_format_msg(m: &'static dyn Module,
                      file_str: &str,
                      line_no: u32,
                      l: LogLevel, format_msg: &str) -> String {
    // date (code) [module][loglevel]info
    let now = get_current_time_str();
    let mut arrs: Vec<&str> = file_str.split("/").collect();
    let mut file_info = file_str;
    let r;
    if arrs.len() > SKIP_CALLER {
        arrs.drain(0..=(arrs.len() - SKIP_CALLER));
        r = arrs.join("/");
        file_info = r.as_str();
    }
    format!("{} ({}:{}) [{}][{}]{}", now, file_info, line_no, m.name(), get_simple_loglevel(l), format_msg)
}

fn stack_trace<'a>(bt: &'a Backtrace) -> (&str, u32) {
    let file_str;
    let line_no;
    let (file, line) = get_log_info(&bt);
    match file {
        None => {
            file_str = file!();
            line_no = line!();
        }
        Some(v) => {
            file_str = v;
            line_no = line.unwrap();
        }
    }
    return (file_str, line_no);
}

// TODO bad code
// FIXME use macro
fn get_log_info<'a>(bt: &'a Backtrace) -> (Option<&'a str>, Option<u32>) {
    let mut v;
    let mut sp = false;
    let it = bt.frames().iter();
    for f in it {
        sp = false;
        for s in f.symbols() {
            match s.filename() {
                None => continue,
                Some(f) => unsafe {
                    let str = f.as_os_str().to_str().unwrap();
                    for value in CONFIGURATION.black_list.iter() {
                        v = *value;
                        if str.contains(v) {
                            sp = true;
                            break;
                        }
                    }
                    if sp {
                        continue;
                    }
                    return (f.as_os_str().to_str(), s.lineno());
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
    use crate::log::{Logger, LoggerEntryContext};
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
