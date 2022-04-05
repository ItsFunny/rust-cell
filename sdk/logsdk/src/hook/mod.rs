pub mod log_hook {
    use cell_base_common;
    use cell_base_common::hook::IHook;
    use crate::loglevel::LogLevel;

    pub struct LogEntry {
        pub log_level: LogLevel

    }

    pub trait ILogHook<T>: IHook<T> {}
}