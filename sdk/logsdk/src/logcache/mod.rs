use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::iter::Map;
use std::sync::atomic;
use std::sync::atomic::AtomicI32;
use cell_base_common::events::IEventResult;
use crate::consumer::LogConsumer::ILogEventConsumer;
use crate::event::event::ILogEvent;
use thread_local::ThreadLocal;
use crate::loglevel::LogLevel;
use crate::module::ModuleTrait;

static CACHE: * LogCache = &LogCache {
    policy_map: Default::default(),
    policies: vec![],
    thread_local: ThreadLocal::new(),
    version: AtomicI32::new(1),
};

pub struct LogPolicy {}

pub struct LogCache {
    policy_map: BTreeMap<i32, * LogPolicy>,
    policies: Vec<* LogPolicy>,
    thread_local: ThreadLocal<* LogThreadCache>,
    version: AtomicI32,
}

pub struct LogThreadCache {
    log_receiver_set_cache_by_mask: HashMap<i64, dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>,
    log_receiver_set_cache_by_key: HashMap<i64, dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>,
    thread_local_cache: ThreadLocal<* LogThreadCache>,
}

impl LogThreadCache {
    pub fn new() -> &Self {
        let ret = &LogThreadCache {
            log_receiver_set_cache_by_mask: HashMap::<i64, dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>::new(),
            log_receiver_set_cache_by_key: HashMap::<i64, dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>::new(),
            thread_local_cache: ThreadLocal::new(),
        };
    }
}
//
// impl LogCache {
//     thread_local! {
//         static thread_local_cache:RefCell<*LogThreadCache>=RefCell::new(LogThreadCache::new())
//     }
// }

impl LogCache {
    pub fn get_log_receivers(&mut self, m: &dyn ModuleTrait, l: LogLevel, log_type: i64) -> HashSet<ILogEventConsumer<ILogEvent, IEventResult>> {
        let cache = self.thread_local.get_or(|| LogThreadCache::new());
        let version = self.version.get_mut();

    }
}