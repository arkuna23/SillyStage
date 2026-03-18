use store::{PromptEntryKind, PromptModuleId};

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
    #[error("preset entry_id must not be empty")]
    EmptyEntryId,
    #[error("custom preset entry '{0}' must contain text")]
    EmptyCustomEntryText(String),
}

#[derive(Clone, Copy)]
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
