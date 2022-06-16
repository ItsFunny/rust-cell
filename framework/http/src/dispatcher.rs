use cell_core::dispatcher::{DefaultDispatcher, DispatchContext};
use cell_core::request::{ServerRequestTrait, ServerResponseTrait};
use cell_core::suit::CommandSuit;
use crate::request::HttpRequest;
use crate::response::HttpResponse;


pub struct HttpDispatcher<'e, 'a>
{
    dispatcher: DefaultDispatcher<'e, 'a>,
}

impl<'e, 'a> HttpDispatcher<'e, 'a> {
    pub fn new(dispatcher: DefaultDispatcher<'e, 'a>) -> Self {
        Self { dispatcher }
    }

    pub fn dispatch(&mut self, req: HttpRequest, resp: HttpResponse) {
        let q: Box<dyn ServerRequestTrait + 'a> = Box::new(req);
        let b: Box<dyn ServerResponseTrait + 'a> = Box::new(resp);
        let ctx = DispatchContext::new(q, b);
        self.dispatcher.dispatch(ctx);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use bytes::Bytes;
    use http::header::HeaderName;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}