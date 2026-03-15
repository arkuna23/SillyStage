use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentPresetConfigPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetAgentPayloads {
    pub planner: AgentPresetConfigPayload,
    pub architect: AgentPresetConfigPayload,
    pub director: AgentPresetConfigPayload,
    pub actor: AgentPresetConfigPayload,
    pub narrator: AgentPresetConfigPayload,
    pub keeper: AgentPresetConfigPayload,
    pub replyer: AgentPresetConfigPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetCreateParams {
    pub preset_id: String,
    pub display_name: String,
    pub agents: PresetAgentPayloads,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetGetParams {
    pub preset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct PresetListParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetUpdateParams {
    pub preset_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agents: Option<PresetAgentPayloads>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetDeleteParams {
    pub preset_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetPayload {
    pub preset_id: String,
    pub display_name: String,
    pub agents: PresetAgentPayloads,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetsListedPayload {
    pub presets: Vec<PresetPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetDeletedPayload {
    pub preset_id: String,
}
