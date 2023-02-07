extern crate core;

pub mod server;
pub mod dispatcher;
pub mod module;
pub mod extension;
pub mod channel;
pub mod context;
pub mod response;
pub mod request;
mod selector;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
