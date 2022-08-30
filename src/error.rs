use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("IO error")]
    IO(#[from] std::io::Error),

    #[error("Serde error")]
    Serde(#[from] serde_json::Error),

    #[error("Key not found")]
    KeyNotFound(String),

    #[error("Invalid command")]
    UnexpectedCommandType,

    #[error("{0}")]
    StringError(String),

    #[error("sled error: {0}")]
    Sled(#[from] sled::Error),

    #[error("utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, KvsError>;
