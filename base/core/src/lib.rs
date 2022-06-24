#[macro_use]
extern crate logsdk;
extern crate core as std_core;

pub mod wrapper;
pub mod body;
pub mod summary;
pub mod channel;
pub mod reactor;
pub mod context;
pub mod request;
pub mod response;
pub mod command;
mod byte_str;
pub mod header;
pub mod input;
pub mod output;
pub mod decorator;
pub mod cerror;
pub mod constants;
pub mod suit;
pub mod dispatcher;
pub mod selector;
pub mod extension;

use std::fmt::Debug;

// pub trait ExecutorValueTrait: Debug {}

pub mod core {
    use std::fmt::Debug;
    use std::io;
    use std::rc::Rc;
    use http::header::HeaderName;
    use http::{Error, HeaderValue, Response};
    use hyper::Body;
    use tokio::sync::oneshot::{Receiver, Sender};
    use crate::wrapper::ContextResponseWrapper;


    pub type ProtocolID = &'static str;

    pub type AliasRequestType = i8;

    pub type AliasResponseType = i8;

    pub type RunType = i8;


    pub trait ExecutorValueTrait<'a>: Debug + 'a {}

    pub fn conv_protocol_to_string(p: ProtocolID) -> String {
        String::from(p as &str)
    }

    // pub trait DynClone
    // {
    //     fn clone_box(&self) -> Box<dyn Executor<'a, T> + 'a>;
    // }

    module_enums!(
        (DISPATCHER,1,&logsdk::common::LogLevel::Info);
    );
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
    fn test_module_enums(){

    }
}
