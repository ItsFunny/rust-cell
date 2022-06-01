use std::error::Error;
use std::{fmt, io, result};
use std::fmt::{Display, Formatter};
use crate::cerror::ErrorEnums::Kind;


pub type CellResult<T> = Result<T, CellError>;


#[derive(Debug)]
pub struct CellError {
    code: usize,
    msg: String,
    err: Option<Box<dyn Error>>,
    wrapped_error: Option<Box<CellError>>,
}

impl CellError {
    pub fn get_code(&self) -> usize {
        self.code
    }
    pub fn get_msg(&self) -> &String {
        &self.msg
    }
    pub fn new(code: usize, msg: String) -> Self {
        CellError { code, msg, err: None, wrapped_error: None }
    }
    pub fn with_wrapped_error(mut self, e: Box<CellError>) -> Self {
        self.wrapped_error = Some(e);
        self
    }
    pub fn with_error(mut self, e: Box<dyn Error>) -> Self {
        self.err = Some(e);
        self
    }
}

impl From<&ErrorEnums> for CellError {
    fn from(s: &ErrorEnums) -> Self {
        CellError::new(s.get_code(), s.get_msg().to_string())
    }
}

// TODO , bad codes here
impl Display for CellError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut msg = format!("failed ,code={},msg={}", self.code, self.msg);
        match &self.err {
            Some(e) => {
                msg.push_str(",err=");
                msg.push_str(e.to_string().as_str())
            }
            None => {}
        }
        match &self.wrapped_error {
            Some(v) => {
                msg.push_str(",wrapped err=");
                msg.push_str(v.to_string().as_str());
            }
            None => {}
        }
        write!(f, "{}", msg)
    }
}

impl Error for CellError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.wrapped_error {
            Some(v) => {
                Some(v)
            }
            None => {
                None
            }
        }
    }
}

// TODO ,add from


//// enums
// #[derive(Debug)]
// #[non_exhaustive]
pub enum ErrorEnums {
    Kind(usize, &'static str),
}

impl ErrorEnums {
    pub fn get_code(&self) -> usize {
        match self {
            Kind(code, msg) => {
                *code
            }
            _ => {
                0
            }
        }
    }
    pub fn get_msg(&self) -> &'static str {
        match self {
            Kind(code, msg) => {
                *msg
            }
            _ => {
                "wrong type"
            }
        }
    }
}



#[macro_export]
macro_rules! error_enums {
    (
       $(
            $(#[$docs:meta])*
             ($name:ident, $code:expr, $msg:expr);
        )+
    )=>{
        #[derive(Debug)]
        pub struct ErrorEnumsStruct {
        }
        impl ErrorEnumsStruct
        {
            $(
                pub const $name:&'static $crate::cerror::ErrorEnums=&$crate::cerror::Kind($code,$msg);
            )+
        }
    }
}

error_enums!(
    (UNKNOWN,1,"unknown");
    (IO_ERROR,2,"IO FAILED");
    (JSON_SERIALIZE,3,"json serialize failed");
    (RESPONSE_FAILED,4,"response failed");
    (COMMAND_NOT_EXISTS,5,"command not exists");
);




//// tests
#[cfg(test)]
mod tests {
    use std::io;
    use crate::cerror::{CellError, ErrorEnums, ErrorEnumsStruct};

    #[test]
    fn test_enums() {
        let a = ErrorEnumsStruct::IO_ERROR;
        let m = a.get_msg();
        println!("code:{},msg:{}", a.get_code(), a.get_msg());
        let a = ErrorEnums::Kind(1, "asd");
        let c = ErrorEnumsStruct::JSON_SERIALIZE;
    }

    #[test]
    fn test_print() {
        let err1 = CellError::new(1, "err1".to_string());
        let mut err2 = CellError::new(2, "err2".to_string());
        err2 = err2.with_wrapped_error(Box::new(err1));
        let err3 = io::Error::from_raw_os_error(12);
        err2 = err2.with_error(Box::new(err3));
        println!("{}", err2)
    }
}