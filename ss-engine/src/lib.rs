pub mod engine;
pub mod event;
pub mod manager;
pub mod registry;
pub mod runtime;

pub use agents::replyer::ReplyOption;
pub use engine::{
    AgentModelConfig, Engine, EngineError, EngineTurnResult, EngineTurnStream, ExecutedBeat,
    RuntimeAgentConfigs, StoryGenerationAgentConfigs, generate_story_graph, generate_story_plan,
};
pub use event::{EngineEvent, EngineStage};
pub use manager::{EngineManager, ManagedTurnStream, ManagerError, ResolvedSessionConfig};
pub use registry::{LlmApiRegistry, RegisteredApi, RegistryError};
pub use runtime::{RuntimeError, RuntimeState, StoryResources};
pub use store::{
    AgentApiIdOverrides, AgentApiIds, RuntimeSnapshot, SessionConfigMode, SessionEngineConfig,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
