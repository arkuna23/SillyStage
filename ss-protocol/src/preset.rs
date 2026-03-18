use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PresetAgentIdPayload {
    Planner,
    Architect,
    Director,
    Actor,
    Narrator,
    Keeper,
    Replyer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptModuleIdPayload {
    Role,
    Task,
    StaticContext,
    DynamicContext,
    Output,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptEntryKindPayload {
    BuiltInText,
    BuiltInContextRef,
    CustomText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetModuleEntryPayload {
    pub entry_id: String,
    pub display_name: String,
    pub kind: PromptEntryKindPayload,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetModuleEntrySummaryPayload {
    pub entry_id: String,
    pub display_name: String,
    pub kind: PromptEntryKindPayload,
    pub enabled: bool,
    pub order: i32,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptModulePayload {
    pub module_id: PromptModuleIdPayload,
    #[serde(default)]
    pub entries: Vec<PresetModuleEntryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptModuleSummaryPayload {
    pub module_id: PromptModuleIdPayload,
    #[serde(default)]
    pub entry_count: usize,
    #[serde(default)]
    pub entries: Vec<PresetModuleEntrySummaryPayload>,
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
    pub modules: Vec<PresetPromptModulePayload>,
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
    pub module_count: usize,
    #[serde(default)]
    pub entry_count: usize,
    #[serde(default)]
    pub modules: Vec<PresetPromptModuleSummaryPayload>,
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
#[serde(deny_unknown_fields)]
pub struct PresetEntryCreateParams {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    pub module_id: PromptModuleIdPayload,
    pub entry_id: String,
    pub display_name: String,
    pub text: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetEntryUpdateParams {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    pub module_id: PromptModuleIdPayload,
    pub entry_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetEntryDeleteParams {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    pub module_id: PromptModuleIdPayload,
    pub entry_id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetEntryPayload {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    pub module_id: PromptModuleIdPayload,
    pub entry: PresetModuleEntryPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetEntryDeletedPayload {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    pub module_id: PromptModuleIdPayload,
    pub entry_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetDeletedPayload {
    pub preset_id: String,
}

fn default_enabled() -> bool {
    true
}
