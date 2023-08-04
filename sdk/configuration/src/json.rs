use crate::error::ConfigurationResult;
use crate::value::ConfigValueTrait;
use serde::de::DeserializeOwned;

pub struct JsonValue {
    data: Vec<u8>,
}

impl JsonValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl<T: DeserializeOwned + Clone> ConfigValueTrait<T> for JsonValue {
    fn as_object(&self) -> ConfigurationResult<T> {
        let ret = serde_json::from_slice::<T>(self.data.as_slice())?;
        Ok(ret)
    }
}
