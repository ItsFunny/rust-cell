use crate::enums::Schema;
use crate::error::{ConfigurationError, ConfigurationResult};
use crate::manager::Manager;
use crate::parser::{ConfigurationParser, JsonParser, ParserEnums};
use crate::value::ConfigValueTrait;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub struct Configuration {
    parser: HashMap<Schema, ParserEnums>,
    config_module: HashMap<String, Vec<String>>,

    modules: HashMap<String, ConfigModule>,

    config_types: Vec<String>,

    repo_root: Option<PathBuf>,
    config_type: Option<String>,
    initialized: bool,
    manager: Manager,
}

impl Configuration {
    pub fn new(manager: Manager) -> Self {
        let mut ret = Configuration {
            parser: Default::default(),
            config_module: Default::default(),
            modules: Default::default(),
            config_types: Default::default(),
            repo_root: Some(manager.root_path.clone()),
            config_type: Some(manager.config_type.clone()),
            initialized: false,
            manager: manager,
        };
        ret.init();
        ret
    }
    fn init(&mut self) {
        self.register_parser(
            Schema::JSON,
            ParserEnums::JSON(Rc::new(RefCell::new(JsonParser::default()))),
        );
    }
    fn register_parser(&mut self, s: Schema, p: ParserEnums) {
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
        match module.schema {
            Schema::JSON => self.get_json_config_object(&module.schema, module_name.clone()),
        }
    }

    fn get_json_config_object<T: serde::de::DeserializeOwned + Clone + Sized>(
        &self,
        schema: &Schema,
        module_name: String,
    ) -> ConfigurationResult<T> {
        let module = self.get_module(module_name.clone()).unwrap();
        let parser = self.parser.get(schema).map_or_else(|| panic!(), |v| v);
        match parser {
            ParserEnums::JSON(parser) => {
                let ret = parser
                    .borrow_mut()
                    .parse_from(module_name.clone(), module.module_full_path.clone())?;
                return ret.as_object();
            }
        }
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

#[cfg(test)]
mod tests {
    use crate::cfg::{Configuration, RootConfig};
    use crate::manager::Manager;
    use serde::{Deserialize, Serialize};


    #[test]
    fn test_get_obj() {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Nacos {
            pub server_addr: String,
        }

        let m = Manager::new("./config", "test1");
        let mut cfg = Configuration::new(m);
        cfg.initialize().unwrap();

        let nacos = cfg.get_config::<Nacos>("nacos").unwrap();
        println!("{:?}", nacos);
    }

    #[test]
    fn it_works() {
        let json = r#"{
        "types": {
            "test-asd2": {
                "parent": "test-asd"
            },
            "test-asd": {
                "parent": "Default"
            },
            "Default": {
                "parent": null
            }
        },
        "defaultType": "Default",
        "configs": [
            {
                "modules": {
                    "public.server.json": "public/server.json",
                    "nacos.properties": "env/shared/discovery/nacos.json",
                    "gateway.properties": "env/shared/gateway/gateway.json",
                    "gateway.metrics.properties": "env/shared/gateway/metrics/metrics.json",
                    "grpc.client.propertoes": "env/shared/rpc/grpc/client.json"
                },
                "schema": null
            }
        ],
        "plugins": {}
    }"#;

        let config: RootConfig = serde_json::from_str(json).unwrap();
        println!("{:#?}", config);
    }
}
