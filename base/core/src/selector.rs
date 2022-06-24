use core::marker::PhantomData;
use core::ops::Deref;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std_core::cell::RefCell;
use rocket::figment::map;
use pipeline2::pipeline2::DefaultPipelineV2;
use crate::cerror::CellResult;
use crate::command::*;
use crate::core::{conv_protocol_to_string, ExecutorValueTrait};
use crate::request::{MockRequest, ServerRequestTrait};

pub trait CommandSelector<'a>: Send + Sync {
    // FIXME , reference or clone ?
    fn select(&self, req: &SelectorRequest) -> Option<Command<'a>>;
    fn on_register_cmd(&mut self, cmd: Command<'a>);
}

pub struct SelectorRequest<'a> {
    pub request: Arc<Box<dyn ServerRequestTrait + 'a>>,
    pub tx: Sender<Command<'a>>,
    // TODO wrap tx
    pub done: RefCell<bool>,
}

impl<'a> SelectorRequest<'a> {
    pub fn new(request: Arc<Box<dyn ServerRequestTrait + 'a>>, tx: Sender<Command<'a>>) -> Self {
        SelectorRequest { request, tx, done: RefCell::new(false) }
    }
}

unsafe impl<'a> Send for SelectorRequest<'a> {}

unsafe impl<'a> Sync for SelectorRequest<'a> {}

impl<'a> Debug for SelectorRequest<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Ok(()) }
}

impl<'a> ExecutorValueTrait<'a> for SelectorRequest<'a> {}


//////// mock
pub struct MockDefaultPureSelector<'a> {
    commands: HashMap<String, Command<'a>>,
}

unsafe impl<'a> Send for MockDefaultPureSelector<'a> {}

unsafe impl<'a> Sync for MockDefaultPureSelector<'a> {}

impl<'a> MockDefaultPureSelector<'a> {
    pub fn new() -> Self {
        let mut ret = MockDefaultPureSelector { commands: Default::default() };
        ret.on_register_cmd(mock_command());
        return ret;
    }
}

impl<'a> CommandSelector<'a> for MockDefaultPureSelector<'a> {
    fn select(&self, req: &SelectorRequest) -> Option<Command<'a>> {
        let any = req.request.as_any();
        let p;
        match any.downcast_ref::<MockRequest>() {
            None => {
                return None;
            }
            Some(v) => {
                p = v;
            }
        }
        let string_id = conv_protocol_to_string(p.protocol);
        let opt_get = self.commands.get(&string_id);
        match opt_get {
            Some(ret) => {
                // TODO
                Some(ret.clone())
            }
            None => {
                None
            }
        }
    }

    fn on_register_cmd(&mut self, cmd: Command<'a>) {
        let id = cmd.id();
        let string_id = conv_protocol_to_string(id);
        self.commands.insert(string_id, cmd);
    }
}
////////

pub struct SelectorStrategy<'a> {
    // selector: DefaultChainExecutor<'e, 'a, SelectorRequest>,
    // selector: DefaultPipelineV2<'a, SelectorRequest<'a>>,
    selectors: Vec<Box<dyn CommandSelector<'a> + 'a>>,
    // register: DefaultChainExecutor<'e, 'a, SelectorRequest>,
}

// impl<'e: 'a, 'a> From<Vec<Box<dyn CommandSelector>>> for SelectorStrategy<'e, 'a> {
//     fn from(v: Vec<Box<dyn CommandSelector>>) -> Self {
//         let selectors: &mut Vec<&dyn ReactorExecutor<DefaultChainExecutor<SelectorRequest>, SelectorRequest>> = &mut Vec::new();
//         let registers: &mut Vec<&dyn ReactorExecutor<DefaultChainExecutor<SelectorRequest>, SelectorRequest>> = &mut Vec::new();
//         for s in v.iter() {
//             let f = move |req: &SelectorRequest| {
//                 let ret = s.select(req);
//                 match ret {
//                     Some(v) => {
//                         req.tx.send(v);
//                     }
//                     None => {
//                         ;
//                     }
//                 };
//             };
//             let reactor_1 = DefaultClosureReactorExecutor::<SelectorRequest>::new(&f);
//             selectors.push(&reactor_1);
//         };
//         let mut selector_executors = DefaultChainExecutor::new(selectors);
//         SelectorStrategy { selector: DefaultPipeline::new(selector_executors) }
//     }
// }

impl<'a> SelectorStrategy<'a> {
    pub fn new(executor: Vec<Box<dyn CommandSelector<'a> + 'a>>) -> Self {
        SelectorStrategy { selectors: executor }
    }
}

impl<'a> CommandSelector<'a> for SelectorStrategy<'a> {
    fn select(&self, req: &SelectorRequest) -> Option<Command<'a>> {
        for s in &self.selectors {
            if let Some(v) = s.select(req) {
                return Some(v);
            }
        }
        None
    }

    fn on_register_cmd(&mut self, cmd: Command<'a>) {
        for s in &mut self.selectors {
            s.on_register_cmd(cmd.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::RefCell;
    use std::borrow::Borrow;
    use std::rc::Rc;
    use std::sync::Arc;
    use bytes::Bytes;
    use futures::select;
    use http::header::HeaderName;
    use http::Response;
    use hyper::Body;
    use logsdk::common::LogLevel;
    use logsdk::module;
    use logsdk::module::CellModule;
    use pipeline2::pipeline2::{ClosureExecutor, DefaultReactorExecutor, PipelineBuilder};
    use crate::command::*;
    use crate::constants::ProtocolStatus;
    use crate::context::BaseBuzzContext;
    use crate::core::{ProtocolID, RunType};
    use crate::request::{MockRequest, ServerRequestTrait, ServerResponseTrait};
    use crate::response::MockResponse;
    use crate::selector::{CommandSelector, MockDefaultPureSelector, SelectorRequest, SelectorStrategy};
    use crate::summary::{Summary, SummaryTrait};
    use crate::wrapper::ContextResponseWrapper;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    // #[test]
    // fn test_selector() {
    //     let mut c = mock_command();
    //
    //     let e1 = MockDefaultPureSelector::new();
    //     let pip = PipelineBuilder::default().add_last(DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Arc::new(move |req: &mut SelectorRequest| {
    //         let ret = e1.select(req);
    //         match ret {
    //             Some(v) => {
    //                 req.tx.send(v);
    //             }
    //             None => {
    //                 ;
    //             }
    //         };
    //     }))))).build();
    //     let mut strategy = SelectorStrategy::new(pip);
    //     let mock_request = MockRequest::new();
    //     let (txx, mut rxx) = std::sync::mpsc::channel::<Command>();
    //     let mut request = SelectorRequest { request: Arc::new(Box::new(mock_request)), tx: (txx), done: RefCell::new(false) };
    //     futures::executor::block_on(strategy.selector.execute(&mut request));
    //     let re = rxx.try_recv();
    //     match re {
    //         Ok(v) => {
    //             println!("success")
    //         }
    //         Err(e) => {
    //             println!("err")
    //         }
    //     }
    // }
}