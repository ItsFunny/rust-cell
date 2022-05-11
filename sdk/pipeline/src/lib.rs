extern crate core;

pub mod executor;

pub mod pipeline {
    use crate::executor::{ChainExecutor, DefaultChainExecutor, DefaultReactorExecutor, ExecutorValueTrait, ReactorExecutor};

    pub trait Pipeline<'e: 'a, 'a, T, E, V>
        where
            V: ExecutorValueTrait<'a>,
            T: ?ReactorExecutor<'e, 'a, E, V>,
            E: ChainExecutor<'e, 'a, V>,
    {
        fn add_last(&'e mut self, exe: T);
    }

    pub struct DefaultPipeline<'e: 'a, 'a, V>
        where
            V: ExecutorValueTrait<'a>,
    {
        executor: DefaultChainExecutor<'e, 'a, V>,
        seald: bool,
    }

    impl<'e: 'a, 'a, V> Pipeline<'e, 'a, &'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>, DefaultChainExecutor<'e, 'a, V>, V> for DefaultPipeline<'e, 'a, V>
        where
            V: ExecutorValueTrait<'a>,
    {
        fn add_last(&'e mut self, exe: &'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>) {
            self.executor.add_last(exe)
        }
    }

    impl<'e: 'a, 'a, V> DefaultPipeline<'e, 'a, V>
        where
            V: ExecutorValueTrait<'a>,
            Self: 'e,
    {
        pub fn new(exe: DefaultChainExecutor<'e, 'a, V>) -> Self {
            let ret = DefaultPipeline { executor: exe, seald: true };
            ret
        }

        pub fn execute(&'e mut self, v: &'a V) {
            self.executor.execute(v)
        }
    }
}


#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Formatter};
    use crate::executor::{DefaultChainExecutor, DefaultChainExecutorBuilder, DefaultReactorExecutor, ExecutorValueTrait, ReactorExecutor};
    use crate::pipeline::DefaultPipeline;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    pub struct PipValue {
        pub name: String,
    }

    impl Debug for PipValue {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.name)
        }
    }

    impl<'a> ExecutorValueTrait<'a> for PipValue {}

    #[test]
    fn test_pipeline() {
        let exe22: &dyn ReactorExecutor<DefaultChainExecutor<PipValue>, PipValue> = &DefaultReactorExecutor::new();
        let mut builder = DefaultChainExecutorBuilder::new();
        let executor = builder.executor(exe22).build();
        let mut pip = DefaultPipeline::new(executor);
        pip.execute(&PipValue { name: String::from("charlie") })
    }
}
