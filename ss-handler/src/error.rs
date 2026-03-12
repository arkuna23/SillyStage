use engine::{EngineError, ManagerError, RegistryError, RuntimeError};
use protocol::{ErrorCode, ErrorPayload};
use store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("request requires session_id")]
    MissingSessionId,
    #[error("global engine config is not initialized")]
    MissingGlobalConfig,
    #[error("upload '{0}' not found")]
    MissingUpload(String),
    #[error("character '{0}' not found")]
    MissingCharacter(String),
    #[error("story resources '{0}' not found")]
    MissingStoryResources(String),
    #[error("story '{0}' not found")]
    MissingStory(String),
    #[error("session '{0}' not found")]
    MissingSession(String),
    #[error("character '{0}' already exists")]
    DuplicateCharacter(String),
    #[error("character_id must not be empty")]
    EmptyCharacterId,
    #[error("character '{0}' does not have a cover yet")]
    MissingCharacterCover(String),
    #[error("character '{0}' is still referenced by story resources")]
    CharacterInUse(String),
    #[error("story resources '{0}' already has generated stories")]
    StoryResourcesInUse(String),
    #[error("story '{0}' still has active sessions")]
    StoryHasSessions(String),
    #[error("character_ids cannot be empty")]
    EmptyCharacterIds,
    #[error("upload chunk index {got} does not match expected {expected}")]
    InvalidChunkIndex { expected: u64, got: u64 },
    #[error("upload chunk offset {got} does not match expected {expected}")]
    InvalidChunkOffset { expected: u64, got: u64 },
    #[error("upload total size mismatch: expected {expected}, got {got}")]
    UploadSizeMismatch { expected: u64, got: u64 },
    #[error("invalid upload chunk payload: {0}")]
    InvalidUploadChunkPayload(String),
    #[error("invalid character cover payload: {0}")]
    InvalidCharacterCoverPayload(String),
    #[error("session config for use_session requires api ids")]
    MissingSessionApiIds,
    #[error("invalid session config: {0}")]
    InvalidSessionConfig(String),
    #[error(transparent)]
    Archive(#[from] protocol::CharacterArchiveError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Store(#[from] StoreError),
}

impl HandlerError {
    pub fn to_error_payload(&self) -> ErrorPayload {
        match self {
            Self::MissingSessionId
            | Self::EmptyCharacterIds
            | Self::InvalidChunkIndex { .. }
            | Self::InvalidChunkOffset { .. }
            | Self::UploadSizeMismatch { .. }
            | Self::InvalidUploadChunkPayload(_)
            | Self::EmptyCharacterId
            | Self::InvalidCharacterCoverPayload(_)
            | Self::MissingSessionApiIds
            | Self::InvalidSessionConfig(_)
            | Self::Archive(_) => ErrorPayload::new(ErrorCode::InvalidRequest, self.to_string()),
            Self::MissingGlobalConfig
            | Self::MissingUpload(_)
            | Self::MissingCharacter(_)
            | Self::MissingStoryResources(_)
            | Self::MissingStory(_)
            | Self::MissingSession(_)
            | Self::Registry(_) => ErrorPayload::new(ErrorCode::NotFound, self.to_string()),
            Self::DuplicateCharacter(_)
            | Self::MissingCharacterCover(_)
            | Self::CharacterInUse(_)
            | Self::StoryResourcesInUse(_)
            | Self::StoryHasSessions(_) => ErrorPayload::new(ErrorCode::Conflict, self.to_string()),
            Self::Engine(_) | Self::Runtime(_) | Self::Store(_) => {
                ErrorPayload::new(ErrorCode::BackendError, self.to_string())
            }
        }
    }
}

impl From<ManagerError> for HandlerError {
    fn from(value: ManagerError) -> Self {
        match value {
            ManagerError::MissingGlobalConfig => Self::MissingGlobalConfig,
            ManagerError::MissingCharacter(id) => Self::MissingCharacter(id),
            ManagerError::MissingStoryResources(id) => Self::MissingStoryResources(id),
            ManagerError::MissingStory(id) => Self::MissingStory(id),
            ManagerError::MissingSession(id) => Self::MissingSession(id),
            ManagerError::EmptyCharacterIds => Self::EmptyCharacterIds,
            ManagerError::Engine(error) => Self::Engine(error),
            ManagerError::Runtime(error) => Self::Runtime(error),
            ManagerError::Registry(error) => Self::Registry(error),
            ManagerError::Store(error) => Self::Store(error),
        }
    }
}
