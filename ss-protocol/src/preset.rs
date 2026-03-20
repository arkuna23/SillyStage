use serde::{Deserialize, Deserializer, Serialize, Serializer};
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PromptModuleIdPayload {
    Role,
    Task,
    StaticContext,
    DynamicContext,
    Output,
    Custom(String),
}

impl PromptModuleIdPayload {
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

impl Serialize for PromptModuleIdPayload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PromptModuleIdPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_raw(String::deserialize(deserializer)?))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptMessageRolePayload {
    System,
    User,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptEntryKindPayload {
    BuiltInText,
    BuiltInContextRef,
    CustomText,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptPreviewKindPayload {
    Template,
    Runtime,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptPreviewMessageRolePayload {
    System,
    User,
    Full,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptPreviewEntrySourcePayload {
    Preset,
    Synthetic,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectPromptModePayload {
    Graph,
    DraftInit,
    DraftContinue,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptPreviewActorPurposePayload {
    AdvanceGoal,
    ReactToPlayer,
    CommentOnScene,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptPreviewNarratorPurposePayload {
    DescribeTransition,
    DescribeScene,
    DescribeResult,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PromptPreviewKeeperPhasePayload {
    AfterPlayerInput,
    AfterTurnOutputs,
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

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptModulePayload {
    pub module_id: PromptModuleIdPayload,
    pub display_name: String,
    pub message_role: PromptMessageRolePayload,
    #[serde(default)]
    pub order: i32,
    #[serde(default)]
    pub entries: Vec<PresetModuleEntryPayload>,
}

impl<'de> Deserialize<'de> for PresetPromptModulePayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawPresetPromptModulePayload {
            module_id: PromptModuleIdPayload,
            #[serde(default)]
            display_name: Option<String>,
            #[serde(default)]
            message_role: Option<PromptMessageRolePayload>,
            #[serde(default)]
            order: Option<i32>,
            #[serde(default)]
            entries: Vec<PresetModuleEntryPayload>,
        }

        let raw = RawPresetPromptModulePayload::deserialize(deserializer)?;
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
                    .unwrap_or(PromptMessageRolePayload::User)
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
#[serde(deny_unknown_fields)]
pub struct PresetPromptModuleSummaryPayload {
    pub module_id: PromptModuleIdPayload,
    pub display_name: String,
    pub message_role: PromptMessageRolePayload,
    pub order: i32,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPreviewTemplateParams {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<PromptModuleIdPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architect_mode: Option<ArchitectPromptModePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPreviewRuntimeParams {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<PromptModuleIdPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architect_mode: Option<ArchitectPromptModePayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_purpose: Option<PromptPreviewActorPurposePayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub narrator_purpose: Option<PromptPreviewNarratorPurposePayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keeper_phase: Option<PromptPreviewKeeperPhasePayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_input: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_limit: Option<u32>,
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
#[serde(deny_unknown_fields)]
pub struct PresetPromptPreviewEntryPayload {
    pub entry_id: String,
    pub display_name: String,
    pub kind: PromptEntryKindPayload,
    pub order: i32,
    pub source: PromptPreviewEntrySourcePayload,
    pub compiled_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptPreviewModulePayload {
    pub module_id: PromptModuleIdPayload,
    pub display_name: String,
    pub order: i32,
    #[serde(default)]
    pub entries: Vec<PresetPromptPreviewEntryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptPreviewMessagePayload {
    pub role: PromptMessageRolePayload,
    #[serde(default)]
    pub modules: Vec<PresetPromptPreviewModulePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PresetPromptPreviewPayload {
    pub preset_id: String,
    pub agent: PresetAgentIdPayload,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<PromptModuleIdPayload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architect_mode: Option<ArchitectPromptModePayload>,
    pub preview_kind: PromptPreviewKindPayload,
    pub message_role: PromptPreviewMessageRolePayload,
    #[serde(default)]
    pub messages: Vec<PresetPromptPreviewMessagePayload>,
    #[serde(default)]
    pub unresolved_context_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresetDeletedPayload {
    pub preset_id: String,
}

fn default_enabled() -> bool {
    true
}

#[derive(Clone, Copy)]
struct BuiltInModuleDefaults {
    display_name: &'static str,
    message_role: PromptMessageRolePayload,
    order: i32,
}

fn builtin_module_defaults(module_id: &PromptModuleIdPayload) -> Option<BuiltInModuleDefaults> {
    match module_id {
        PromptModuleIdPayload::Role => Some(BuiltInModuleDefaults {
            display_name: "Role",
            message_role: PromptMessageRolePayload::System,
            order: 10,
        }),
        PromptModuleIdPayload::Task => Some(BuiltInModuleDefaults {
            display_name: "Task",
            message_role: PromptMessageRolePayload::System,
            order: 20,
        }),
        PromptModuleIdPayload::StaticContext => Some(BuiltInModuleDefaults {
            display_name: "Static Context",
            message_role: PromptMessageRolePayload::User,
            order: 30,
        }),
        PromptModuleIdPayload::DynamicContext => Some(BuiltInModuleDefaults {
            display_name: "Dynamic Context",
            message_role: PromptMessageRolePayload::User,
            order: 40,
        }),
        PromptModuleIdPayload::Output => Some(BuiltInModuleDefaults {
            display_name: "Output",
            message_role: PromptMessageRolePayload::System,
            order: 50,
        }),
        PromptModuleIdPayload::Custom(_) => None,
    }
}
