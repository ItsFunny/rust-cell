#![deny(rust_2018_idioms)]

mod executor;

use std::marker::PhantomData;

pub mod pipeline2 {
    use async_recursion::async_recursion;
    use async_trait::async_trait;
    use dyn_clone::{clone_trait_object, DynClone};
    use std::borrow::Borrow;
    use std::fmt::Debug;
    use std::marker::PhantomData;
    use std::rc::Rc;
    use std::sync::Arc;

    pub struct PipelineBuilder<'a, T> {
        executors: Vec<DefaultReactorExecutor<'a, T>>,
    }

    impl<'a, T> Default for PipelineBuilder<'a, T>
    where
        T: 'a + Sync + Send,
    {
        fn default() -> Self {
            PipelineBuilder {
                executors: Vec::new(),
            }
        }
    }

    impl<'a, T> PipelineBuilder<'a, T>
    where
        T: 'a + Sync + Send,
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

    pub struct DefaultPipelineV2<'a, T> {
        executor: DefaultChainExecutor<'a, T>,
    }

    pub fn is_send<T: Send>(t: T) {
        println!("true")
    }

    // unsafe impl<'a, T> Send for DefaultPipelineV2<'a, T> {}
    //
    // unsafe impl<'a, T> Sync for DefaultPipelineV2<'a, T> {}

    impl<'a, T> DefaultPipelineV2<'a, T>
    where
        T: 'a + Sync + Send,
    {
        pub fn new(executor: DefaultChainExecutor<'a, T>) -> Self {
            Self { executor }
        }

        pub async fn execute(&self, v: &mut T) {
            self.executor.execute(v).await
        }

        // TODO builder
        pub fn build(self) -> Self {
            self
        }
        pub fn add_last(&mut self, e: DefaultReactorExecutor<'a, T>) {
            self.executor.executors.push(e);
        }
    }

    impl<'a, T> Default for DefaultPipelineV2<'a, T>
    where
        T: 'a + Sync + Send,
    {
        fn default() -> Self {
            DefaultPipelineV2 {
                executor: DefaultChainExecutor::default(),
            }
        }
    }

    // TODO: async future
    pub struct DefaultChainExecutor<'a, T> {
        executors: Vec<DefaultReactorExecutor<'a, T>>,
    }

    // unsafe impl<'a, T> Send for DefaultChainExecutor<'a, T> {}

    // unsafe impl<'a, T> Sync for DefaultChainExecutor<'a, T> {}

    impl<'a, T> Default for DefaultChainExecutor<'a, T> {
        fn default() -> Self {
            DefaultChainExecutor {
                executors: Vec::new(),
            }
        }
    }

    impl<'a, T> DefaultChainExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        pub fn new(executors: Vec<DefaultReactorExecutor<'a, T>>) -> Self {
            Self { executors }
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

    pub struct DefaultReactorExecutor<'a, T> {
        f: Box<dyn Executor<'a, T> + 'a>,
    }

    // unsafe impl<'a, T> Send for DefaultReactorExecutor<'a, T> {}

    // unsafe impl<'a, T> Sync for DefaultReactorExecutor<'a, T> {}

    impl<'a, T> Clone for DefaultReactorExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        fn clone(&self) -> Self {
            DefaultReactorExecutor { f: self.f.clone() }
        }
    }

    impl<'a, T> DefaultReactorExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        #[async_recursion]
        pub async fn execute(self, t: &mut T, c: &mut ExecutorContext<'a, T>) {
            self.f.execute(t);
            c.next(t).await
        }
        pub fn new(f: Box<dyn Executor<'a, T> + 'a>) -> Self {
            Self { f }
        }
    }

    impl<'a, T> DefaultChainExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        pub async fn execute(&self, t: &mut T) {
            let ct = copy_shuffle(&self.executors);
            let mut ctx = ExecutorContext::new(ct);
            ctx.next(t).await;
        }
    }

    pub fn copy_shuffle<T: Clone>(vec: &Vec<T>) -> Vec<T> {
        let mut vec = vec.clone();
        vec
    }

    pub struct ExecutorContext<'a, T> {
        executors: Vec<DefaultReactorExecutor<'a, T>>,
    }

    // unsafe impl<'a, T> Send for ExecutorContext<'a,T> {}

    // unsafe impl<'a, T> Sync for ExecutorContext<'a,T> {}

    impl<'a, T> ExecutorContext<'a, T>
    where
        T: 'a + Sync + Send,
    {
        pub fn new(executors: Vec<DefaultReactorExecutor<'a, T>>) -> Self {
            Self { executors }
        }
    }

    impl<'a, T> ExecutorContext<'a, T>
    where
        T: 'a + Sync + Send,
    {
        pub async fn next(&mut self, t: &mut T) {
            if self.executors.len() == 0 {
                return;
            }
            let ee = self.executors.remove(0);
            ee.execute(t, self).await;
        }
    }

    pub struct ClosureExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        f: Arc<dyn Fn(&mut T) + 'a>,
    }

    unsafe impl<'a, T> Sync for ClosureExecutor<'a, T> where T: 'a + Sync + Send {}

    unsafe impl<'a, T> Send for ClosureExecutor<'a, T> where T: 'a + Sync + Send {}

    impl<'a, T> ClosureExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        pub fn new(f: Arc<dyn Fn(&mut T) + 'a>) -> Self {
            Self { f }
        }
    }

    // impl<T, F> ExecutorClone for ClosureExecutor<T, F> where F: 'static + Clone + Fn(&T) {
    //     fn clone_box(&self) -> Box<dyn ExecutorClone> {
    //         todo!()
    //     }
    // }

    pub trait Executor<'a, T>: ExecutorClone<'a, T> + Send + Sync
    where
        T: 'a + Sync + Send,
    {
        fn execute(&self, v: &mut T);
    }

    impl<'a, T> Clone for ClosureExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        fn clone(&self) -> Self {
            ClosureExecutor { f: self.f.clone() }
        }
    }

    impl<'a, T> Clone for Box<dyn Executor<'a, T> + 'a>
    where
        T: 'a + Sync + Send,
    {
        fn clone(&self) -> Box<dyn Executor<'a, T> + 'a> {
            self.clone_box()
        }
    }

    pub trait ExecutorClone<'a, T>
    where
        T: 'a + Sync + Send,
    {
        fn clone_box(&self) -> Box<dyn Executor<'a, T> + 'a>;
    }

    impl<'a, T, F> ExecutorClone<'a, F> for T
    where
        T: Executor<'a, F> + Clone + 'a,
        F: 'a + Sync + Send,
    {
        fn clone_box(&self) -> Box<dyn Executor<'a, F> + 'a> {
            Box::new(self.clone())
        }
    }

    impl<'a, T> Executor<'a, T> for ClosureExecutor<'a, T>
    where
        T: 'a + Sync + Send,
    {
        fn execute(&self, v: &mut T) {
            (self.f)(v)
        }
    }

    pub struct MockExecutor<T> {
        _marker_a: PhantomData<T>,
    }

    impl<T> Default for MockExecutor<T> {
        fn default() -> Self {
            MockExecutor {
                _marker_a: Default::default(),
            }
        }
    }

    impl<'a, T> Executor<'a, T> for MockExecutor<T>
    where
        T: 'a + Sync + Send,
    {
        fn execute(&self, v: &mut T) {
            println!("{:?}", 1)
        }
    }

    impl<'a, T> Clone for MockExecutor<T>
    where
        T: 'a + Sync + Send,
    {
        fn clone(&self) -> Self {
            MockExecutor {
                _marker_a: Default::default(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::pipeline2::{
        is_send, ClosureExecutor, DefaultChainExecutor, DefaultPipelineV2, DefaultReactorExecutor,
        ExecutorContext, PipelineBuilder,
    };
    use std::rc::Rc;
    use std::sync::Arc;
    use std::{rc, thread, time};

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_is_send() {
        let a = DefaultPipelineV2::<i32>::default();
        is_send(a);
    }

    #[test]
    fn test_pipeline() {
        let mut pip = DefaultPipelineV2::<i64>::default();
        let f1 = |v: &mut i64| println!("f1:{}", v);
        let f2 = |v: &mut i64| println!("f2:{}", v);
        is_send(ExecutorContext::<i32>::new(Vec::new()));
        let e1 = DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Arc::new(f1))));
        let e2 = DefaultReactorExecutor::new(Box::new(ClosureExecutor::new(Arc::new(f2))));
        let builder = PipelineBuilder::default();
        let pip2 = builder.add_last(e1);
        let mut final_pip = pip2.add_last(e2).build();
        futures::executor::block_on(final_pip.execute(&mut 123));
        futures::executor::block_on(final_pip.execute(&mut 456));
    }
}
