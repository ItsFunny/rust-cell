use crate::cfg::Configuration;
use crate::error::ConfigurationResult;
use std::path::{Path, PathBuf};

pub struct Manager {
    pub root_path: PathBuf,
    pub config_type: String,

    pub current_configuration: Configuration,
}

impl Manager {
    pub fn new_with_init<P: AsRef<Path>>(root_path: P, config_type: &str) -> Self {
        let mut m = Self::new(root_path, config_type);
        m.initialize().unwrap();
        m
    }

    pub fn new<P: AsRef<Path>>(root_path: P, config_type: &str) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        if !root_path.exists() {
            panic!("configuration is not exist")
        }
        let cfg = Configuration::new(root_path.clone(), config_type);
        Self {
            root_path: root_path.clone(),
            config_type: config_type.to_string(),
            current_configuration: cfg,
        }
    }

    pub fn initialize(&mut self) -> ConfigurationResult<()> {
        self.current_configuration.initialize()
    }

    pub fn get_configuration(&self) -> &Configuration {
        &self.current_configuration
    }
    pub fn get_configuration_mut(&mut self) -> &mut Configuration {
        &mut self.current_configuration
    }
}

#[cfg(test)]
mod tests {
    use crate::cfg::{Configuration, RootConfig};
    use crate::manager::Manager;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Nacos {
        pub server_addr: String,
    }
    #[test]
    pub fn test_toml() {
        let manager = Manager::new_with_init("./config_toml", "Default");
        let nacos = manager
            .get_configuration()
            .get_config::<Nacos>("nacos")
            .unwrap();
        println!("{:?}", nacos);
    }
    #[test]
    fn test_get_obj() {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub struct Test {
            pub test: String,
        }

        let manager = Manager::new_with_init("./config", "test2");

        let nacos = manager
            .get_configuration()
            .get_config::<Nacos>("nacos")
            .unwrap();
        println!("{:?}", nacos);

        let test = manager
            .get_configuration()
            .get_config::<Test>("test")
            .unwrap();
        println!("{:?}", test);
    }
}
