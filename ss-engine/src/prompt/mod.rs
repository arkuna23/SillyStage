mod compile;
mod contracts;
mod defaults;
mod normalize;
mod preview;
mod templates;
mod types;

pub(crate) use compile::{
    compile_architect_prompt_module, compile_architect_prompt_preview_profile,
    compile_prompt_module, compile_prompt_preview_profile,
};
pub use compile::{compile_architect_prompt_profiles, compile_prompt_profile};
pub use defaults::default_agent_preset_config;
pub use normalize::{compact_agent_preset_config, normalize_agent_preset_config};
pub(crate) use preview::{render_module_preview, render_profile_preview};
pub use types::{
    ArchitectPromptMode, PromptAgentKind, PromptConfigError, PromptPreview,
    PromptPreviewActorPurpose, PromptPreviewEntry, PromptPreviewEntrySource,
    PromptPreviewKeeperPhase, PromptPreviewMessage, PromptPreviewMessageRole, PromptPreviewModule,
    PromptPreviewNarratorPurpose, RuntimePromptPreviewOptions,
};
