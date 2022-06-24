extern crate core;

pub mod server;
pub mod dispatcher;
mod channel;
mod context;
mod response;
mod request;
mod selector;
pub mod extension;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
