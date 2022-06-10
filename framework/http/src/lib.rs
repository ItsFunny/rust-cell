pub mod server;
pub mod dispatcher;
mod channel;
mod suit;
mod context;
mod response;
mod request;
mod selector;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
