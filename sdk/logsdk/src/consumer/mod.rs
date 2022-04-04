pub mod LogConsumer {
    use cell_base_common;
    use cell_base_common::consumer::IConsumer;
    use cell_base_common::events::{IEvent, IEventResult};
    use crate::hook::log_hook;
    use crate::hook::log_hook::{ILogHook, LogEntry};
    use crate::loglevel::LogLevel;

    pub trait ILogEventConsumer<T, V>: IConsumer<T, V>
        where
            T: IEvent,
            V: IEventResult
    {
        fn log_able(&self, l: LogLevel) -> bool;
    }


    pub struct DefaultLogConsumer
    {
        hooks: Vec<dyn ILogHook<LogEntry>>,
    }

    impl ILogEventConsumer<T, V> for DefaultLogConsumer {
        fn log_able(&self, l: LogLevel) -> bool {
            true
        }
    }

    impl IConsumer<T, V> for DefaultLogConsumer {
        fn consume(_: T) -> V {

        }
    }

    impl DefaultLogConsumer {
        fn v(&self) {
            self.log_able(LogLevel::ERROR);
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
