use crate::request::HttpRequest;
use crate::response::HttpResponse;
use cell_core::cerror::CellResult;
use cell_core::command::{mock_context, Command, CommandContext};
use cell_core::context::{BaseBuzzContext, BuzzContextTrait};
use cell_core::dispatcher::{DefaultDispatcher, DispatchContext, Dispatcher};
use cell_core::request::{ServerRequestTrait, ServerResponseTrait};
use cell_core::summary::Summary;
use chrono::Local;
use logsdk::common::LogLevel;
use logsdk::module::CellModule;
use std::rc::Rc;
use std::string::String;
use std::sync::Arc;

pub struct HttpDispatcher {
    m: &'static CellModule,
}

pub static DispatchModule: &CellModule = &CellModule::new(1, "HTTP_DISPATCH", &LogLevel::Info);

impl HttpDispatcher {
    pub fn new() -> Self {
        Self { m: DispatchModule }
    }
}

impl Dispatcher for HttpDispatcher {
    fn get_info<'a>(
        &self,
        req: Arc<Box<dyn ServerRequestTrait + 'a>>,
        resp: Box<dyn ServerResponseTrait + 'a>,
        cmd: &Command<'a>,
    ) -> Box<dyn BuzzContextTrait<'a> + 'a> {
        let (c, rxx, ctx) = mock_context();
        let ip = req.get_ip();
        let sequence_id = String::from("seq");
        let protocol_id = cmd.protocol_id;
        let any = req.as_any();
        let summ = Box::new(Summary::new(
            Arc::new(String::from(ip)),
            Arc::new(sequence_id),
            protocol_id,
        ));
        let c_ctx: CommandContext = CommandContext::new(self.m, req, resp, summ);
        let l = Local::now();
        let mut ctx = BaseBuzzContext::new(l.timestamp_millis(), c_ctx);
        let res: Box<dyn BuzzContextTrait<'a>> = Box::new(ctx);
        res
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use chrono::Local;
    use http::header::HeaderName;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use std::sync::Arc;

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
