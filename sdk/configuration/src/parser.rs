use crate::enums::Schema;
use crate::error::{ConfigurationError, ConfigurationResult};
use crate::value::ConfigValueTrait;
use jsonnet::JsonnetVm;
use serde::de::DeserializeOwned;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

pub enum ParserEnums {
    JSON(Rc<RefCell<JsonParser>>),
}

pub trait ConfigurationParser {
    fn parse_from<T: serde::de::DeserializeOwned + Clone + Sized>(
        &mut self,
        module_name: String,
        file_path: PathBuf,
    ) -> ConfigurationResult<Box<dyn ConfigValueTrait<T>>>;
}

#[derive(Default)]
pub struct JsonParser {
    json_values: HashMap<JsonKey, JsonValue>,
}

#[derive(Eq, PartialEq, Hash)]
struct JsonKey {
    module_name: String,
}

impl JsonKey {
    pub fn new(module_name: String) -> Self {
        Self { module_name }
    }
}

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

impl ConfigurationParser for JsonParser {
    fn parse_from<T: serde::de::DeserializeOwned + Clone + Sized>(
        &mut self,
        module_name: String,
        file_path: PathBuf,
    ) -> ConfigurationResult<Box<dyn ConfigValueTrait<T>>> {
        let mut vm = JsonnetVm::new();
        let data = vm.evaluate_file(file_path)?.to_string().as_bytes().to_vec();
        let key = JsonKey::new(module_name.clone());
        let info = self.json_values.get(&key);
        if info.is_some() {}
        let value = JsonValue::new(data);

        Ok(Box::new(value))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn it_works() {}
}
