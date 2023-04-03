pub mod cellerrors;
pub mod consumer;
pub mod context;
pub mod events;
pub mod hook;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
