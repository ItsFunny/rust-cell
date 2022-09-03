use std::borrow::BorrowMut;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::format;
use std::sync::{Arc, Mutex, RwLock};
use crossbeam::channel::{Receiver, Select, Sender, unbounded};
use crossbeam::deque::Steal::Retry;
use futures::stream::SelectNextSome;
use regex::Regex;
use rocket::figment::map;
use rocket::http::ext::IntoCollection;
use tokio::runtime::Runtime;
use crate::cerror::{CellError, CellResult, ErrorEnums, ErrorEnumsStruct};

type operation = i8;

const subscribe: operation = 1;
const publish: operation = 2;
const unsubscribe: operation = 3;
const shutdown: operation = 4;


pub struct EventBus<T>
    where
        T: Send + Sync + 'static
{
    runtime: Arc<tokio::runtime::Runtime>,
    cmds: Sender<cmd<T>>,
    receivers: Receiver<cmd<T>>,
    cmd_cap: i32,
    mtx: RwLock<u8>,
    subscriptions: HashMap<String, HashSet<&'static str>>,
}


impl<T> Clone for EventBus<T>
    where
        T: Send + Sync + 'static
{
    fn clone(&self) -> Self {
        EventBus {
            runtime: self.runtime.clone(),
            cmds: self.cmds.clone(),
            receivers: self.receivers.clone(),
            cmd_cap: self.cmd_cap,
            mtx: Default::default(),
            subscriptions: self.subscriptions.clone(),
        }
    }
}

// type Query interface {
// Matches(events map[string][]string) (bool, error)
// String() string
// }
pub trait Query: Send + Sync + 'static {
    fn matches(&self, events: &HashMap<String, Vec<String>>) -> bool;
    fn String(&self) -> &'static str;
}

pub struct DefaultRegexQuery {
    id: &'static str,
    reg: Regex,
    events: HashSet<String>,
}

impl DefaultRegexQuery {
    pub fn new(id: &'static str, str: String, events: HashSet<String>) -> DefaultRegexQuery {
        let reg = Regex::new(str.as_str()).map_err(|e| {
            panic!("illegal regex");
        }).unwrap();
        let ret = DefaultRegexQuery { id: id, reg: reg, events };
        return ret;
    }
}

impl Query for DefaultRegexQuery {
    fn matches(&self, events: &HashMap<String, Vec<String>>) -> bool {
        for (k, v) in events {
            if !self.reg.is_match(k.as_ref()) {
                return false;
            }

            for event_str in v {
                if self.events.contains(event_str.as_str()) {
                    return true;
                }
            }
        }

        return false;
    }

    fn String(&self) -> &'static str {
        self.id
    }
}

struct cmd<T> {
    operation: operation,
    query: Option<Arc<Box<dyn Query>>>,
    subscription: Option<Arc<SubscriptionImpl<T>>>,
    client_id: String,
    data: Option<T>,
    events: HashMap<String, Vec<String>>,
}

struct state<T> {
    subscriptions: HashMap<&'static str, HashMap<String, Arc<SubscriptionImpl<T>>>>,
    queries: HashMap<&'static str, queryPlusRefCount>,
}

impl<T> state<T> {
    fn add(&mut self, client_id: String, q: Arc<Box<dyn Query>>, sub: Arc<SubscriptionImpl<T>>) {
        let q_str = q.String();
        if !self.subscriptions.contains_key(q_str) {
            self.subscriptions.insert(q_str, HashMap::new());
        }
        self.subscriptions.get_mut(q_str).unwrap().insert(client_id, sub);

        let query_res = self.queries.get_mut(q_str);
        match query_res {
            Some(v) => {
                v.ref_count = v.ref_count + 1;
            }
            None => {
                self.queries.insert(q_str, queryPlusRefCount { q, ref_count: 0 });
            }
        }
    }
    fn send(&mut self, data: Arc<T>, evens: &HashMap<String, Vec<String>>) {
        for (k, v) in &self.subscriptions {
            let query = self.queries.get(k).unwrap();
            if query.q.matches(evens) {
                for (k2, v2) in v {
                    // TODO, remove client
                    // TODO,handle error
                    v2.out.send(data.clone());
                }
            }
        }
    }
}

struct queryPlusRefCount {
    q: Arc<Box<dyn Query>>,
    ref_count: u32,
}

unsafe impl<T> Send for cmd<T> {}

unsafe impl<T> Sync for cmd<T> {}

