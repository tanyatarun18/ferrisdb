use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Key not found")]
    KeyNotFound,
    
    #[error("Corruption detected: {0}")]
    Corruption(String),
    
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    
    #[error("Storage engine error: {0}")]
    StorageEngine(String),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
}

pub type Result<T> = std::result::Result<T, Error>;