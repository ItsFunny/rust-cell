pub mod LogConsumer {
    use std::fmt::{Arguments, Error};
    use log::{Level, Log};
    use cell_base_common;
    use cell_base_common::consumer::IConsumer;
    use cell_base_common::events::{IEvent, IEventResult};
    use crate::consumer::cell_log4rs::cell_log4rs::buildCfg;
    use crate::consumer::ILogEvent;
    use crate::event::event::ILogEvent;
    use crate::hook::log_hook;
    use crate::hook::log_hook::{ILogHook, LogEntry};
    use crate::layout::layout::IEntryLayOut;
    use crate::loglevel::LogLevel;

    pub trait ILogEventConsumer<T, V>: IConsumer<T, V>
        where
            T: ILogEvent,
            V: IEventResult
    {
    }


    pub struct DefaultLogConsumer
    {
        hooks: Vec<dyn ILogHook<LogEntry>>,
        lay_out: IEntryLayOut,
        log4rs: log4rs::Logger,
    }

    impl ILogEventConsumer<T, V> for DefaultLogConsumer {
    }

    impl<T: ILogEvent, V: IEventResult> IConsumer<T, V> for DefaultLogConsumer
    {
        fn consume(&self, event: T) -> Option<V> {
            let entry = event.get_log_entry();
            if !self.hooks.is_empty() {
                for h in self.hooks.iter() {}
            }
            self.loglevel_to_log4rs(entry);
            None
        }
    }

    impl DefaultLogConsumer {
        fn loglevel_to_log4rs(self, entry: *LogEntry) {
            let level;
            match entry.log_level {
                LogLevel::TRACE => {
                    level = log::Level::Trace;
                }
                LogLevel::DEBUG => {
                    level = log::Level::Debug;
                }
                LogLevel::INFO => {
                    level = log::Level::Info;
                }
                LogLevel::WARN => {
                    level = log::Level::Warn;
                }
                LogLevel::ERROR => {
                    level = log::Level::Error;
                }
            }

            let record = log::Record::builder()
                .level(level)
                .args(format_args!(entry.msg))
                .build();
            self.log4rs.log(&record);
            ;
        }
        pub fn new() -> Self {
            let cfg = log_config::setup_by_name("info", Level::Info);
            let logrs = log4rs::Logger::new(cfg);
            let ret = DefaultLogConsumer {
                hooks: vec![],
                lay_out: (),
                log4rs: logrs,
            };
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

        pub struct AppenderProperty {
            file_appender: bool,
            decorator: * str,
            level: Level,
            appender_name: * str,
            pattern: Option<* str>,

        }

        impl AppenderProperty {
            pub fn new(file_appender: bool, decorator: *const str, level: Level, appender_name: *const str, pattern: Option<*const str>) -> Self {
                AppenderProperty { file_appender, decorator, level, appender_name, pattern }
            }
        }


        pub struct ConsoleAppenderProperty {
            default_append_property: * AppenderProperty,
        }

        impl ConsoleAppenderProperty {
            pub fn new(default_append_property: *const AppenderProperty) -> Self {
                ConsoleAppenderProperty { default_append_property }
            }
        }

        // pub fn buildCfg() -> log4rs::Config {
        //     let stdout = log4rs::ConsoleAppender::builder()
        //         .encoder(Box::new(log4rs::PatternEncoder::new("[Console] {d} - {l} -{t} - {m}{n}")))
        //         .build();
        //
        //     let file = log4rs::FileAppender::builder()
        //         .encoder(Box::new(log4rs::PatternEncoder::new("[File] {d} - {l} - {t} - {m}{n}")))
        //         .build("log/test.log")
        //         .unwrap();
        //
        //     let config = log4rs::Config::builder()
        //         .appender(Appender::builder().build("stdout", Box::new(stdout)))
        //         .appender(Appender::builder().build("file", Box::new(file)))
        //         .logger(Logger::builder()
        //             .appender("file")
        //             .additive(false)
        //             .build("app", LevelFilter::Info))
        //         .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        //         .unwrap();
        //     config
        // }


        pub fn setup_by_name(name: * str, l: Level) -> config::Config {
            let default_property = &AppenderProperty::new(false, "/", l, name, Some("%m%n"));
            let c = &ConsoleAppenderProperty::new(default_property);
            decorate_console_appender(c);
        }

        fn decorate_console_appender(property: * ConsoleAppenderProperty) -> log4rs::Config {
            let pattern;
            let level;
            let level_filter;
            match property.default_append_property.pattern {
                Some(v) => pattern = v,
                None => pattern = "%m%n",
            }
            if property.default_append_property.level == Level::Debug {
                level = Level::Debug;
                level_filter = LevelFilter::Debug
            } else if property.default_append_property.level == Level::Trace {
                level = Level::Trace;
                level_filter = LevelFilter::Trace
            } else if property.default_append_property.level == Level::Info {
                level = Level::Info;
                level_filter = LevelFilter::Info
            } else {
                level = Level::Error;
                level_filter = LevelFilter::Error
            }

            let stdout = log4rs::ConsoleAppender::builder()
                .encoder(Box::new(log4rs::PatternEncoder::new(pattern)))
                .build();

            let config = log4rs::Config::builder()
                .appender(Appender::builder().build(property.default_append_property.appender_name, Box::new(stdout)))
                .build(Root::builder().appender(property.default_append_property.appender_name).build(level_filter))
                .unwrap();
            config
        }
    }
}


#[cfg(test)]
mod tests {
    use cell_base_common::consumer::IConsumer;
    use super::*;
    use LogConsumer::*;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_console() {
        let c = DefaultLogConsumer::new();
        c.consume()
    }
}
