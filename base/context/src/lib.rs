pub mod context {
    pub trait Context {
        fn discard();
        fn done() -> bool;
        // fn unsafe_notify_done();
    }


    pub trait ServerRequestTrait {}

    pub trait ServerResponseTrait {}


    pub trait  SummaryTrait{

    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
