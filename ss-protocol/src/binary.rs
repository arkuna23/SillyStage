use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceFileRefPayload {
    pub resource_id: String,
    pub file_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceFilePayload {
    pub resource_id: String,
    pub file_id: String,
    pub file_name: Option<String>,
    pub content_type: String,
    pub size_bytes: u64,
}
