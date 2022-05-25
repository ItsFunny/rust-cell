use std::collections::HashMap;
use std::marker::PhantomData;
use bytes::Bytes;
use rocket::response::Body;
use crate::constants::EnumsProtocolStatus;
use crate::output::OutputArchive;

pub struct ContextResponseWrapper<'a> {
    status: Option<&'static EnumsProtocolStatus>,
    headers: HashMap<String, String>,
    body: Option<Bytes>,
    _prv_r: PhantomData<&'a ()>,
}


impl<'a> ContextResponseWrapper<'a> {
    #[inline(always)]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn status(&self) -> Option<&EnumsProtocolStatus> {
        self.status
    }
    #[inline(always)]
    pub fn body_mut(self) -> Option<Bytes> {
        self.body
    }
    pub fn with_status(mut self, s: &'static EnumsProtocolStatus) -> Self {
        self.status = Some(s);
        self
    }
    pub fn with_body(mut self, b: Bytes) -> Self {
        self.body = Some(b);
        self
    }
    pub fn with_header(mut self) -> Self {
        self
    }
}

impl Default for ContextResponseWrapper<'_> {
    fn default() -> Self {
        ContextResponseWrapper {
            status: None,
            headers: Default::default(),
            body: None,
            _prv_r: Default::default(),
        }
    }
}