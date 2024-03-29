#[macro_use]
pub mod cmacro;

use crate::common::{LogEntry, LogLevel};
use crate::log::{Logger, MLogger};
use crate::log4rs::log_config::{setup_by_name, AppenderProperty};
use crate::module::{CellModule, Module};
use crate::{CONFIGURATION, DEFAULT_MODULE};
use lazy_static::lazy_static;
use log::{info, Log, RecordBuilder};
use log4rs::append::console::ConsoleAppender;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread::sleep;

lazy_static! {
    pub static ref DEFAULT_LOGGER: Logger =
        Logger::new(Box::new(Log4rsLogger::new(&DEFAULT_MODULE)));
}

pub struct Log4rsLogger {
    log4rs: log4rs::Logger,
}

impl MLogger for Log4rsLogger {
    fn log(&self, entry: LogEntry) {
        self.loglevel_to_log4rs(entry);
    }
}

impl Log4rsLogger {
    fn loglevel_to_log4rs(&self, entry: LogEntry) {
        let level;
        unsafe {
            if CONFIGURATION.global_loglevel.gt(entry.log_level) {
                return;
            }
        }
        match entry.log_level {
            LogLevel::Trace => {
                level = log::Level::Trace;
            }
            LogLevel::Debug => {
                level = log::Level::Debug;
            }
            LogLevel::Info => {
                level = log::Level::Info;
            }
            LogLevel::Warn => {
                level = log::Level::Warn;
            }
            LogLevel::Error => {
                level = log::Level::Error;
            }
        }
        self.log4rs.log(
            &log::Record::builder()
                .level(level)
                .args(format_args!("{}", entry.msg))
                .build(),
        );
    }
    pub fn new(m: &CellModule) -> Self {
        let module_name = m.name();
        let level = m.log_level();
        let cfg = setup_by_name(module_name, level);
        let lg4 = log4rs::Logger::new(cfg);
        let mut ret = Log4rsLogger { log4rs: lg4 };
        ret
    }
}

mod log_config {
    use crate::common::LogLevel;
    use log::{error, Level, LevelFilter, Log, Metadata, Record};
    use log4rs::append::console::ConsoleAppender;
    use log4rs::append::file::FileAppender;
    use log4rs::config;
    use log4rs::config::{Appender, Config, Logger, Root};
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::filter::Response;
    use log4rs::filter::Response::Accept;
    use std::borrow::Borrow;
    use std::fmt::Arguments;
    use std::io::stdout;
    use std::ptr::null;

    pub struct AppenderProperty {
        file_appender: bool,
        decorator: *const str,
        level: &'static LogLevel,
        appender_name: &'static str,
        pattern: Option<&'static str>,
    }

    impl AppenderProperty {
        pub fn new(
            file_appender: bool,
            decorator: *const str,
            level: &'static LogLevel,
            appender_name: &'static str,
            pattern: Option<&'static str>,
        ) -> Self {
            AppenderProperty {
                file_appender,
                decorator,
                level,
                appender_name,
                pattern,
            }
        }
    }

    pub struct ConsoleAppenderProperty {
        default_append_property: AppenderProperty,
    }

    impl ConsoleAppenderProperty {
        pub fn new(default_append_property: AppenderProperty) -> Self {
            ConsoleAppenderProperty {
                default_append_property,
            }
        }
    }

    pub fn setup_by_name(name: &'static str, l: &'static LogLevel) -> Config {
        let default_property = AppenderProperty::new(false, "/", l, name, Some("%m%n"));
        let c = &ConsoleAppenderProperty::new(default_property);
        return decorate_console_appender(c);
    }

    fn decorate_console_appender(property: &ConsoleAppenderProperty) -> config::Config {
        let pattern;
        let level;
        let level_filter;
        match property.default_append_property.pattern {
            Some(v) => pattern = v,
            None => pattern = "%m%n",
        }
        match property.default_append_property.level {
            LogLevel::Trace => {
                level = Level::Trace;
                level_filter = LevelFilter::Trace
            }
            LogLevel::Info => {
                level = Level::Info;
                level_filter = LevelFilter::Info
            }
            LogLevel::Warn => {
                level = Level::Warn;
                level_filter = LevelFilter::Warn
            }
            LogLevel::Error => {
                level = Level::Warn;
                level_filter = LevelFilter::Warn
            }
            _ => {
                level = Level::Warn;
                level_filter = LevelFilter::Warn
            }
        }

        let stdout = ConsoleAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{m}{n}")))
            // .encoder(Box::new(log4rs::PatternEncoder::new(pattern)))
            .build();

        let config = config::Config::builder()
            .appender(Appender::builder().build(
                property.default_append_property.appender_name,
                Box::new(stdout),
            ))
            .build(
                Root::builder()
                    .appender(property.default_append_property.appender_name)
                    .build(level_filter),
            )
            .unwrap();
        config
    }
}

#[cfg(test)]
mod tests {
    use crate::common::LogLevel;
    use crate::log::{Logger, LoggerEntryContext, MLogger};
    use crate::log4rs::{Log4rsLogger, DEFAULT_LOGGER};
    use crate::module::{CellModule, Module};
    use crate::{
        module, setup_logger_configuration, stack_trace, CellLoggerConfiguration, ColorProperty,
        PaintF, CONFIGURATION, DEFAULT_BLACK_LIST, DEFAULT_DEBUG_LEVEL_COLOR,
        DEFAULT_ERROR_LEVEL_COLOR, DEFAULT_INFO_LEVEL_COLOR, DEFAULT_MODULE_COLOR,
        DEFAULT_TRACE_LEVEL_COLOR, DEFAULT_WARN_LEVEL_COLOR,
    };
    use ansi_term::Color::Red;
    use backtrace::Backtrace;
    use lazy_static::lazy_static;
    use log::{info, log};
    use phf::phf_map;
    use std::borrow::Borrow;
    use std::collections::HashMap;
    use std::{thread, time};

    #[test]
    fn test_log() {
        static m: &CellModule = &module::CellModule::new(1, "asd", &LogLevel::Info);
        let l = Log4rsLogger::new(m);
        let bt = Backtrace::new();
        let (s, line) = stack_trace(&bt);
        let entry = LoggerEntryContext::create_log_entry(m, LogLevel::Info, s, line, "asdddd");
        l.log(entry);
    }

    #[test]
    fn test_logger() {
        static m: &CellModule = &module::CellModule::new(1, "LOGGER", &LogLevel::Info);
        let l = Log4rsLogger::new(m);
        let logger = Logger::new(Box::new(l));
        logger.info(m, String::from("rust msg "));
    }

    #[test]
    fn test_default_log() {
        static M: &CellModule = &module::CellModule::new(1, "LOGGER", &LogLevel::Info);
        DEFAULT_LOGGER.info(M, String::from("zzzzzzzz"));
        for i in 0..10 {
            thread::spawn(move || {
                let v = i.to_string();
                DEFAULT_LOGGER.info(M, v);
            });
        }
        DEFAULT_LOGGER.error(M, String::from("error msg"));
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis)
    }
}
