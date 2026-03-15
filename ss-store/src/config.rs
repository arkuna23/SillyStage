use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    OpenAi,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiGroupAgentBindings {
    pub planner_api_id: String,
    pub architect_api_id: String,
    pub director_api_id: String,
    pub actor_api_id: String,
    pub narrator_api_id: String,
    pub keeper_api_id: String,
    pub replyer_api_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentPresetConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PresetAgentConfigs {
    pub planner: AgentPresetConfig,
    pub architect: AgentPresetConfig,
    pub director: AgentPresetConfig,
    pub actor: AgentPresetConfig,
    pub narrator: AgentPresetConfig,
    pub keeper: AgentPresetConfig,
    pub replyer: AgentPresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionBindingConfig {
    pub api_group_id: String,
    pub preset_id: String,
}
