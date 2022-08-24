use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    #[error("IO error")]
    IO(#[from] std::io::Error),

    #[error("Serde error")]
    Serde(#[from] serde_json::Error),

    #[error("No such key: {0}")]
    KeyNotFound(String),

    #[error("Invalid command")]
    UnexpectedCommandType,
}

pub type Result<T> = std::result::Result<T, KvsError>;
