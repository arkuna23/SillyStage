pub mod engine;
pub mod event;
mod logging;
mod lorebook;
pub mod manager;
pub mod prompt;
pub mod registry;
pub mod runtime;

pub use agents::{ArchitectPromptProfiles, replyer::ReplyOption};
pub use engine::{
    AgentModelConfig, ArchitectModelConfig, Engine, EngineError, EngineTurnResult,
    EngineTurnStream, ExecutedBeat, RuntimeAgentConfigs, StoryGenerationAgentConfigs,
    generate_story_graph, generate_story_plan,
};
pub use event::{EngineEvent, EngineStage};
pub use manager::{
    EngineManager, ManagedTurnStream, ManagerError, ResolvedSessionConfig, SessionCharacterUpdate,
};
pub use prompt::{
    PromptAgentKind, PromptConfigError, compile_architect_prompt_profiles, compile_prompt_profile,
    default_agent_preset_config, normalize_agent_preset_config,
};
pub use registry::{LlmApiRegistry, RegisteredApi, RegistryError, RuntimeApiRecords};
pub use runtime::{RuntimeError, RuntimeState, StoryResources};
pub use store::RuntimeSnapshot;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
