use thiserror::Error;

pub type MerkleResult<T> = Result<T, MerkleError>;

#[derive(Debug, Error)]
pub enum MerkleError {
    #[error("Merkle error: {0},index:{1},code:{2}")]
    WithErrorCode([u8; 32], u64, MerkleErrorCode)
}

#[derive(Debug)]
pub enum MerkleErrorCode {
    InvalidLeafIndex,
    InvalidHash,
    InvalidDepth,
    InvalidIndex,
}