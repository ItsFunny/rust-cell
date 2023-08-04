use crate::enums::Schema;
use crate::error::{ConfigurationError, ConfigurationResult};
use crate::manager::Manager;
use crate::parser::{ConfigurationParser, DefaultParser, ParserEnums};
use crate::value::ConfigValueTrait;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct Configuration {
    parser: HashMap<Schema, Rc<RefCell<DefaultParser>>>,
    config_module: HashMap<String, Vec<String>>,

    modules: HashMap<String, ConfigModule>,

    config_types: Vec<String>,

    repo_root: Option<PathBuf>,
    config_type: Option<String>,
    initialized: bool,
}

impl Configuration {
    pub fn new<P: AsRef<Path>>(repo_root: P, config_type: &str) -> Self {
        let repo_root = repo_root.as_ref().to_path_buf();
        let mut ret = Configuration {
            parser: Default::default(),
            config_module: Default::default(),
            modules: Default::default(),
            config_types: Default::default(),
            repo_root: Some(repo_root),
            config_type: Some(String::from(config_type)),
            initialized: false,
        };
        ret.init();
        ret
    }
    fn init(&mut self) {
        let json = Rc::new(RefCell::new(DefaultParser::new(Schema::JSON)));
        let toml = Rc::new(RefCell::new(DefaultParser::new(Schema::TOML)));
        self.register_parser(Schema::JSON, json);
        self.register_parser(Schema::TOML, toml);
    }
    fn register_parser(&mut self, s: Schema, p: Rc<RefCell<DefaultParser>>) {
        self.parser.insert(s, p);
    }

    pub fn get_config<T: serde::de::DeserializeOwned + Clone>(
        &self,
        module_name: &str,
    ) -> ConfigurationResult<T> {
        let module_name = String::from(module_name);
        let module = self.get_module(module_name.clone());
        if module.is_none() {
            return Err(ConfigurationError::ModuleNotExists);
        }
        let module = module.unwrap();
        self.get_config_object(&module.schema, module_name.clone())
    }

    fn get_config_object<T: serde::de::DeserializeOwned + Clone + Sized>(
        &self,
        schema: &Schema,
        module_name: String,
    ) -> ConfigurationResult<T> {
        let module = self.get_module(module_name.clone()).unwrap();
        let parser = self.parser.get(schema).map_or_else(|| panic!(), |v| v);
        let mut parser = parser.borrow_mut();
        let v = parser.parse_from(module_name.clone(), module.module_full_path.clone())?;
        v.as_object()
    }

    fn get_module(&self, module_name: String) -> Option<&ConfigModule> {
        self.modules.get(&module_name)
    }

    pub fn initialize(&mut self) -> ConfigurationResult<()> {
        let root_path = self.repo_root.as_ref().unwrap().clone();
        let config_type = self.config_type.as_ref().unwrap().clone();
        let root_json = root_path.clone().join("root.json");
        let data = fs::read(root_json).unwrap();
        let root_config: RootConfig =
            serde_json::from_slice::<RootConfig>(data.as_slice()).unwrap();

        let mut repos: HashMap<PathBuf, RootConfig> = Default::default();
        let config_types = root_config.get_config_types();
        if config_types.get(&config_type.clone()).is_none() {
            panic!()
        }
        repos.insert(root_path.clone(), root_config);

        let inheritance = self.build_inheritance_list(&config_types);
        self.build_module_path_map(&repos, &inheritance);

        for k in &config_types {
            self.config_types.push(k.0.clone());
        }
        self.initialized = true;
        Ok(())
    }

    fn build_module_path_map(
        &mut self,
        repos: &HashMap<PathBuf, RootConfig>,
        inheritance: &Vec<String>,
    ) {
        for (k, v) in repos {
            let mut modules = v.get_modules();
            let mut mut_iter = modules.iter_mut();
            loop {
                match mut_iter.next() {
                    None => break,
                    Some((k2, v2)) => {
                        let mut module_full_path = Default::default();
                        for type_def in inheritance {
                            let temp = k.join(type_def).join(v2.module_full_path.clone());
                            if !temp.exists() {
                                continue;
                            }
                            if v2.module_due_path.is_none() {
                                v2.set_due_path(temp.clone());
                            }
                            module_full_path = temp.clone();
                        }
                        v2.set_full_path(module_full_path);
                    }
                }
            }

            for (k, v) in modules {
                self.modules.insert(k.clone(), v);
            }
        }
    }
    fn build_inheritance_list(
        &mut self,
        config_types: &HashMap<String, Option<String>>,
    ) -> Vec<String> {
        let mut ret = Vec::new();
        ret.push(self.config_type.as_ref().unwrap().clone());

        let config_type = self.config_type.as_ref().unwrap();
        let mut parent = config_types.get(config_type);
        loop {
            match parent {
                None => {
                    break;
                }
                Some(p) => {
                    if p.is_none() {
                        break;
                    }
                    let internal = p.as_ref().unwrap();
                    ret.insert(0, internal.clone());
                    parent = config_types.get(internal);
                }
            }
        }

        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootConfig {
    types: HashMap<String, Types>,
    #[serde(rename = "defaultType")]
    default_type: String,
    configs: Vec<ConfigNode>,
    plugins: HashMap<String, serde_json::Value>,
}

impl RootConfig {
    pub fn get_config_types(&self) -> HashMap<String, Option<String>> {
        let mut ret = HashMap::default();
        for (k, v) in self.types.clone() {
            if k.as_str() == "Default" {
                ret.insert(k.clone(), None);
                continue;
            }
            let parent = v.parent;
            ret.insert(k.clone(), parent);
        }
        return ret;
    }

    pub fn get_modules(&self) -> HashMap<String, ConfigModule> {
        let mut ret = HashMap::new();
        for i in 0..self.configs.len() {
            let data = self.configs.get(i);
            if data.is_none() {
                continue;
            }
            let data = data.unwrap();
            let module = data.modules.clone();
            let mut schema = "json".to_string();
            if data.schema.is_some() {
                schema = data.schema.as_ref().unwrap().clone();
            }

            for (module_name, module_path) in module {
                ret.insert(
                    module_name,
                    ConfigModule::new(PathBuf::from(module_path), schema.clone()),
                );
            }
        }

        ret
    }
}

pub struct ConfigModule {
    pub module_full_path: PathBuf,
    pub module_due_path: Option<PathBuf>,
    schema: Schema,
}

impl ConfigModule {
    pub fn new(module_path: PathBuf, schema: String) -> Self {
        Self {
            module_full_path: module_path,
            module_due_path: Default::default(),
            schema: Schema::from(schema),
        }
    }
    pub fn set_full_path(&mut self, full: PathBuf) {
        self.module_full_path = full;
    }
    pub fn set_due_path(&mut self, path: PathBuf) {
        self.module_due_path = Some(path);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Types {
    parent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigNode {
    #[serde(rename = "modules")]
    modules: HashMap<String, String>,
    schema: Option<String>,
}
