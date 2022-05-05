use std::thread::sleep;
use log::{Log, RecordBuilder};
use crate::common::{LogEntry, LogLevel};
use crate::log::{Logger, MLogger};
use crate::log4rs::log_config::{AppenderProperty, setup_by_name};
use crate::module::{CellModule, Module};
use log4rs::append::console::ConsoleAppender;

pub struct Log4rsLogger {
    m: &'static CellModule,
    log4rs: log4rs::Logger,

}

// impl MLogger for Log4rsLogger {
//     fn log(entry: &LogEntry) {
//         todo!()
//     }
// }


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
        // let m = format_args!("{:}", entry.msg);
        // let rr = log::Record::builder()
        //     .level(level)
        //     .args(m);
        // let r = &rr.build();
        // self.log4rs.log(r);
        self.log4rs.log(&log::Record::builder()
            .level(level)
            .args(format_args!("{:}", entry.msg))
            .build());
    }
    pub fn new(m: &'static CellModule) -> Self {
        let module_name = m.name();
        let level = m.log_level();
        let cfg = setup_by_name(module_name, level);
        let lg4 = log4rs::Logger::new(cfg);
        let mut ret = Log4rsLogger { m, log4rs: lg4 };
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
        // if property.default_append_property.level.get_value() == LogLevel::Debug.get_value() {
        //     level = Level::Debug;
        //     level_filter = LevelFilter::Debug
        // } else if property.default_append_property.level.get_value() == LogLevel::Trace.get_value() {
        //     level = Level::Trace;
        //     level_filter = LevelFilter::Trace
        // } else if property.default_append_property.level.get_value() == LogLevel::Info.get_value() {
        //     level = Level::Info;
        //     level_filter = LevelFilter::Info
        // } else {
        //     level = Level::Error;
        //     level_filter = LevelFilter::Error
        // }

        let stdout = ConsoleAppender::builder()
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
    use crate::common::{EntryFactory, LogLevel};
    use crate::log4rs::Log4rsLogger;
    use crate::module;
    use crate::module::{CellModule, Module};

    #[test]
    fn test_log() {
        static m: &'static CellModule = &module::CellModule::new(1, "asd", &LogLevel::Info);
        let l = Log4rsLogger::new(m);
        let entry = EntryFactory::new_log_entry("asdkjkk", LogLevel::Info);
        l.loglevel_to_log4rs(entry);
        let name = l.m.name();
        println!("{}", name);
    }
}