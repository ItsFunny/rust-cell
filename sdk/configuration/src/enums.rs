#[derive(Eq, PartialEq, Hash)]
pub enum Schema {
    JSON,
    TOML,
}
impl From<String> for Schema {
    fn from(value: String) -> Self {
        if value.as_str() == "json" {
            return Schema::JSON;
        } else if value.as_str() == "toml" {
            return Schema::TOML;
        }
        panic!("invalid schema")
    }
}
impl ToString for Schema {
    fn to_string(&self) -> String {
        match self {
            Schema::JSON => String::from("json"),
            Schema::TOML => String::from("toml"),
        }
    }
}

#[derive(Clone)]
pub struct ModuleKey {
    pub module_name: String,
}

impl ModuleKey {
    pub fn new(module_name: String) -> Self {
        Self { module_name }
    }
}

#[derive(Clone)]
pub struct ModuleValue {
    data: Vec<u8>,
}

impl ModuleValue {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
