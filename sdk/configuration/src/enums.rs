#[derive(Eq,PartialEq,Hash)]
pub enum Schema {
    JSON,
}
impl From<String> for Schema {
    fn from(value: String) -> Self {
        if value.as_str() == "json" {
            return Schema::JSON;
        }
        panic!()
    }
}
impl ToString for Schema {
    fn to_string(&self) -> String {
        match self {
            Schema::JSON => String::from("json"),
        }
    }
}
