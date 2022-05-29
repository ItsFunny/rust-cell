use std::borrow::BorrowMut;
use std::collections::LinkedList;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

pub trait ExecutorValueTrait<'a>: Debug + 'a {}


//////////
pub trait ChainExecutor<'e: 'a, 'a, V>: Debug + 'e
    where
        V: ExecutorValueTrait<'a> + 'a,
        Self: 'e,
{
    fn execute(&'e mut self, v: &'a V);
}

pub trait ReactorExecutor<'e: 'a, 'a, E, V>: Debug + 'e
    where
        V: ExecutorValueTrait<'a> + 'a,
        E: ChainExecutor<'e, 'a, V> + 'e,
        Self: 'e,
{
    // TODO: ASYNC
    fn execute(&'e self, v: &'a V, chain: &'e mut E) {
        self.on_execute(v);
        chain.execute(v)
    }
    fn on_execute(&'e self, v: &'a V) {
        // do nothing
    }
}


/////////////
pub struct DefaultChainExecutor<'e, 'a, V: 'a>
    where
        V: ExecutorValueTrait<'a>,
        Self: 'e,
{
    executors: &'e mut Vec<&'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>>,

    index: usize,
}

impl<'e: 'a, 'a, V: 'a> ChainExecutor<'e, 'a, V> for DefaultChainExecutor<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a> + 'a,
        Self: 'e,
{
    fn execute(&'e mut self, v: &'a V) {
        if self.index < self.executors.len() {
            let mut executor = self.executors.get(self.index).unwrap();
            self.index += 1;
            executor.execute(v, self);
        }
    }
}

impl<'e, 'a, V: 'a> DefaultChainExecutor<'e, 'a, V> where
    V: ExecutorValueTrait<'a>,
    Self: 'e, {
    pub fn new(exs: &'e mut Vec<&'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>>,
    ) -> Self {
        DefaultChainExecutor { executors: exs, index: 0 }
    }

    pub fn add_last(&'e mut self, exe: &'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>) {
        self.executors.push(exe)
    }
}


impl<'e, 'a, V> Debug for DefaultChainExecutor<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a> + 'a,
        Self: 'e,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'e, 'a, V: 'a> DefaultChainExecutor<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a>,
{}

/////////////
pub struct DefaultClosureReactorExecutor<'e, 'a, V> {
    f: &'e dyn Fn(&'a V),
}

impl<'e, 'a, V> DefaultClosureReactorExecutor<'e, 'a, V> {
    pub fn new(f: &'e dyn Fn(&'a V)) -> Self {
        DefaultClosureReactorExecutor { f }
    }
}

impl<'e, 'a, V> Debug for DefaultClosureReactorExecutor<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a> + 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'e, 'a, V> ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> for DefaultClosureReactorExecutor<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a> + 'a,
{
    fn on_execute(&'e self, v: &'a V) {
        (self.f)(v);
    }
}

/////////////
pub struct DefaultReactorExecutor
{}

impl DefaultReactorExecutor {
    pub fn new() -> Self {
        DefaultReactorExecutor {}
    }
}

impl Debug for DefaultReactorExecutor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'e, 'a, V> ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> for DefaultReactorExecutor
    where
        V: ExecutorValueTrait<'a>
{
    fn on_execute(&'e self, v: &'a V) {
        println!("{:?}", v);
    }
}

/////////////

pub struct DefaultChainExecutorBuilder<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a>,
{
    executors: Vec<&'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>>,
}

impl<'e, 'a, V> DefaultChainExecutorBuilder<'e, 'a, V>
    where
        V: ExecutorValueTrait<'a>,
{
    pub fn new() -> DefaultChainExecutorBuilder<'e, 'a, V> {
        DefaultChainExecutorBuilder { executors: Vec::new() }
    }
    pub fn executor(&mut self, e: &'e dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V>) -> &mut DefaultChainExecutorBuilder<'e, 'a, V> {
        self.executors.push(e);
        self
    }
    pub fn build(&'e mut self) -> DefaultChainExecutor<'e, 'a, V> {
        DefaultChainExecutor::new(&mut self.executors)
    }
}


/////////


#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::LinkedList;
    use std::fmt::{Debug, Formatter};
    use std::marker::PhantomData;
    use std::rc::Rc;
    use crate::executor::{ChainExecutor, DefaultChainExecutor, DefaultReactorExecutor, ExecutorValueTrait, ReactorExecutor};

    pub struct AA {
        name: String,
    }

    impl Debug for AA {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", "asd")
        }
    }

    impl<'a> ExecutorValueTrait<'a> for AA {}

    pub struct BB<'e, 'a, V>
        where
            V: ExecutorValueTrait<'a>,
    {
        _marker: PhantomData<&'e ()>,
        l: &'e LinkedList<&'e Box<dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> + 'e>>,
        l2: &'e Vec<&'e Box<dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> + 'e>>,
    }

    impl<'e, 'a, V> BB<'e, 'a, V> where
        V: ExecutorValueTrait<'a> + 'a, {
        pub fn new(l: &'e LinkedList<&'e Box<dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> + 'e>>,
                   l2: &'e Vec<&'e Box<dyn ReactorExecutor<'e, 'a, DefaultChainExecutor<'e, 'a, V>, V> + 'e>>) -> Self {
            BB { _marker: PhantomData::default(), l, l2 }
        }
    }

    impl<'e, 'a, V> BB<'e, 'a, V>
        where
            V: ExecutorValueTrait<'a>,
    {
        pub fn call(&'e mut self) {
            // let v = self.copy();
        }
    }

    #[test]
    fn test_chain_executor() {
        let mut ess2: &mut Vec<&dyn ReactorExecutor<DefaultChainExecutor<AA>, AA>> = &mut Vec::new();
        let e1: &dyn ReactorExecutor<DefaultChainExecutor<AA>, AA> = &DefaultReactorExecutor::new();
        let e2: &dyn ReactorExecutor<DefaultChainExecutor<AA>, AA> = &DefaultReactorExecutor::new();
        ess2.push(e1);
        ess2.push(e2);

        let mut chain_executor = DefaultChainExecutor::new(ess2);
        chain_executor.execute(&AA { name: String::from("asd") });
    }

    #[test]
    fn test_bb() {
        let mut _v1: LinkedList<&Box<dyn ReactorExecutor<DefaultChainExecutor<AA>, AA>>> = LinkedList::new();
        let mut _v2: Vec<&Box<dyn ReactorExecutor<DefaultChainExecutor<AA>, AA>>> = Vec::new();
        let mut b = BB::new(&_v1, &_v2);
    }

    #[test]
    fn test_default_executor() {}
}