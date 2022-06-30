use core::any::Any;
use std::fmt::{Display, Formatter, write};

pub trait Event: Send + Sync + Display {
    fn as_any(&self) -> &dyn Any;
}

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