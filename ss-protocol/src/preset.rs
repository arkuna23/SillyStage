use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptEntryPayload {
    pub entry_id: String,
    pub title: String,
    pub content: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptEntrySummaryPayload {
    pub entry_id: String,
    pub title: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentPresetConfigPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
    #[serde(default)]
    pub prompt_entries: Vec<PresetPromptEntryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AgentPresetConfigSummaryPayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
    #[serde(default)]
    pub prompt_entry_count: usize,
    #[serde(default)]
    pub prompt_entries: Vec<PresetPromptEntrySummaryPayload>,
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
pub struct PresetAgentSummaryPayloads {
    pub planner: AgentPresetConfigSummaryPayload,
    pub architect: AgentPresetConfigSummaryPayload,
    pub director: AgentPresetConfigSummaryPayload,
    pub actor: AgentPresetConfigSummaryPayload,
    pub narrator: AgentPresetConfigSummaryPayload,
    pub keeper: AgentPresetConfigSummaryPayload,
    pub replyer: AgentPresetConfigSummaryPayload,
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
pub struct PresetSummaryPayload {
    pub preset_id: String,
    pub display_name: String,
    pub agents: PresetAgentSummaryPayloads,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetsListedPayload {
    pub presets: Vec<PresetSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetDeletedPayload {
    pub preset_id: String,
}

fn default_enabled() -> bool {
    true
}
