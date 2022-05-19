use std::collections::HashMap;
use std::marker::PhantomData;
use rocket::response::Body;
use crate::output::OutputArchive;

pub struct ContextResponseWrapper<'r> {
    status: Option<i32>,
    headers: HashMap<String, String>,
    body: Option<Box<dyn OutputArchive + 'r>>,
    _prv_r: PhantomData<&'r ()>,
}


impl<'r> ContextResponseWrapper<'r> {
    #[inline(always)]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    #[inline(always)]
    pub fn body_mut(&mut self) -> &Option<Box<dyn OutputArchive + 'r>> {
        &self.body
    }
    pub fn with_status(mut self, s: i32) -> Self {
        self.status = Some(s);
        self
    }
    pub fn with_body(mut self, b: Box<dyn OutputArchive>) -> Self {
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