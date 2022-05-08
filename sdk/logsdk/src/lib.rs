pub mod module;
pub mod common;
pub mod log4rs;

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::iter::Map;
use std::sync::atomic::{AtomicUsize, Ordering};
use ansi_term::{ANSIGenericString, Color};
use ansi_term::Color::Red;
use ansi_term::Colour::*;
use backtrace::Backtrace;
use chrono::Local;
use lazy_static::lazy_static;
use cell_base_common::cellerrors::{CellError, ErrorEnum};
use crate::common::{get_simple_loglevel, LogLevel};
use crate::log::{CellLoggerConfiguration, ColorProperty, Logger};
use crate::module::{CellModule, Module};

const DATE_FORMAT_STR: &'static str = "%Y/%m/%d %H:%M:%S";
const SKIP_CALLER: usize = 3;


static STATE: AtomicUsize = AtomicUsize::new(0);
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

pub struct A {
    m: HashMap<String, String>,
}

lazy_static! {
    static ref a:  A = A { m: HashMap::new() };
}
static mut MAP: Option<HashMap<Box<String>, Box<String>>> = None;
static DEFAULT_BLACK_LIST: &'static [&'static str] = &[
    "src/backtrace/libunwind.rs",
    "src/backtrace/mod.rs",
    "src/capture.rs",
    "rust-cell/sdk/logsdk/src/lib.rs", ];

static ERROR_SETUP_FAILED: &ErrorEnum = &ErrorEnum::Error(1, "setup config failed");
static mut CONFIGURATION: &CellLoggerConfiguration = &CellLoggerConfiguration {
    black_list: DEFAULT_BLACK_LIST,
    color_property: &ColorProperty {
        trace_level_color: DEFAULT_TRACE_LEVEL_COLOR,
        debug_level_color: DEFAULT_DEBUG_LEVEL_COLOR,
        info_level_color: DEFAULT_INFO_LEVEL_COLOR,
        warn_level_color: DEFAULT_WARN_LEVEL_COLOR,
        error_level_color: DEFAULT_ERROR_LEVEL_COLOR,
        default_module_color: DEFAULT_MODULE_COLOR,
    },
};
static mut GLOBAL_LOGLEVEL: &LogLevel = &LogLevel::Info;
static DEFAULT_MODULE: CellModule = CellModule::new(1, "ALL", unsafe { GLOBAL_LOGLEVEL });

const DEFAULT_TRACE_LEVEL_COLOR: PaintF = |v| {
    Green.paint(v)
};

const DEFAULT_DEBUG_LEVEL_COLOR: PaintF = |v| {
    Blue.paint(v)
};
const DEFAULT_INFO_LEVEL_COLOR: PaintF = |v| {
    Green.paint(v)
};

const DEFAULT_WARN_LEVEL_COLOR: PaintF = |v| {
    Yellow.paint(v)
};

const DEFAULT_ERROR_LEVEL_COLOR: PaintF = |v| {
    Red.paint(v)
};

const DEFAULT_FATAL_LEVEL_COLOR: PaintF = |v| {
    Red.paint(v)
};

const DEFAULT_MODULE_COLOR: PaintF = |v| {
    Cyan.paint(v)
};


pub mod log {
    use std::borrow::Cow;
    use std::collections::{HashMap, HashSet};
    use std::fmt;
    use std::fmt::{Debug, format};
    use backtrace::Backtrace;
    use crate::common::{get_simple_loglevel, LogEntry, LogLevel};
    use crate::{DATE_FORMAT_STR, DEFAULT_DEBUG_LEVEL_COLOR, DEFAULT_ERROR_LEVEL_COLOR, default_format_msg, DEFAULT_INFO_LEVEL_COLOR, DEFAULT_MODULE_COLOR, DEFAULT_TRACE_LEVEL_COLOR, DEFAULT_WARN_LEVEL_COLOR, get_current_time_str, get_log_info, PaintF, stack_trace};
    use crate::module::Module;

    pub trait MLogger {
        fn log(&self, entry: LogEntry);
    }

    pub struct Logger {
        logger: Box<dyn MLogger>,
    }

    // logger 是无状态的,可以直接实现
    unsafe impl Sync for Logger {}

