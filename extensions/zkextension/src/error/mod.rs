pub use thiserror::Error;

pub type ZKResult<T> = Result<T, ZKError>;

#[derive(Debug, Error)]
pub enum ZKError {}
