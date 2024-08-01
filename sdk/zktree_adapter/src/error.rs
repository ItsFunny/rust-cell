use std::fmt::{Display, Formatter};
use thiserror::Error;

pub type MerkleResult<T> = Result<T, MerkleError>;

#[derive(Debug, Error)]
pub enum MerkleError {
    #[error("error")]
    WithErrorCode([u8; 32], u64, MerkleErrorCode)
}

#[derive(Debug)]
pub enum MerkleErrorCode {
    InvalidLeafIndex,
    InvalidHash,
    InvalidDepth,
    InvalidIndex,
}

impl Display for MerkleErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MerkleErrorCode::InvalidLeafIndex => write!(f, "InvalidLeafIndex"),
            MerkleErrorCode::InvalidHash => write!(f, "InvalidHash"),
            MerkleErrorCode::InvalidDepth => write!(f, "InvalidDepth"),
            MerkleErrorCode::InvalidIndex => write!(f, "InvalidIndex"),
        }
    }
}