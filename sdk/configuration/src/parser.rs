use crate::enums::{ModuleKey, ModuleValue, Schema};
use crate::error::{ConfigurationError, ConfigurationResult};
use crate::json::JsonValue;
use crate::toml::TomlValue;
use crate::value::ConfigValueTrait;
use anyhow::Context;
use serde::de::DeserializeOwned;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub enum ParserEnums {
    JSON(Rc<RefCell<DefaultParser>>),
}

pub trait ConfigurationParser {
    fn parse_from<T: serde::de::DeserializeOwned + Clone + Sized>(
        &mut self,
        module_name: String,
        file_path: PathBuf,
    ) -> ConfigurationResult<Box<dyn ConfigValueTrait<T>>>;
}

pub struct DefaultParser {
    values: HashMap<ModuleKey, ModuleValue>,
    schema: Schema,
}

impl DefaultParser {
    pub fn new(schema: Schema) -> Self {
        Self {
            values: Default::default(),
            schema,
        }
    }
}

impl ConfigurationParser for DefaultParser {
    fn parse_from<T: DeserializeOwned + Clone + Sized>(
        &mut self,
        module_name: String,
        file_path: PathBuf,
    ) -> ConfigurationResult<Box<dyn ConfigValueTrait<T>>> {
        let data = fs::read_to_string(file_path.as_ref()).with_context(|| {
            format!(
                "Failed to read config from {}",
                file_path.as_ref().display()
            )
        })?;
        let data = data.as_bytes().to_vec();
        let value = ModuleValue::new(data.clone());
        let key = ModuleKey::new(module_name);
        self.values.insert(key, value.clone());
        match self.schema {
            Schema::JSON => {
                let value = JsonValue::new(data);
                Ok(Box::new(value))
            }
            Schema::TOML => {
                let value = TomlValue::new(data);
                Ok(Box::new(value))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn it_works() {}
}
