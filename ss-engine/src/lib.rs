pub mod engine;
pub mod event;
pub mod runtime;

pub use engine::{
    Engine, EngineError, EngineTurnResult, EngineTurnStream, ExecutedBeat, generate_story_graph,
    generate_story_plan,
};
pub use event::{EngineEvent, EngineStage};
pub use runtime::{RuntimeError, RuntimeSnapshot, RuntimeState, StoryResources};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
