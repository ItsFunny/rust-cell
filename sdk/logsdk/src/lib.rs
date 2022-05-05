pub mod module;
pub mod common;
pub mod log4rs;

pub mod log {
    use crate::common::LogEntry;
    use crate::module::Module;

    pub trait MLogger {
        fn log(entry: &LogEntry);
    }

    pub struct Logger {}

    impl MLogger for Logger {
        fn log(entry: &LogEntry) {
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