    impl Logger {
        // TODO macros
        pub fn info(&self, m: &'static dyn Module, msg: String) {
            let bt = &Backtrace::new();
            let (file_str, line_no) = stack_trace(bt);
            self.log(m, LogLevel::Info, file_str, line_no, msg.as_str())
        }
        pub fn error(&self, m: &'static dyn Module, msg: String) {
            let bt = &Backtrace::new();
            let (file_str, line_no) = stack_trace(bt);
            self.log(m, LogLevel::Error, file_str, line_no, msg.as_str())
        }
        pub fn warn(&self, m: &'static dyn Module, msg: String) {
            let bt = &Backtrace::new();
            let (file_str, line_no) = stack_trace(bt);
            self.log(m, LogLevel::Warn, file_str, line_no, msg.as_str())
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
        pub color_property: &'static ColorProperty,
    }

    pub trait ColorTrait: ToOwned + Sized + Debug {}

    pub struct ColorProperty
    {
        pub trace_level_color: PaintF,
        pub debug_level_color: PaintF,
        pub info_level_color: PaintF,
        pub warn_level_color: PaintF,
        pub error_level_color: PaintF,
        pub default_module_color: PaintF,
    }

    impl ColorProperty {
        pub fn default_color_property() -> Self {
            ColorProperty {
                trace_level_color: DEFAULT_TRACE_LEVEL_COLOR,
                debug_level_color: DEFAULT_DEBUG_LEVEL_COLOR,
                info_level_color: DEFAULT_INFO_LEVEL_COLOR,
                warn_level_color: DEFAULT_WARN_LEVEL_COLOR,
                error_level_color: DEFAULT_ERROR_LEVEL_COLOR,
                default_module_color: DEFAULT_MODULE_COLOR,
            }
        }
    }
}

type PaintF = fn(&str) -> ANSIGenericString<'_, str>;

pub fn setup_logger_configuration(cfg: &'static CellLoggerConfiguration) {
    setup_logger_configuration_inner(|| cfg);
}

// #[cfg(atomic_cas)]
fn setup_logger_configuration_inner<'a, F>(make_f: F) -> Result<(), CellError>
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
    // [date] level (module)(file:line)
    let now = get_current_time_str();
    let mut arrs: Vec<&str> = file_str.split("/").collect();
    let mut file_info = file_str;
    let r;
    if arrs.len() > SKIP_CALLER {
        arrs.drain(0..=(arrs.len() - SKIP_CALLER));
        r = arrs.join("/");
        file_info = r.as_str();
    }
    let (level_color, module_color) = get_color(l, m.name());
    // format!("[{}] {} ({})({}:{}):{}", now, get_simple_loglevel(l), m.name(), file_info, line_no, format_msg)
    format!("[{}] {} ({})({}:{}):{}", now, level_color(get_simple_loglevel(l)), module_color(m.name()), file_info, line_no, format_msg)
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

fn get_color(l: LogLevel, module_name: &str) -> (PaintF, PaintF) {
    let level_color;
    let module_color;
    unsafe {
        match l {
            LogLevel::Trace => {
                level_color = CONFIGURATION.color_property.trace_level_color
            }
            LogLevel::Debug => {
                level_color = CONFIGURATION.color_property.debug_level_color
            }
            LogLevel::Info => level_color = {
                CONFIGURATION.color_property.info_level_color
            },
            LogLevel::Warn => level_color = {
                CONFIGURATION.color_property.warn_level_color
            },
            LogLevel::Error => level_color = {
                CONFIGURATION.color_property.error_level_color
            },
            _ => level_color = {
                CONFIGURATION.color_property.info_level_color
            },
        }
        module_color = CONFIGURATION.color_property.default_module_color
    }

    (level_color, module_color)
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
    use std::borrow::Borrow;
    use std::ops::Deref;
    use std::time::SystemTime;
    use ansi_term::ANSIGenericString;
    use ansi_term::Color::Red;
    use crate::{CellLoggerConfiguration, CONFIGURATION, DATE_FORMAT_STR, DEFAULT_DEBUG_LEVEL_COLOR, DEFAULT_ERROR_LEVEL_COLOR, default_format_msg, DEFAULT_INFO_LEVEL_COLOR, DEFAULT_TRACE_LEVEL_COLOR, DEFAULT_WARN_LEVEL_COLOR, LogLevel, Module, PaintF, setup_logger_configuration};
    use chrono::Local;
    use crate::log::{ColorProperty, Logger, LoggerEntryContext};
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

    #[test]
    fn test_color() {
        let v: ANSIGenericString<str, > = Red.paint("asd");
        let vv: &str = v.deref();
        println!("{}", vv)
    }

    #[test]
    fn test_paint() {
        let v: PaintF = |v| {
            Red.paint(v)
        };
        let ret = v("asd");
        println!("{}", ret);
    }
}
