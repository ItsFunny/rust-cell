#[macro_use]
extern crate logsdk;

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

use std::fmt::Debug;

pub trait ExecutorValueTrait: Debug {}

pub mod core {
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}