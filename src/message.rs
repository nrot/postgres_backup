use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub source: String,
    pub filename: String,
    pub dst: String,
    pub error: String,
    pub orig_size: u64,
    pub back_size: u64,
    pub time_spent: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Record {
    pub timestamp: String,
    pub index_name: String,
    pub version: String,
    pub password: String,
    pub host: String,
    pub message: Option<Message>,
}
