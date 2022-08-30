use crate::error::{KvsError, Result};

/// Trait for key-value store engine.
pub trait KvsEngine{
    fn set(&mut self, key: String, value: String) -> Result<()>;
    fn get(&mut self, key: String) -> Result<String>;
    fn remove(&mut self, key: String) -> Result<()>;
}
