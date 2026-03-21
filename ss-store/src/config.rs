use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LlmProvider {
    OpenAi,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PromptModuleId {
    Role,
    Task,
    StaticContext,
    DynamicContext,
    Output,
    Custom(String),
}

impl PromptModuleId {
    pub fn from_raw(raw: String) -> Self {
        match raw.as_str() {
            "role" => Self::Role,
            "task" => Self::Task,
            "static_context" => Self::StaticContext,
            "dynamic_context" => Self::DynamicContext,
            "output" => Self::Output,
            _ => Self::Custom(raw),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Role => "role",
            Self::Task => "task",
            Self::StaticContext => "static_context",
            Self::DynamicContext => "dynamic_context",
            Self::Output => "output",
            Self::Custom(value) => value.as_str(),
        }
    }
}

impl Serialize for PromptModuleId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PromptModuleId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_raw(String::deserialize(deserializer)?))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptMessageRole {
    System,
    User,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptEntryKind {
    BuiltInText,
    BuiltInContextRef,
    CustomText,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentPromptModuleEntryConfig {
    pub entry_id: String,
    pub display_name: String,
    pub kind: PromptEntryKind,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AgentPromptModuleConfig {
    pub module_id: PromptModuleId,
    pub display_name: String,
    pub message_role: PromptMessageRole,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub entries: Vec<AgentPromptModuleEntryConfig>,
}

impl<'de> Deserialize<'de> for AgentPromptModuleConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawAgentPromptModuleConfig {
            module_id: PromptModuleId,
            #[serde(default)]
            display_name: Option<String>,
            #[serde(default)]
            message_role: Option<PromptMessageRole>,
            #[serde(default)]
            order: Option<i32>,
            #[serde(default)]
            entries: Vec<AgentPromptModuleEntryConfig>,
        }

        let raw = RawAgentPromptModuleConfig::deserialize(deserializer)?;
        let defaults = builtin_module_defaults(&raw.module_id);

        Ok(Self {
            display_name: raw.display_name.unwrap_or_else(|| {
                defaults
                    .map(|defaults| defaults.display_name.to_owned())
                    .unwrap_or_else(|| raw.module_id.as_str().to_owned())
            }),
            message_role: raw.message_role.unwrap_or_else(|| {
                defaults
                    .map(|defaults| defaults.message_role)
                    .unwrap_or(PromptMessageRole::User)
            }),
            order: raw
                .order
                .unwrap_or_else(|| defaults.map(|defaults| defaults.order).unwrap_or(1_000)),
            module_id: raw.module_id,
            entries: raw.entries,
        })
    }
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
    pub director_shared_history_limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_shared_history_limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_private_memory_limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub narrator_shared_history_limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replyer_session_history_limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra: Option<Value>,
    #[serde(default)]
    pub modules: Vec<AgentPromptModuleConfig>,
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

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Copy)]
struct BuiltInModuleDefaults {
    display_name: &'static str,
    message_role: PromptMessageRole,
    order: i32,
}

fn builtin_module_defaults(module_id: &PromptModuleId) -> Option<BuiltInModuleDefaults> {
    match module_id {
        PromptModuleId::Role => Some(BuiltInModuleDefaults {
            display_name: "Role",
            message_role: PromptMessageRole::System,
            order: 10,
        }),
        PromptModuleId::Task => Some(BuiltInModuleDefaults {
            display_name: "Task",
            message_role: PromptMessageRole::System,
            order: 20,
        }),
        PromptModuleId::StaticContext => Some(BuiltInModuleDefaults {
            display_name: "Static Context",
            message_role: PromptMessageRole::User,
            order: 30,
        }),
        PromptModuleId::DynamicContext => Some(BuiltInModuleDefaults {
            display_name: "Dynamic Context",
            message_role: PromptMessageRole::User,
            order: 40,
        }),
        PromptModuleId::Output => Some(BuiltInModuleDefaults {
            display_name: "Output",
            message_role: PromptMessageRole::System,
            order: 50,
        }),
        PromptModuleId::Custom(_) => None,
    }
}
