use jsonnet::Error;
pub use thiserror::Error;

pub type ConfigurationResult<T> = Result<T, ConfigurationError>;

#[derive(Debug, Error)]
pub enum ConfigurationError {
    #[error("unknown")]
    Unknown,
    #[error("module not exist")]
    ModuleNotExists,

    #[error("serde error:{0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("io error:{0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    StringError(String),

    #[error("anyhow errror {0}")]
    AnyHowError(#[from] anyhow::Error),
}

impl<'a> From<jsonnet::Error<'a>> for ConfigurationError {
    fn from(value: Error<'a>) -> Self {
        let err = value.to_string();
        ConfigurationError::StringError(err)
    }
}
