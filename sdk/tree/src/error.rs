use crate::error::TreeError::Unknown;
use merk::rocksdb;
use sp_trie::Error;
pub use thiserror::Error;
use trie_db::TrieError;

pub type TreeResult<T> = Result<T, TreeError>;

#[derive(Debug, Error)]
pub enum TreeError {
    #[error("unknown")]
    Unknown,
    #[error("{0}")]
    Other(String),

    #[error("Store Error: {0}")]
    Store(String),

    #[error("AccountNotExist")]
    AccountNotExist,

    #[error("AccountAlreadyExist")]
    AccountAlreadyExist,

    #[error("OrderAlreadyExist")]
    OrderAlreadyExist,

    #[error("{0}")]
    ConvertStateRootError(String),

    #[error("rocksdb error: {0}")]
    RocksDBError(#[from] rocksdb::Error),

    #[error("merk error: {0}")]
    MerkError(#[from] merk::Error),
}

impl From<serde_json::Error> for TreeError {
    fn from(_: serde_json::Error) -> Self {
        Unknown
    }
}

impl From<sqlx::Error> for TreeError {
    fn from(e: sqlx::Error) -> Self {
        Self::Store(format!("sqlx error: {}", e))
    }
}

impl From<rmp_serde::decode::Error> for TreeError {
    fn from(e: rmp_serde::decode::Error) -> Self {
        Self::Other(format!("rmp_serde decode error: {}", e))
    }
}
impl std::convert::From<Box<trie_db::TrieError<[u8; 32], sp_trie::Error<[u8; 32]>>>> for TreeError {
    fn from(_value: Box<TrieError<[u8; 32], Error<[u8; 32]>>>) -> Self {
        Unknown
    }
}
impl From<rmp_serde::encode::Error> for TreeError {
    fn from(e: rmp_serde::encode::Error) -> Self {
        Self::Other(format!("rmp_serde encode error: {}", e))
    }
}

impl From<anyhow::Error> for TreeError {
    fn from(_value: anyhow::Error) -> Self {
        Unknown
    }
}
