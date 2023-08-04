use crate::error::ConfigurationResult;
use crate::value::ConfigValueTrait;
use anyhow::Context;
use serde::de::DeserializeOwned;

pub struct TomlValue {
    data: Vec<u8>,
}

impl TomlValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
impl<T: DeserializeOwned + Clone> ConfigValueTrait<T> for TomlValue {
    fn as_object(&self) -> ConfigurationResult<T> {
        let str = String::from_utf8_lossy(self.data.as_slice()).to_string();
        let config = toml::from_str(str.as_str())
            .with_context(|| format!("deseralize config  to toml failed",))?;
        Ok(config)
    }
}
