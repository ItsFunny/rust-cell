use crate::command::Command;
use crate::core::conv_protocol_to_string;
use std::collections::HashMap;

pub trait Collector<T>
where
    T: Clone,
{
    fn collect(&mut self, t: T);
}

pub trait CommandCollector: Collector<Command<'static>> {}

pub struct DefaultCommandCollector {
    commands: HashMap<String, Command<'static>>,
}

impl CommandCollector for DefaultCommandCollector {}

impl Collector<Command<'static>> for DefaultCommandCollector {
    fn collect(&mut self, t: Command<'static>) {
        let mut cmd = t.clone();
        let id = conv_protocol_to_string(cmd.protocol_id);
        self.commands.insert(id, cmd);
    }
}
