use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::StateUpdate;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetSessionVariablesParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateSessionVariablesParams {
    pub update: StateUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionVariablesPayload {
    pub custom: HashMap<String, Value>,
    pub player_state: HashMap<String, Value>,
    pub character_state: HashMap<String, HashMap<String, Value>>,
}
