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
pub struct ApplicationInitEvent {
    pub cb: Box<dyn Fn() + Send + Sync + 'static>,
}

impl ApplicationInitEvent {
    pub fn new(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        Self { cb: cb }
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
pub struct ApplicationStartedEvent {
    pub cb: Box<dyn Fn() + Send + Sync + 'static>,
}

impl ApplicationStartedEvent {
    pub fn new(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        Self { cb: cb }
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
pub struct ApplicationReadyEvent {
    pub cb: Box<dyn Fn() + Send + Sync + 'static>,
}

impl ApplicationReadyEvent {
    pub fn new(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        Self { cb: cb }
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
pub struct ApplicationCloseEvent {
    pub cb: Box<dyn Fn() + Send + Sync + 'static>,
}

impl ApplicationCloseEvent {
    pub fn new(cb: Box<dyn Fn() + Send + Sync + 'static>) -> Self {
        Self { cb: cb }
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
}

impl NextStepEvent {
    pub fn new(current: u8) -> Self {
        Self { current }
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