use std::fmt::Debug;
use std::marker::PhantomData;
use bytes::Bytes;
use futures::task::ArcWake;
use json::JsonValue;
use serde::{Deserialize, Serialize};


pub trait OutputArchive: Sync + Send {
    fn as_bytes(&self) -> Bytes;
}

pub struct JSONOutputArchive<'a, T>
    where
        T: Serialize + Deserialize<'a> + Debug + 'a
{
    data: Box<T>,
    _marker: PhantomData<&'a ()>,
}

unsafe impl<'a, T> Send for JSONOutputArchive<'a, T>
    where
        T: Serialize + Deserialize<'a> + Debug + 'a
{}

unsafe impl<'a, T> Sync for JSONOutputArchive<'a, T>
    where
        T: Serialize + Deserialize<'a> + Debug + 'a
{}


impl<'a, T> JSONOutputArchive<'a, T>
    where
        T: Serialize + Deserialize<'a> + Debug {
    pub fn new(data: Box<T>) -> Self {
        JSONOutputArchive { data, _marker: Default::default() }
    }
}

impl<'a, T> OutputArchive for JSONOutputArchive<'a, T>
    where
        T: Serialize + Deserialize<'a> + Debug
{
    fn as_bytes(&self) -> Bytes {
        Bytes::from(serde_json::to_string(&self.data).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use json::JsonValue;
    use crate::output::{JSONOutputArchive, OutputArchive};

    use serde::{Deserialize, Serialize};
    use serde_json::Result;

    #[derive(Serialize, Deserialize, Debug)]
    struct Address {
        street: String,
        city: String,
    }

    fn print_an_address() -> Result<()> {
        // Some data structure.
        let address = Address {
            street: "10 Downing Street".to_owned(),
            city: "London".to_owned(),
        };

        // Serialize it to a JSON string.
        let j = serde_json::to_string(&address)?;

        // Print, write to a file, or send to an HTTP server.
        println!("{}", j);

        Ok(())
    }

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

    #[test]
    fn test_asd() {
        let a = print_an_address();
        println!("{:?}", a)
    }

    #[test]
    fn test_as_bytes() {
        let a = A::new(String::from("asd"), B::new(String::from("nnn")));
        let ar = JSONOutputArchive::new(Box::new(a));
        println!("{:?}", ar.as_bytes())
    }
}