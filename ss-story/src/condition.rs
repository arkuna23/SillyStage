use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub key: String,
    pub op: ConditionOperator,
    pub value: Value,
}

impl Condition {
    pub fn new(key: impl Into<String>, op: ConditionOperator, value: Value) -> Self {
        Self {
            key: key.into(),
            op,
            value,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}
