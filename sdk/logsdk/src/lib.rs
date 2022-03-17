pub mod module;

use log::Log;
use crate::logsdk::CommonLogger;

pub mod logsdk {
    use std::os::macos::raw::stat;
    use log::{Log, Metadata, Record};
    use crate::module::ModuleInterface;

    pub trait Logger {
        fn debug(m: dyn ModuleInterface, msg: String);
        fn info(m: dyn ModuleInterface, msg: String);
        fn error(m: dyn ModuleInterface, msg: String);
        fn warn(m: dyn ModuleInterface, msg: String);
    }

    pub struct CommonLogger {}

    impl Log for CommonLogger {
        fn enabled(&self, metadata: &Metadata) -> bool {
            true
        }

        fn log(&self, record: &Record) {
            println!("{}",record.level().to_string());
        }

        fn flush(&self) {}
    }

    impl Logger for CommonLogger {
        fn debug(m: Box<dyn ModuleInterface>, msg: String) {
            todo!()
        }

        fn info(m: Box<dyn ModuleInterface>, msg: String) {
            todo!()
        }

        fn error(m: Box<dyn ModuleInterface>, msg: String) {
            todo!()
        }

        fn warn(m: Box<dyn ModuleInterface>, msg: String) {
            todo!()
        }
    }
}

pub fn init_log() {
    log::set_logger(&CommonLogger {});
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
