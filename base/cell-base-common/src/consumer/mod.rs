use std::fmt::Error;

pub trait IConsumer<T, V> {
    fn consume(&self,t:T) -> Option<V>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
