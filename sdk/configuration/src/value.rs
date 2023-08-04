use crate::error::ConfigurationResult;

pub trait ConfigValueTrait<T: serde::de::DeserializeOwned + Clone + Sized> {
    fn as_object(&self) -> ConfigurationResult<T>;
}
