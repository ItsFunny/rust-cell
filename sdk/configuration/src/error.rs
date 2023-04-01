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
}
