#[macro_use]
extern crate logsdk;
extern crate core as std_core;

pub mod application;
mod banner;
pub mod body;
pub mod bus;
mod byte_str;
pub mod cell_macro;
pub mod cerror;
pub mod channel;
pub mod collector;
pub mod command;
pub mod constants;
pub mod context;
pub mod decorator;
pub mod di;
pub mod dispatcher;
pub mod event;
pub mod extension;
pub mod header;
pub mod input;
pub mod module;
pub mod output;
pub mod reactor;
pub mod request;
pub mod response;
pub mod selector;
pub mod suit;
pub mod summary;
pub mod wrapper;

use std::fmt::Debug;

// pub trait ExecutorValueTrait: Debug {}

pub mod core {
    use crate::wrapper::ContextResponseWrapper;
    use http::header::HeaderName;
    use http::{Error, HeaderValue, Response};
    use hyper::Body;
    use std::fmt::Debug;
    use std::io;
    use std::rc::Rc;
    use tokio::sync::oneshot::{Receiver, Sender};

    pub type ProtocolID = &'static str;

    pub type AliasRequestType = i8;

    pub type AliasResponseType = i8;

    pub type RunType = i8;

    pub const runTypeHttp: RunType = 1 as RunType;
    pub const runTypeHttpPost: RunType = runTypeHttp << 1 | runTypeHttp;
    pub const runTypeHttpGet: RunType = runTypeHttp << 2 | runTypeHttp;

    pub trait ExecutorValueTrait<'a>: Debug + 'a {}

    pub fn conv_protocol_to_string(p: ProtocolID) -> String {
        String::from(p as &str)
    }

    // pub trait DynClone
    // {
    //     fn clone_box(&self) -> Box<dyn Executor<'a, T> + 'a>;
    // }
}

#[cfg(test)]
mod tests {
    use logsdk::common::LogLevel;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_module_enums() {}
}
