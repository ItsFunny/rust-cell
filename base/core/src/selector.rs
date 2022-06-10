use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std_core::cell::RefCell;
use rocket::figment::map;
use pipeline::executor::{DefaultChainExecutor, DefaultClosureReactorExecutor, ExecutorValueTrait, ReactorExecutor};
use pipeline::pipeline::DefaultPipeline;
use crate::cerror::CellResult;
use crate::command::CommandTrait;
use crate::core::conv_protocol_to_string;
use crate::request::{MockRequest, ServerRequestTrait};

pub trait CommandSelector {
    fn select(&self, req: &SelectorRequest) -> Option<&'static dyn CommandTrait>;
    fn on_register_cmd(&mut self, cmd: &'static dyn CommandTrait);
}

pub struct SelectorRequest<'a> {
    pub request: Rc<Box<dyn ServerRequestTrait+'a>>,
    pub tx: Sender<&'static dyn CommandTrait>,
    // TODO wrap tx
    pub done: RefCell<bool>,
}

impl<'a> SelectorRequest<'a> {
    pub fn new(request: Rc<Box<dyn ServerRequestTrait+'a>>, tx: Sender<&'static dyn CommandTrait>) -> Self {
        SelectorRequest { request, tx, done: RefCell::new(false) }
    }
}

impl<'a> Debug for SelectorRequest<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { Ok(()) }
}

impl<'a> ExecutorValueTrait<'a> for SelectorRequest<'a> {}


//////// mock
pub struct MockDefaultPureSelector {
    commands: HashMap<String, &'static dyn CommandTrait>,
}

impl MockDefaultPureSelector {
    pub fn new() -> Self {
        MockDefaultPureSelector { commands: Default::default() }
    }
}

impl CommandSelector for MockDefaultPureSelector {
    fn select(&self, req: &SelectorRequest) -> Option<&'static dyn CommandTrait> {
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
                let a = &(**ret);
                Some(a)
            }
            None => {
                None
            }
        }
    }

    fn on_register_cmd(&mut self, cmd: &'static dyn CommandTrait) {
        let id = cmd.id();
        let string_id = conv_protocol_to_string(id);
        self.commands.insert(string_id, cmd);
    }
}
////////

pub struct SelectorStrategy<'e: 'a, 'a> {
    // selector: DefaultChainExecutor<'e, 'a, SelectorRequest>,
    selector: DefaultPipeline<'e, 'a, SelectorRequest<'a>>,

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

impl<'e: 'a, 'a> SelectorStrategy<'e, 'a> {
    pub fn new(executor: DefaultChainExecutor<'e, 'a, SelectorRequest<'a>>) -> Self {
        SelectorStrategy { selector: DefaultPipeline::new(executor) }
    }
}

// impl CommandSelector for SelectorStrategy {
//     fn select(&self, req: &SelectorRequest) -> Option<&'static dyn CommandTrait> {}
//
//     fn on_register_cmd(&mut self, cmd: &'static dyn CommandTrait) {
//         todo!()
//     }
// }

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
    use pipeline::executor::{ChainExecutor, DefaultChainExecutor, DefaultClosureReactorExecutor, ReactorExecutor};
    use crate::command::{Command, CommandContext, CommandTrait};
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

    #[test]
    fn test_selector() {
        let mut c = Command::default();
        c = c.with_protocol_id("/v1/protocol");
        c = c.with_run_type(1 as RunType);
        c = c.with_executor(&move |ctx, v| {
            println!("ctx")
        });
        c = c.do_seal();

        let e1 = MockDefaultPureSelector::new();
        let ess2: &mut Vec<&dyn ReactorExecutor<DefaultChainExecutor<SelectorRequest>, SelectorRequest>> = &mut Vec::new();
        let f = move |req: &SelectorRequest| {
            let ret = e1.select(req);
            match ret {
                Some(v) => {
                    req.tx.send(v);
                }
                None => {
                    ;
                }
            };
        };
        let reactor_1 = DefaultClosureReactorExecutor::<SelectorRequest>::new(&f);
        ess2.push(&reactor_1);

        let mut chain_executor = DefaultChainExecutor::new(ess2);
        let mut strategy = SelectorStrategy::new(chain_executor);
        let mock_request = MockRequest::new();
        let (txx, mut rxx) = std::sync::mpsc::channel::<&'static dyn CommandTrait>();
        let mut request = SelectorRequest { request: Rc::new(Box::new(mock_request)), tx: (txx), done: RefCell::new(false) };
        strategy.selector.execute(&mut request);
        let re = rxx.try_recv();
    }
}