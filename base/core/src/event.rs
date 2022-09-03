use core::any::Any;
use core::future::Future;
use std::fmt::{Display, Formatter, write};
use std::sync::{Arc, mpsc, Mutex};
use std_core::cell::RefCell;
use flo_stream::{MessagePublisher, Publisher};
use tokio::runtime::Runtime;

pub trait Event: Send + Sync + Display {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Clone)]
pub struct ApplicationEnvironmentPreparedEvent {
    pub args: Vec<String>,
}

impl ApplicationEnvironmentPreparedEvent {
    pub fn new(args: Vec<String>) -> Self {
        Self { args }
    }
}

impl Display for ApplicationEnvironmentPreparedEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("");
        for v in &self.args {
            s += v;
        }
        write!(f, "ApplicationEnvironmentPreparedEvent msg,args:{}", s)
    }
}

impl Event for ApplicationEnvironmentPreparedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}


/////////
pub struct ApplicationInitEvent {}

impl ApplicationInitEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for ApplicationInitEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApplicationInitEvent msg")
    }
}

impl Event for ApplicationInitEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

//////////////
pub struct ApplicationStartedEvent {}

impl ApplicationStartedEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for ApplicationStartedEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApplicationStartedEvent msg")
    }
}

impl Event for ApplicationStartedEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

///////
pub struct ApplicationReadyEvent {}

impl ApplicationReadyEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for ApplicationReadyEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApplicationReadyEvent msg")
    }
}

impl Event for ApplicationReadyEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

////////
pub struct ApplicationCloseEvent {}

impl ApplicationCloseEvent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for ApplicationCloseEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApplicationCloseEvent msg")
    }
}

impl Event for ApplicationCloseEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}


///////////
pub struct NextStepEvent {
    pub current: u8,
    pub next: Arc<dyn Event>,
}

impl NextStepEvent {
    pub fn new(current: u8, next: Arc<dyn Event>) -> Self {
        Self { current, next }
    }
}

impl Display for NextStepEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "next step msg")
    }
}

impl Event for NextStepEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for NextStepEvent {
    fn clone(&self) -> Self {
        NextStepEvent { current: self.current, next: self.next.clone() }
    }
}

type Callback = Box<dyn Fn() + Send + Sync>;

pub struct CallBackEvent {
    pub cb: Callback,
}

impl CallBackEvent {
    pub fn new(cb: Callback) -> Self {
        Self { cb }
    }
}

impl Display for CallBackEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "call back event")
    }
}

impl Event for CallBackEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

unsafe impl Send for CallBackEvent {}

unsafe impl Sync for CallBackEvent {}