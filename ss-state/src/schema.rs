use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldStateSchema {
    pub fields: HashMap<String, StateFieldSchema>,
}

impl WorldStateSchema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    pub fn insert_field(
        &mut self,
        key: impl Into<String>,
        field: StateFieldSchema,
    ) -> Option<StateFieldSchema> {
        self.fields.insert(key.into(), field)
    }

    pub fn get_field(&self, key: &str) -> Option<&StateFieldSchema> {
        self.fields.get(key)
    }

    pub fn has_field(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }

    pub fn remove_field(&mut self, key: &str) -> Option<StateFieldSchema> {
        self.fields.remove(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateFieldSchema {
    pub value_type: StateValueType,

    pub default: Option<Value>,

    pub description: Option<String>,
}

impl StateFieldSchema {
    pub fn new(value_type: StateValueType) -> Self {
        Self {
            value_type,
            default: None,
            description: None,
        }
    }

    pub fn with_default(mut self, value: Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StateValueType {
    Bool,
    Int,
    Float,
    String,
    Array,
    Object,
    Null,
}
