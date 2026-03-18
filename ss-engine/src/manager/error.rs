use agents::architect::ArchitectError;
use agents::replyer::ReplyerError;
use store::StoreError;

use crate::{EngineError, RegistryError, RuntimeError};

#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("llm engine config is not initialized")]
    LlmConfigNotInitialized,
    #[error("api '{0}' not found")]
    MissingApi(String),
    #[error("api group '{0}' not found")]
    MissingApiGroup(String),
    #[error("preset '{0}' not found")]
    MissingPreset(String),
    #[error("schema '{0}' not found")]
    MissingSchema(String),
    #[error("lorebook '{0}' not found")]
    MissingLorebook(String),
    #[error("character '{0}' not found")]
    MissingCharacter(String),
    #[error("player profile '{0}' not found")]
    MissingPlayerProfile(String),
    #[error("story resources '{0}' not found")]
    MissingStoryResources(String),
    #[error("story draft '{0}' not found")]
    MissingStoryDraft(String),
    #[error("story '{0}' not found")]
    MissingStory(String),
    #[error("session '{0}' not found")]
    MissingSession(String),
    #[error("session character '{0}' not found")]
    MissingSessionCharacter(String),
    #[error("character_ids cannot be empty")]
    EmptyCharacterIds,
    #[error("invalid generated schema: {0}")]
    InvalidGeneratedSchema(String),
    #[error("invalid common variable: {0}")]
    InvalidCommonVariable(String),
    #[error("invalid story draft: {0}")]
    InvalidDraft(String),
    #[error(transparent)]
    Architect(#[from] ArchitectError),
    #[error(transparent)]
    Replyer(#[from] ReplyerError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Store(#[from] StoreError),
}
