pub mod module;
pub mod loglevel;
pub mod config;
pub mod consumer;
pub mod hook;
mod event;
pub mod layout;
mod logwrapper;

use logwrapper::{LevelFilter, Log, SetLoggerError};
use crate::logsdk::CommonLogger;

//  需求: 外部调用这个log,可以打印出module ,kv等信息
#[macro_export]
macro_rules! log {
    ( $($x:expr),* )=> {
        $(println!($x))*
    }
}
pub mod logsdk {
    use std::os::macos::raw::stat;
    use log::{debug, info, Log, Metadata, Record};
    use crate::loglevel::LogLevel;
    use crate::module::ModuleTrait;

    pub trait LogConsumer {
        fn log_able(level: LogLevel) -> bool;

    }

    pub trait Logger {
        fn debug(m: Box<dyn ModuleTrait>, msg: String);
        fn info(m: Box<dyn ModuleTrait>, msg: String);
        fn error(m: Box<dyn ModuleTrait>, msg: String);
        fn warn(m: Box<dyn ModuleTrait>, msg: String);
        fn log(&self, l: LogLevel, m: Box<dyn ModuleTrait>, msg: String);
    }

    pub struct CommonLogger {}

    impl Log for CommonLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            true
        }

        fn log(&self, record: &Record) {
            let a = record.args();
            println!("{}", record.file().unwrap());
        }

        fn flush(&self) {}
    }

    impl Logger for CommonLogger {
        fn debug(m: Box<dyn ModuleTrait>, msg: String) {
            todo!()
        }

        fn info(m: Box<dyn ModuleTrait>, msg: String) {
            todo!()
        }

        fn error(m: Box<dyn ModuleTrait>, msg: String) {
            todo!()
        }

        fn warn(m: Box<dyn ModuleTrait>, msg: String) {
            todo!()
        }

        fn log(&self, l: LogLevel, m: Box<dyn ModuleTrait>, msg: String) {
            if !self.log_able(l) {
                return;
            }
        }
    }

    impl CommonLogger {
        fn log_able(&self, logLevel: LogLevel) -> bool {
            false
        }
    }
}

pub fn init_log() {
    logwrapper::set_max_level(LevelFilter::Trace);
    logwrapper::set_logger(&CommonLogger {});
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
