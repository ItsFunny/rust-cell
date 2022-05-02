use std::cell::RefCell;
use std::cmp::max_by_key;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::intrinsics::size_of_val;
use std::iter::Map;
use std::rc::Rc;
use std::sync::{atomic, Mutex};
use std::sync::atomic::AtomicI32;
use cell_base_common::events::IEventResult;
use crate::consumer::LogConsumer::ILogEventConsumer;
use crate::event::event::ILogEvent;
use thread_local::ThreadLocal;
use crate::loglevel::LogLevel;
use crate::module;
use crate::module::ModuleTrait;

static CACHE: * LogCache = &LogCache {
    policy_map: Default::default(),
    policies: vec![],
    thread_local: ThreadLocal::new(),
    version: AtomicI32::new(1),
    mutex: Mutex::new(false),
};

pub struct LogPolicy {
    module_ids: Vec<i32>,
    log_types: Vec<i64>,
}


pub struct LogCache {
    policy_map: BTreeMap<i32, * LogPolicy>,
    policies: Vec<* LogPolicy>,
    thread_local: ThreadLocal<*mut LogThreadCache>,
    version: AtomicI32,
    mutex: Mutex<bool>,
}

pub struct LogThreadCache {
    log_receiver_set_cache_by_mask: HashMap<i64, HashSet<dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>>,
    log_receiver_set_cache_by_key: HashMap<i64, HashSet<dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>>,
    thread_local_cache: * ThreadLocal<* LogThreadCache>,
    version: i32,
}

impl LogThreadCache {
    pub fn new() -> &mut Self {
        let mut ret = &LogThreadCache {
            log_receiver_set_cache_by_mask: HashMap::<i64, dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>::new(),
            log_receiver_set_cache_by_key: HashMap::<i64, dyn ILogEventConsumer<dyn ILogEvent, dyn IEventResult>>::new(),
            thread_local_cache: &ThreadLocal::new(),
            version: 0,
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
    pub fn get_log_receivers(&mut self, m: Option<&dyn ModuleTrait>, l: LogLevel, log_type: i64) -> * HashSet<ILogEventConsumer<ILogEvent, IEventResult>> {
        let mut cache = self.thread_local.get_or(|| LogThreadCache::new());
        let mut ver = self.version.get_mut();
        if ver != cache.version {
            cache.log_receiver_set_cache_by_mask.drain();
            cache.log_receiver_set_cache_by_key.drain();
            cache.version = *ver;
        }
        let key = self.make_key(m, l, log_type);
        let receivers = cache.log_receiver_set_cache_by_key.get(&key);
        match receivers {
            Some(v) => return v,
            None => {}
        }
        let receive_mask = -1;
        self.mutex.lock();
        for (k, policy) in self.policy_map {}
        None
    }
    fn make_key(&self, module: Option<&dyn ModuleTrait>, l: LogLevel, log_type: i64) -> i64 {
        let module_id;
        match module {
            None => module_id = 0,
            Some(v) => module_id = v.index(),
        }
        let v = l.get_value();
        (moduleId + (v << 16) + ((log_type) << 32))
    }
}

impl LogPolicy {
    fn match_module_id(m: &dyn ModuleTrait) -> bool {}
}