use std::error::Error;
use std::fmt::{Display, Formatter, write};


#[derive(Debug)]
pub enum ErrorEnum {
    Error(i32, &'static str),
}


#[derive(Debug)]
pub struct CellError {
    error_enum: &'static ErrorEnum,
}

impl<'a> CellError {
    pub fn new(error_enum: &'static ErrorEnum) -> Self {
        CellError { error_enum }
    }
}


impl Display for CellError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.error_enum {
            ErrorEnum::Error(code, msg) => {
                write!(f, "code:{},msg:{}", code, msg)
            }
        }
    }
}

impl Error for CellError {}