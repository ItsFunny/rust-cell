mod cell_macro;

use std::thread::sleep;
use log::{info, Log, RecordBuilder};
use crate::common::{LogEntry, LogLevel};
use crate::log::{Logger, MLogger};
use crate::log4rs::log_config::{AppenderProperty, setup_by_name};
use crate::module::{CellModule, Module};
use log4rs::append::console::ConsoleAppender;

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
        self.log4rs.log(&log::Record::builder()
            .level(level)
            .args(format_args!("{}", entry.msg))
            .build());
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
    use std::borrow::Borrow;
    use std::fmt::Arguments;
    use std::io::stdout;
    use std::ptr::null;
    use log::{error, Level, LevelFilter, Log, Metadata, Record};
    use log4rs::append::console::ConsoleAppender;
    use log4rs::append::file::FileAppender;
    use log4rs::config;
    use log4rs::encode::pattern::PatternEncoder;
    use log4rs::config::{Appender, Config, Logger, Root};
    use log4rs::filter::Response;
    use log4rs::filter::Response::Accept;
    use crate::common::LogLevel;

    pub struct AppenderProperty {
        file_appender: bool,
        decorator: *const str,
        level: &'static LogLevel,
        appender_name: &'static str,
        pattern: Option<&'static str>,
    }

    impl AppenderProperty {
        pub fn new(file_appender: bool, decorator: *const str, level: &'static LogLevel, appender_name: &'static str, pattern: Option<&'static str>) -> Self {
            AppenderProperty { file_appender, decorator, level, appender_name, pattern }
        }
    }


    pub struct ConsoleAppenderProperty {
        default_append_property: AppenderProperty,
    }

    impl ConsoleAppenderProperty {
        pub fn new(default_append_property: AppenderProperty) -> Self {
            ConsoleAppenderProperty { default_append_property }
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
            .appender(Appender::builder().build(property.default_append_property.appender_name, Box::new(stdout)))
            .build(Root::builder().appender(property.default_append_property.appender_name).build(level_filter))
            .unwrap();
        config
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use backtrace::Backtrace;
    use crate::common::{LogLevel};
    use crate::log4rs::Log4rsLogger;
    use crate::log::{Logger, LoggerEntryContext, MLogger};
    use crate::{module, stack_trace};
    use crate::module::{CellModule, Module};

    #[test]
    fn test_log() {
        static m: &CellModule = &module::CellModule::new(1, "asd", &LogLevel::Info);
        let l = Log4rsLogger::new(m);
        let bt = Backtrace::new();
        let (s, line) = stack_trace(&bt);
        let entry = LoggerEntryContext::create_log_entry(m, LogLevel::Info, s, line, "asdddd");
        l.log(entry);
        // let entry = EntryFactory::new_log_entry("asdkjkk", LogLevel::Info);
        // l.loglevel_to_log4rs(&entry);
        // let name = l.m.name();
        // println!("{}", name);
    }

    #[test]
    fn test_logger() {
        static m: &CellModule = &module::CellModule::new(1, "LOGGER", &LogLevel::Info);
        let l = Log4rsLogger::new(m);
        let logger = Logger::new(Box::new(l));
        logger.info(m, String::from("rust msg "));
    }
}