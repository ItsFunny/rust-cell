extern crate core;

pub mod channel;
pub mod context;
pub mod dispatcher;
pub mod extension;
pub mod module;
pub mod request;
pub mod response;
mod selector;
pub mod server;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
