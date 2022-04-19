pub mod layout {
    use crate::hook::log_hook::LogEntry;

    pub trait IEntryLayOut {
        fn lay_out(e: &LogEntry) -> str;
    }
}