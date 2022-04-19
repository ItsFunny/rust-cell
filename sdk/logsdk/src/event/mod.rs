pub mod event {
    use cell_base_common::events::IEvent;
    use crate::hook::log_hook::LogEntry;

    pub trait ILogEvent: IEvent {
        fn get_log_entry(&self) -> *LogEntry;
    }


    pub struct DefaultLogEvent {
        log_entry: *LogEntry,
    }

    impl DefaultLogEvent {
        pub fn new(logEntry: *LogEntry) -> Self {
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