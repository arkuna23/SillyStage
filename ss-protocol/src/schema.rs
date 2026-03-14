use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use state::StateFieldSchema;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaCreateParams {
    pub schema_id: String,
    pub display_name: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub fields: HashMap<String, StateFieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SchemaGetParams {
    pub schema_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct SchemaListParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaUpdateParams {
    pub schema_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, StateFieldSchema>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SchemaDeleteParams {
    pub schema_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPayload {
    pub schema_id: String,
    pub display_name: String,
    pub tags: Vec<String>,
    pub fields: HashMap<String, StateFieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemasListedPayload {
    pub schemas: Vec<SchemaPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaDeletedPayload {
    pub schema_id: String,
}
