use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorldStateSchema {
    pub fields: HashMap<String, StateFieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerStateSchema {
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

impl PlayerStateSchema {
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

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

impl StateFieldSchema {
    pub fn new(value_type: StateValueType) -> Self {
        Self {
            value_type,
            default: None,
            description: None,
            enum_values: None,
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

    pub fn with_enum_values(mut self, enum_values: Vec<Value>) -> Self {
        self.enum_values = Some(enum_values);
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if let Some(default) = &self.default {
            self.value_type
                .validate_value(default)
                .map_err(|error| format!("default {error}"))?;
        }

        if let Some(enum_values) = self
            .enum_values
            .as_ref()
            .filter(|enum_values| !enum_values.is_empty())
        {
            if !self.value_type.supports_enum_values() {
                return Err(format!(
                    "enum_values are only supported for scalar types, got {}",
                    self.value_type.as_str()
                ));
            }

            for value in enum_values {
                self.value_type
                    .validate_value(value)
                    .map_err(|error| format!("enum_values item {error}"))?;
            }

            if let Some(default) = &self.default {
                if !enum_values.iter().any(|value| value == default) {
                    return Err(
                        "default must be one of enum_values when enum_values are configured"
                            .to_owned(),
                    );
                }
            }
        }

        Ok(())
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

impl StateValueType {
    pub fn validate_value(&self, value: &Value) -> Result<(), String> {
        let matches = match self {
            Self::Bool => value.is_boolean(),
            Self::Int => value.as_i64().is_some() || value.as_u64().is_some(),
            Self::Float => value.is_number(),
            Self::String => value.is_string(),
            Self::Array => value.is_array(),
            Self::Object => value.is_object(),
            Self::Null => value.is_null(),
        };

        if matches {
            Ok(())
        } else {
            Err(format!(
                "must match value_type '{}', got {}",
                self.as_str(),
                value
            ))
        }
    }

    pub const fn supports_enum_values(&self) -> bool {
        matches!(self, Self::Bool | Self::Int | Self::Float | Self::String)
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Int => "int",
            Self::Float => "float",
            Self::String => "string",
            Self::Array => "array",
            Self::Object => "object",
            Self::Null => "null",
        }
    }
}
