#![deny(rust_2018_idioms)]

mod executor;


use std::marker::PhantomData;

pub mod pipeline2 {
    use std::borrow::Borrow;
    use std::fmt::Debug;
    use std::marker::PhantomData;
    use std::rc::Rc;
    use dyn_clone::{clone_trait_object, DynClone};

    pub struct PipelineBuilder<'a, T>
    {
        executors: Vec<DefaultReactorExecutor<'a, T>>,
    }

    impl<'a, T> Default for PipelineBuilder<'a, T>
        where
            T: 'a,
    {
        fn default() -> Self {
            PipelineBuilder { executors: Vec::new() }
        }
    }

    impl<'a, T> PipelineBuilder<'a, T>
        where
            T: 'a,
    {
        pub fn add_last(mut self, e: DefaultReactorExecutor<'a, T>) -> Self {
            self.executors.push(e);
            self
        }

        pub fn build(self) -> DefaultPipelineV2<'a, T> {
            let chain = DefaultChainExecutor::new(self.executors);
            let ret = DefaultPipelineV2 { executor: chain };
            ret
        }
    }

    pub struct DefaultPipelineV2<'a, T>
    {
        executor: DefaultChainExecutor<'a, T>,
    }

    impl<'a, T> DefaultPipelineV2<'a, T>
        where
            T: 'a,
    {
        pub fn new(executor: DefaultChainExecutor<'a, T>) -> Self {
            Self { executor }
        }
        pub fn execute(&mut self, v: &T) {
            self.executor.execute(v);
        }

        // TODO builder
        pub fn build(self) -> Self {
            self
        }
    }


    impl<'a, T> Default for DefaultPipelineV2<'a, T>
        where
            T: 'a,
    {
        fn default() -> Self {
            DefaultPipelineV2 { executor: DefaultChainExecutor::default() }
        }
    }
    // TODO: async future
    pub struct DefaultChainExecutor<'a, T>
    {
        executors: Vec<DefaultReactorExecutor<'a, T>>,
        _marker_a: PhantomData<T>,
    }

    impl<'a, T> Default for DefaultChainExecutor<'a, T>
    {
        fn default() -> Self {
            DefaultChainExecutor { executors: Vec::new(), _marker_a: Default::default() }
        }
    }

    impl<'a, T> DefaultChainExecutor<'a, T>
        where
            T: 'a,
    {
        pub fn new(executors: Vec<DefaultReactorExecutor<'a, T>>) -> Self {
            Self { executors, _marker_a: PhantomData::default() }
        }
    }


    // pub trait ExecutorClone {
    //     fn clone_box(&self) -> Box<dyn ExecutorClone>;
    // }
    //
    // impl<T, F> ExecutorClone for F
    //     where
    //         F: 'static + Executor<'a,T> + Clone
    // {
    //     fn clone_box(&self) -> Box<dyn ExecutorClone> {
    //         Box::new(self.clone())
    //     }
    // }

    // impl<'a,T> Clone for Box<dyn Executor<'a,T>> {
    //     fn clone(&self) -> Self {
    //         self.box_clone()
    //     }
    // }

    pub struct DefaultReactorExecutor<'a, T>
    {
        _marker_a: PhantomData<T>,
        f: Box<dyn Executor<'a, T> + 'a>,
    }

    impl<'a, T> Clone for DefaultReactorExecutor<'a, T>
        where
            T: 'a,
    {
        fn clone(&self) -> Self {
            DefaultReactorExecutor { _marker_a: Default::default(), f: self.f.clone() }
        }
    }

    impl<'a, T> DefaultReactorExecutor<'a, T>
        where
            T: 'a,
    {
        pub fn execute(self, t: &T, c: &mut ExecutorContext<'a, T>) {
            self.f.execute(t);
            c.next(t)
        }
        pub fn new(f: Box<dyn Executor<'a, T> + 'a>) -> Self {
            Self { _marker_a: PhantomData::default(), f }
        }
    }

    impl<'a, T> DefaultChainExecutor<'a, T>
        where
            T: 'a,
    {
        pub fn execute(&self, t: &T) {
            let ct = copy_shuffle(&self.executors);
            let mut ctx = ExecutorContext::new(ct);
            ctx.next(t);
        }
    }

    pub fn copy_shuffle<T: Clone>(vec: &Vec<T>) -> Vec<T> {
        let mut vec = vec.clone();
        vec
    }

    pub struct ExecutorContext<'a, T>
    {
        executors: Vec<DefaultReactorExecutor<'a, T>>,
    }

    impl<'a, T> ExecutorContext<'a, T>
        where
            T: 'a,
    {
        pub fn new(executors: Vec<DefaultReactorExecutor<'a, T>>) -> Self {
            Self { executors }
        }
    }

    impl<'a, T> ExecutorContext<'a, T>
        where
            T: 'a,
    {
        pub fn next(&mut self, t: &T) {
            if self.executors.len() == 0 {
                return;
            }
            let ee = self.executors.remove(0);
            ee.execute(t, self);
        }
    }

    pub struct ClosureExecutor<'a, T>
        where
            T: 'a,
    {
        _marker_t: PhantomData<T>,
        _marker_a: PhantomData<&'a ()>,
        f: Rc<dyn Fn(&T) + 'a>,
    }

    impl<'a, T> ClosureExecutor<'a, T>
        where
            T: 'a,
    {
        pub fn new(f: Rc<dyn Fn(&T) + 'a>) -> Self {
            Self { _marker_t: PhantomData::default(), _marker_a: Default::default(), f }
        }
    }

    // impl<T, F> ExecutorClone for ClosureExecutor<T, F> where F: 'static + Clone + Fn(&T) {
    //     fn clone_box(&self) -> Box<dyn ExecutorClone> {
    //         todo!()
    //     }
    // }


    pub trait Executor<'a, T>: ExecutorClone<'a, T>
        where
            T: 'a,
    {
        fn execute(&self, v: &T);
    }

    impl<'a, T> Clone for ClosureExecutor<'a, T>
        where
            T: 'a,
    {
        fn clone(&self) -> Self {
            ClosureExecutor { _marker_t: Default::default(), _marker_a: Default::default(), f: self.f.clone() }
        }
    }

    impl<'a, T> Clone for Box<dyn Executor<'a, T> + 'a>
        where
            T: 'a,
    {
        fn clone(&self) -> Box<dyn Executor<'a, T> + 'a> {
            self.clone_box()
        }
    }

    pub trait ExecutorClone<'a, T>
        where
            T: 'a,
    {
        fn clone_box(&self) -> Box<dyn Executor<'a, T> + 'a>;
    }

    impl<'a, T, F> ExecutorClone<'a, F> for T
        where
            T: Executor<'a, F> + Clone + 'a,
            F: 'a,
    {
        fn clone_box(&self) -> Box<dyn Executor<'a, F> + 'a> {
            Box::new(self.clone())
        }
    }

    //
    // impl<T, F> ExecutorClone<'a,T> for ClosureExecutor<T, F> where F: 'static + Clone + Fn(&T) {
    //     fn clone_box(&self) -> Box<dyn Executor<'a,T>> {
    //         todo!()
    //     }
    // }

    impl<'a, T> Executor<'a, T> for ClosureExecutor<'a, T>
        where
            T: 'a,
    {
        fn execute(&self, v: &T) {
            (self.f)(v)
        }
    }

    pub struct MockExecutor<T> {
        _marker_a: PhantomData<T>,
    }
    impl<T> Default for MockExecutor<T>{
        fn default() -> Self {
            MockExecutor{ _marker_a: Default::default() }
        }
    }

    impl<'a,T> Executor<'a, T> for MockExecutor<T>
        where
            T: 'a,
    {
        fn execute(&self, v: &T) {
            println!("{:?}",1)
        }
    }
    impl<'a,T> Clone for MockExecutor<T>
        where
            T: 'a,
    {
        fn clone(&self) -> Self {
            MockExecutor{ _marker_a: Default::default() }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::rc;
    use std::rc::Rc;
    use crate::pipeline2::{ClosureExecutor, DefaultChainExecutor, DefaultPipelineV2, DefaultReactorExecutor, PipelineBuilder};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }


    #[test]
    fn test_pipeline() {
        let mut pip = DefaultPipelineV2::<i64>::default();
        let f1 = |v: &i64| {
            println!("f1:{}", v)
        };
        let f2 = |v: &i64| {
            println!("f2:{}", v)
        };

        let e1 = DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(f1))));
        let e2 = DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Rc::new(f2))));
        let builder = PipelineBuilder::default();
        let pip2 = builder.add_last(e1);
        let mut final_pip = pip2.add_last(e2).build();
        final_pip.execute(&123);
        final_pip.execute(&456);
    }
}