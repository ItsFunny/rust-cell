use std::fmt::{Display, Formatter, write};

pub trait ModuleTrait{
    fn index(&self) -> u16;
    fn name(&self) -> String;
}


