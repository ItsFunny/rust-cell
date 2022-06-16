use std::collections::HashMap;
use cell_core::command::Command;
use cell_core::core::conv_protocol_to_string;
use cell_core::selector::{CommandSelector, SelectorRequest};
use crate::request::HttpRequest;

pub struct HttpSelector {
    commands: HashMap<String, Command>,
}
impl Default for HttpSelector{
    fn default() -> Self {
        HttpSelector{
            commands: Default::default()
        }
    }
}


impl CommandSelector for HttpSelector {
    fn select(&self, req: &SelectorRequest)  -> Option<Command>{
        let any = req.request.as_any();
        let p;
        match any.downcast_ref::<HttpRequest>() {
            None => {
                return None;
            }
            Some(v) => {
                p = v;
            }
        }
        let uri=p.request.uri();
        let string_id = String::from(uri.path().clone());
        let opt_get = self.commands.get(&string_id);
        match opt_get {
            Some(ret) => {
                Some(ret.clone())
            }
            None => {
                None
            }
        }
    }

    fn on_register_cmd(&mut self, cmd: Command){
        let id = cmd.id();
        let string_id = conv_protocol_to_string(id);
        self.commands.insert(string_id, cmd);
    }
}