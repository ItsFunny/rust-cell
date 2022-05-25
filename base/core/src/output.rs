use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Add;
use bytes::Bytes;
use chrono::format::Item;
use futures::task::ArcWake;
use json::JsonValue;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::cerror::{CellError, CellResult, ErrorEnums, ErrorEnumsStruct};


pub trait Serializable<'a>: Serialize + Deserialize<'a> + Debug + 'a {}

pub trait OutputArchive<T>: Sync + Send
{
    fn as_bytes(&self, s: Box<T>) -> CellResult<Bytes>;
}

pub struct JSONOutputArchive<'a>
{
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a> Send for JSONOutputArchive<'a>
{}

unsafe impl<'a> Sync for JSONOutputArchive<'a>
{}

impl<'a> Default for JSONOutputArchive<'a> {
    fn default() -> Self {
        JSONOutputArchive {
            _marker: Default::default(),
        }
    }
}

impl<'a, T> OutputArchive<T> for JSONOutputArchive<'a>
    where
        T: Serializable<'a>
{
    fn as_bytes(&self, syn: Box<T>) -> CellResult<Bytes> {
        // TODO NONE
        serde_json::to_string(&syn).and_then(|v| {
            Ok(Bytes::from(v))
        }).map_err(|e| {
            CellError::from(ErrorEnumsStruct::JSON_SERIALIZE).with_error(Box::new(e))
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fmt::{Debug, Formatter};
    use std::ops::Add;
    use bytes::{Buf, Bytes};
    use json::JsonValue;
    use rocket::debug;
    use crate::output::{JSONOutputArchive, OutputArchive, Serializable};

    use serde::{Deserialize, Serialize, Serializer};
    use serde_json::Result;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct A {
        name: String,
        b: B,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct B {
        name_b: String,
    }

    impl B {
        pub fn new(name_b: String) -> Self {
            B { name_b }
        }
    }

    impl A {
        pub fn new(name: String, b: B) -> Self {
            A { name, b }
        }
    }

    impl Add for A {
        type Output = ();

        fn add(self, rhs: Self) -> Self::Output {
            todo!()
        }
    }

    impl<'a> Serializable<'a> for A {}

    #[test]
    fn test_as_bytes() {
        let a = A::new(String::from("asd"), B::new(String::from("nnn")));
        let ar = JSONOutputArchive::default();
        let mut bb = ar.as_bytes(Box::new(a));
        println!("{:?}", bb);
        let ar2 = JSONOutputArchive::default();
        let bs = bb.unwrap();
        let a = String::from_utf8_lossy(bs.chunk());
        println!("{:?}", a);
        let ret = serde_json::to_string(bs.chunk());
        println!("{:?}", ret);
    }


    pub struct C {
        name: String,
    }

    #[derive(Debug)]
    pub struct CCOutPut {
        ccc: Box<dyn Debug>,
    }

    unsafe impl Sync for CCOutPut {}

    unsafe impl Send for CCOutPut {}

    pub trait NewTrait: Debug + Serialize {}
}