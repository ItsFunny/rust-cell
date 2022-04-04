pub mod log_hook {
    use cell_base_common;
    use cell_base_common::hook::IHook;

    pub struct LogEntry {}

    pub trait ILogHook<T>: IHook<T> {}
}