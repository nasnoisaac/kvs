use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Set { key: String, value: String },
    Get { key: String },
    Remove { key: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Ok(),
    Err(String),
}
