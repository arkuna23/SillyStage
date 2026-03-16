pub mod engine;
pub mod event;
mod logging;
pub mod manager;
pub mod registry;
pub mod runtime;

pub use agents::replyer::ReplyOption;
pub use engine::{
    AgentModelConfig, Engine, EngineError, EngineTurnResult, EngineTurnStream, ExecutedBeat,
    RuntimeAgentConfigs, StoryGenerationAgentConfigs, generate_story_graph, generate_story_plan,
};
pub use event::{EngineEvent, EngineStage};
pub use manager::{
    EngineManager, ManagedTurnStream, ManagerError, ResolvedSessionConfig, SessionCharacterUpdate,
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
