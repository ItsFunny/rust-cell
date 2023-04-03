use crate::error::ConfigurationResult;
use serde::Deserialize;

pub trait ConfigValueTrait<T: serde::de::DeserializeOwned + Clone + Sized> {
    fn as_object(&self) -> ConfigurationResult<T>;
}