pub struct SubscriptionImpl<T> {
    out: Sender<Arc<T>>,
    mtx: RwLock<u8>,
    block: bool,
    err: Option<Box<dyn Error>>,
}

type SubscriptionOption<T> = dyn FnMut(&SubscriptionImpl<T>);

impl<T> SubscriptionImpl<T> {
    pub fn new(out: Sender<Arc<T>>, mut ops: Option<Box<SubscriptionOption<T>>>) -> Self {
        let mut ret = SubscriptionImpl {
            out: out,
            mtx: Default::default(),
            block: false,
            err: None,
        };
        if let Some(mut v) = ops {
            v(&ret);
        }
        ret
    }
}


impl<T> EventBus<T>
    where
        T: Send + Sync + 'static
{
    pub fn new(rt: Arc<Runtime>) -> EventBus<T> {
        let (sender, receiver) = unbounded::<cmd<T>>();
        let ret = EventBus {
            runtime: rt.clone(),
            cmds: sender,
            receivers: receiver,
            cmd_cap: 0,
            mtx: Default::default(),
            subscriptions: Default::default(),
        };
        ret
    }
    fn start(mut self) {
        self.runtime.clone().spawn(self.do_start());
    }

    pub fn publish(&self, msg: T, events: HashMap<String, Vec<String>>) {
        // TODO ,handle error
        let res = self.cmds.send(cmd {
            operation: publish,
            query: None,
            subscription: None,
            client_id: "".to_string(),
            data: Some(msg),
            events,
        });
    }
    pub fn subscribe(&mut self, client_id: String, cap: usize, query: Box<dyn Query>, ops: Option<Box<SubscriptionOption<T>>>) -> CellResult<Arc<Receiver<Arc<T>>>> {
        {
            self.mtx.read().unwrap();
            let contains = self.subscriptions.get(client_id.clone().as_str());
            if let Some(v) = contains {
                if v.contains(query.String()) {
                    return Err(CellError::from(ErrorEnumsStruct::EVENT_BUS_DUPLICATE_CLIENTID));
                }
            }
        }


        let (sender, receiver) = crossbeam::channel::bounded(cap);

        let sub_impl = SubscriptionImpl::new(sender, ops);
        let q = Arc::new(query);
        let res = self.cmds.send(cmd {
            operation: subscribe,
            query: Some(q.clone()),
            subscription: Some(Arc::new(sub_impl)),
            client_id: client_id.clone(),
            data: None,
            events: Default::default(),
        });
        match res {
            Err(e) => {
                return Err(CellError::from(ErrorEnumsStruct::EVENT_BUS_SUBSCRIBE_FAILED).with_error(Box::new(e)));
            }
            Ok(v) => {
                self.mtx.write().unwrap();
                let contains = self.subscriptions.contains_key(client_id.as_str());
                if !contains {
                    self.subscriptions.insert(client_id.clone(), HashSet::new());
                }
                let v = self.subscriptions.get_mut(client_id.clone().as_str()).unwrap();
                v.insert(q.clone().String());
            }
        }

        Ok(Arc::new(receiver))
    }
    async fn do_start(mut self) {
        self.do_loop(state { subscriptions: Default::default(), queries: Default::default() });
    }
    fn do_loop(&mut self, mut st: state<T>) {
        let clone_sub = self.receivers.clone();
        let mut sel = Select::new();
        sel.recv(&clone_sub);

        loop {
            let index = sel.ready();
            let res = clone_sub.try_recv();
            // If the operation turns out not to be ready, retry.
            if let Err(e) = res {
                if e.is_empty() {
                    continue;
                }
            }
            let v = res.unwrap();
            match v.operation {
                subscribe => {
                    st.add(v.client_id, v.query.unwrap(), v.subscription.unwrap().clone())
                }
                publish => {
                    st.send(Arc::new(v.data.unwrap()), &v.events)
                }
                // TODO
                unsubscribe => {}
                shutdown => {}
                _ => {}
            }
        }
    }
}

unsafe impl<T> Send for EventBus<T>
    where
        T: Send + Sync + 'static
{}

unsafe impl<T> Sync for EventBus<T>
    where
        T: Send + Sync + 'static
{}


#[cfg(test)]
mod tests {
    use core::future::Future;
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use tokio::sync::broadcast::Receiver;
    use crate::module::ModuleEnumsStruct;
    use logsdk::common::LogLevel;
    use logsdk::module::CellModule;
    use crate::bus::{DefaultRegexQuery, EventBus};

