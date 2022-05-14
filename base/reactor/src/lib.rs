pub mod command;
pub mod context;
pub mod channel;

type ProtocolID = &'static str;

pub type AliasRequestType = i8;

pub type AliasResponseType = i8;

pub type RunType = i8;

pub mod reactor {
    pub trait Channel {}
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
