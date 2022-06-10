extern crate core;

use std::marker::PhantomData;

pub mod pipeline2 {
    use std::marker::PhantomData;
    use std::rc::Rc;

    pub struct PipelineBuilder<T>
    {
        executors: Vec<DefaultReactorExecutor<T>>,
    }

    impl<T> Default for PipelineBuilder<T>
    {
        fn default() -> Self {
            PipelineBuilder { executors: Vec::new() }
        }
    }

    impl<T> PipelineBuilder<T>
    {
        pub fn add_last(mut self, e: DefaultReactorExecutor<T>) -> Self {
            self.executors.push(e);
            self
        }

        pub fn build(self) -> DefaultPipeline<T> {
            let chain = DefaultChainExecutor::new(self.executors);
            let ret = DefaultPipeline { executor: chain };
            ret
        }
    }


    pub struct DefaultPipeline<T>
    {
        executor: DefaultChainExecutor<T>,
    }

    impl<T> DefaultPipeline<T>
    {
        pub fn new(executor: DefaultChainExecutor<T>) -> Self {
            Self { executor }
        }
        pub fn execute(&mut self, v: &T) {
            self.executor.execute(v);
            // let e = Rc::clone(&self.executor);
            // e.execute(v);
        }

        // TODO builder
        pub fn build(self) -> Self {
            self
        }
    }


    impl<T> Default for DefaultPipeline<T>
    {
        fn default() -> Self {
            DefaultPipeline { executor: DefaultChainExecutor::default() }
        }
    }

    pub struct DefaultChainExecutor<T>
    {
        executors: Vec<DefaultReactorExecutor<T>>,
        _marker_a: PhantomData<T>,
    }

    impl<T> Default for DefaultChainExecutor<T>
    {
        fn default() -> Self {
            DefaultChainExecutor { executors: Vec::new(), _marker_a: Default::default() }
        }
    }

    impl<T> DefaultChainExecutor<T>
    {
        pub fn new(executors: Vec<DefaultReactorExecutor<T>>) -> Self {
            Self { executors, _marker_a: PhantomData::default() }
        }
    }

    pub trait Executor<T>: Clone {
        fn execute(v: &T);
    }

    pub struct DefaultReactorExecutor<T>
    {
        _marker_a: PhantomData<T>,
        f: fn(&T),
    }


    impl<T> Clone for DefaultReactorExecutor<T>
    {
        fn clone(&self) -> Self {
            DefaultReactorExecutor { _marker_a: Default::default(), f: self.f.clone() }
        }
    }

    impl<T> DefaultReactorExecutor<T>
    {
        pub fn execute(self, t: &T, c: &mut ExecutorContext<T>) {
            (self.f)(t);
            c.next(t)
        }
        pub fn new(f: fn(&T)) -> Self {
            Self { _marker_a: PhantomData::default(), f }
        }
    }

    impl<T> DefaultChainExecutor<T>
    {
        pub fn execute(&self, t: &T) {
            // let mut ctx = ExecutorContext::new(self.executors);
            // ctx.next(t);

            let ct = copy_shuffle(&self.executors);
            let mut ctx = ExecutorContext::new(ct);
            ctx.next(t);
        }
    }

    pub fn copy_shuffle<T: Clone>(vec: &Vec<T>) -> Vec<T> {
        let mut vec = vec.clone();
        vec
    }

    pub struct ExecutorContext<T>
    {
        executors: Vec<DefaultReactorExecutor<T>>,
    }

    impl<T> ExecutorContext<T>
    {
        pub fn new(executors: Vec<DefaultReactorExecutor<T>>) -> Self {
            Self { executors }
        }
    }

    impl<T> ExecutorContext<T>
    {
        pub fn next(&mut self, t: &T) {
            if self.executors.len() == 0 {
                return;
            }
            let ee = self.executors.remove(0);
            ee.execute(t, self);
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::pipeline2::{DefaultChainExecutor, DefaultPipeline, DefaultReactorExecutor, PipelineBuilder};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }


    #[test]
    fn test_pipeline() {
        let mut pip = DefaultPipeline::<i64>::default();
        let f1 = |v: &i64| {
            println!("f1:{}", v)
        };
        let f2 = |v: &i64| {
            println!("f2:{}", v)
        };
        let e1 = DefaultReactorExecutor::new(f1);
        let e2 = DefaultReactorExecutor::new(f2);
        let builder = PipelineBuilder::default();
        let pip2 = builder.add_last(e1);
        let mut final_pip = pip2.add_last(e2).build();
        final_pip.execute(&123);
        final_pip.execute(&456);
    }
}