    #[test]
    fn test_bus() {
        let mut runtime = tokio::runtime::Runtime::new().unwrap();
        let arc_run = Arc::new(runtime);
        let mut bus = EventBus::<u8>::new(arc_run.clone());
        bus.clone().start();

        let clone_bus = bus.clone();
        let client_id = String::from("client_id");
        let cap = 10;
        let mut set = HashSet::<String>::new();
        set.insert(String::from("event1"));
        let q = Box::new(DefaultRegexQuery::new("test_id", String::from("client_id*"), set));
        let recv_res = clone_bus.clone().subscribe(client_id.clone(), cap, q, None);
        let recv = recv_res.unwrap();

        let mut set2 = HashSet::<String>::new();
        set2.insert(String::from("event1"));
        let q2 = Box::new(DefaultRegexQuery::new("test_id2", String::from("client_id*"), set2));
        let recv2 = clone_bus.clone().subscribe(client_id.clone(), cap, q2, None).unwrap();

        let send = clone_bus.clone();
        arc_run.clone().spawn(async move {
            thread::sleep(Duration::from_secs(3));
            let mut events = HashMap::new();
            events.insert(client_id.clone(), vec![String::from("event1")]);
            send.publish(1, events);
        });

        let send2 = clone_bus.clone();

        arc_run.clone().spawn(async move {
            thread::sleep(Duration::from_secs(2));
            let ret = recv.recv();
            match ret {
                Err(e) => {
                    println!("err")
                }
                Ok(v) => {
                    cinfo!(ModuleEnumsStruct::EXTENSION,"msg1:{}",v);
                }
            }
        });

        arc_run.clone().spawn(async move {
            thread::sleep(Duration::from_secs(2));
            let ret = recv2.recv();
            match ret {
                Err(e) => {
                    println!("err")
                }
                Ok(v) => {
                    cinfo!(ModuleEnumsStruct::EXTENSION,"msg2:{}",v);
                }
            }
        });


        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn test_tokio_broadcast() {
        let (sender, receiver) = tokio::sync::broadcast::channel::<u8>(10);
        let r2 = receiver.resubscribe();
        let rec = Arc::new(receiver);
        let mut runtime = tokio::runtime::Runtime::new().unwrap();

        let rec1 = rec.clone();
        let c1 = async move {
            let mut rrr = rec1.clone().resubscribe();
            let v = rrr.recv().await.unwrap();
            cinfo!(ModuleEnumsStruct::EXTENSION,"msg:{}",v);
        };

        let rec2 = rec.clone();
        let c2 = async move {
            let mut rrr = rec2.clone().resubscribe();
            let v = rrr.recv().await.unwrap();
            cinfo!(ModuleEnumsStruct::EXTENSION,"msg:{}",v);
        };
        let rec3 = rec.clone();
        let c3 = async move {
            let mut rrr = rec3.clone().resubscribe();
            let v = rrr.recv().await.unwrap();
            cinfo!(ModuleEnumsStruct::EXTENSION,"msg:{}",v);
        };
        let f = async move {
            thread::sleep(Duration::from_secs(2));
            sender.send(1).unwrap();
        };
        runtime.spawn(f);
        runtime.spawn(c1);
        runtime.spawn(c2);
        runtime.spawn(c3);
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    fn test_flume() {
        let (sender, receiver) = flume::unbounded::<u8>();
        let run = Arc::new(tokio::runtime::Runtime::new().unwrap());


        let run_time = Arc::new(tokio::runtime::Runtime::new().unwrap());

        let r1 = receiver.clone();
        let r2 = receiver.clone();
        let r3 = receiver.clone();
        sender.send(1).unwrap();
        let r1_res = r1.recv();
        match r1_res {
            Err(e) => {
                cinfo!(ModuleEnumsStruct::EXTENSION,"err");
            }
            Ok(v) => {
                cinfo!(ModuleEnumsStruct::EXTENSION,"msg:{}",v);
            }
        }

        let r2_res = r2.recv();
        match r2_res {
            Err(e) => {
                cinfo!(ModuleEnumsStruct::EXTENSION,"err");
            }
            Ok(v) => {
                cinfo!(ModuleEnumsStruct::EXTENSION,"msg:{}",v);
            }
        }

        let r3_res = r3.recv();
        match r3_res {
            Err(e) => {
                cinfo!(ModuleEnumsStruct::EXTENSION,"err");
            }
            Ok(v) => {
                cinfo!(ModuleEnumsStruct::EXTENSION,"msg:{}",v);
            }
        }
    }
}