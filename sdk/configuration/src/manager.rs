use std::path::{Path, PathBuf};

pub struct Manager {
    pub root_path: PathBuf,
    pub config_type: String,
}

impl Manager {
    pub fn new<P: AsRef<Path>>(root_path: P, config_type: &str) -> Self {
        let root_path = root_path.as_ref().to_path_buf();
        if !root_path.exists() {
            panic!("configuration is not exist")
        }
        Self {
            root_path,
            config_type: config_type.to_string(),
        }
    }
}
