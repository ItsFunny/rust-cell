use std::rc::Rc;
use std::sync::Arc;
use chrono::Local;
use cell_core::cerror::CellResult;
use cell_core::command::{Command, CommandContext, mock_context};
use cell_core::context::{BaseBuzzContext, BuzzContextTrait};
use cell_core::dispatcher::{DefaultDispatcher, DispatchContext, Dispatcher};
use cell_core::request::{ServerRequestTrait, ServerResponseTrait};
use cell_core::summary::Summary;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use crate::request::HttpRequest;
use crate::response::HttpResponse;
use std::string::String;


pub struct HttpDispatcher
{
    m: &'static CellModule,
}

pub static DispatchModule: &CellModule = &CellModule::new(1, "HTTP_DISPATCH", &LogLevel::Info);

impl HttpDispatcher {
    pub fn new() -> Self {
        Self { m: DispatchModule }
    }
    //
    // pub fn dispatch(&mut self, req: HttpRequest, resp: HttpResponse) {
    //     let q: Box<dyn ServerRequestTrait + 'a> = Box::new(req);
    //     let b: Box<dyn ServerResponseTrait + 'a> = Box::new(resp);
    //     let ctx = DispatchContext::new(q, b);
    //     self.dispatcher.dispatch(ctx);
    // }
}

impl Dispatcher for HttpDispatcher {
    fn get_info<'a>(&self, req: Arc<Box<dyn ServerRequestTrait + 'a>>, resp: Box<dyn ServerResponseTrait + 'a>, cmd: &Command<'a>) -> CellResult<Box<dyn BuzzContextTrait<'a> + 'a>> {
        let (c, rxx, ctx) = mock_context();
        let ip = req.get_ip();
        let sequence_id = String::from("seq");
        let protocol_id = cmd.protocol_id;
        let any=req.as_any();
        let summ = Box::new(Summary::new(Arc::new(String::from(ip)), Arc::new(sequence_id), protocol_id));
        let c_ctx: CommandContext = CommandContext::new(self.m, req, resp, summ);
        let l = Local::now();
        let mut ctx = BaseBuzzContext::new(l.timestamp_millis(), c_ctx);
        let res: Box<dyn BuzzContextTrait<'a>> = Box::new(ctx);
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use bytes::Bytes;
    use chrono::Local;
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

    #[test]
    fn test_date() {
        let l = Local::now();
        println!("{}", l.to_string())
    }
}