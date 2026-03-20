use store::{PromptEntryKind, PromptMessageRole, PromptModuleId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptAgentKind {
    Planner,
    Architect,
    Director,
    Actor,
    Narrator,
    Keeper,
    Replyer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchitectPromptMode {
    Graph,
    DraftInit,
    DraftContinue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptPreviewMessageRole {
    System,
    User,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptPreviewEntrySource {
    Preset,
    Synthetic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptPreviewActorPurpose {
    AdvanceGoal,
    ReactToPlayer,
    CommentOnScene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptPreviewNarratorPurpose {
    DescribeTransition,
    DescribeScene,
    DescribeResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromptPreviewKeeperPhase {
    AfterPlayerInput,
    AfterTurnOutputs,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimePromptPreviewOptions {
    pub character_id: Option<String>,
    pub actor_purpose: Option<PromptPreviewActorPurpose>,
    pub narrator_purpose: Option<PromptPreviewNarratorPurpose>,
    pub keeper_phase: Option<PromptPreviewKeeperPhase>,
    pub previous_node_id: Option<String>,
    pub player_input: Option<String>,
    pub reply_limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPreview {
    pub message_role: PromptPreviewMessageRole,
    pub messages: Vec<PromptPreviewMessage>,
    pub unresolved_context_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPreviewMessage {
    pub role: PromptMessageRole,
    pub modules: Vec<PromptPreviewModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPreviewModule {
    pub module_id: PromptModuleId,
    pub display_name: String,
    pub order: i32,
    pub entries: Vec<PromptPreviewEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptPreviewEntry {
    pub entry_id: String,
    pub display_name: String,
    pub kind: PromptEntryKind,
    pub order: i32,
    pub source: PromptPreviewEntrySource,
    pub compiled_text: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PromptConfigError {
    #[error(
        "unknown built-in preset entry '{entry_id}' in module '{module_id:?}' for agent '{agent:?}'"
    )]
    UnknownBuiltInEntry {
        agent: PromptAgentKind,
        module_id: PromptModuleId,
        entry_id: String,
    },
    #[error("duplicate preset entry '{entry_id}' in module '{module_id:?}' for agent '{agent:?}'")]
    DuplicateEntryId {
        agent: PromptAgentKind,
        module_id: PromptModuleId,
        entry_id: String,
    },
    #[error("duplicate preset module '{module_id:?}' for agent '{agent:?}'")]
    DuplicateModuleId {
        agent: PromptAgentKind,
        module_id: PromptModuleId,
    },
    #[error("preset module_id must not be empty")]
    EmptyModuleId,
    #[error("preset entry_id must not be empty")]
    EmptyEntryId,
    #[error("custom preset entry '{0}' must contain text")]
    EmptyCustomEntryText(String),
}

#[derive(Clone)]
pub(super) struct BuiltInModuleTemplate {
    pub(super) module_id: PromptModuleId,
    pub(super) display_name: &'static str,
    pub(super) message_role: PromptMessageRole,
    pub(super) order: i32,
}

#[derive(Clone)]
pub(super) struct BuiltInEntryTemplate {
    pub(super) module_id: PromptModuleId,
    pub(super) entry_id: &'static str,
    pub(super) display_name: &'static str,
    pub(super) kind: PromptEntryKind,
    pub(super) required: bool,
    pub(super) order: i32,
    pub(super) text: Option<&'static str>,
    pub(super) context_key: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledPromptPreviewProfile {
    pub(crate) system_modules: Vec<CompiledPromptPreviewModule>,
    pub(crate) user_modules: Vec<CompiledPromptPreviewModule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledPromptPreviewModule {
    pub(crate) module_id: PromptModuleId,
    pub(crate) display_name: String,
    pub(crate) order: i32,
    pub(crate) entries: Vec<CompiledPromptPreviewEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledPromptPreviewEntry {
    pub(crate) entry_id: String,
    pub(crate) display_name: String,
    pub(crate) kind: PromptEntryKind,
    pub(crate) order: i32,
    pub(crate) source: PromptPreviewEntrySource,
    pub(crate) value: CompiledPromptPreviewEntryValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CompiledPromptPreviewEntryValue {
    Text(String),
    ContextRef(String),
}
