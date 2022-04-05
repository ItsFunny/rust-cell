pub mod LogConsumer {
    use std::fmt::Error;
    use log::error;
    use cell_base_common;
    use cell_base_common::consumer::IConsumer;
    use cell_base_common::events::{IEvent, IEventResult};
    use crate::consumer::ILogEvent;
    use crate::hook::log_hook;
    use crate::hook::log_hook::{ILogHook, LogEntry};
    use crate::loglevel::LogLevel;

    pub trait ILogEventConsumer<T, V>: IConsumer<T, V>
        where
            T: ILogEvent,
            V: IEventResult
    {
        fn log_able(&self, l: &LogLevel) -> bool;
    }


    pub struct DefaultLogConsumer
    {
        hooks: Vec<dyn ILogHook<LogEntry>>,
    }

    impl ILogEventConsumer<T, V> for DefaultLogConsumer {
        fn log_able(&self, l: &LogLevel) -> bool {
            true
        }
    }

    impl<T: ILogEvent, V: IEventResult> IConsumer<T, V> for DefaultLogConsumer
    {
        fn consume(&self, event: T) -> Option<V> {
            let entry = event.get_log_entry();
            if !self.log_able(&entry.log_level) {
                None
            }

            None
        }
    }


    pub trait ILogEvent: IEvent {
        fn get_log_entry(&self) -> &LogEntry;
    }


    pub struct DefaultLogEvent {
        log_entry: LogEntry,
    }

    impl DefaultLogEvent {
        pub fn new(logEntry: LogEntry) -> Self {
            DefaultLogEvent { log_entry: logEntry }
        }
    }

    impl IEvent for DefaultLogEvent {}

    impl ILogEvent for DefaultLogEvent {
        fn get_log_entry(&self) -> &LogEntry {
            &self.log_entry
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
