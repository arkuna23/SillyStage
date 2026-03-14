use engine::{EngineError, ManagerError, RegistryError, RuntimeError};
use protocol::{ErrorCode, ErrorPayload};
use store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("request requires session_id")]
    MissingSessionId,
    #[error("global engine config is not initialized")]
    MissingGlobalConfig,
    #[error("schema '{0}' not found")]
    MissingSchema(String),
    #[error("player profile '{0}' not found")]
    MissingPlayerProfile(String),
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
    #[error("llm api '{0}' not found")]
    MissingLlmApi(String),
    #[error("character '{0}' already exists")]
    DuplicateCharacter(String),
    #[error("schema '{0}' already exists")]
    DuplicateSchema(String),
    #[error("player profile '{0}' already exists")]
    DuplicatePlayerProfile(String),
    #[error("llm api '{0}' already exists")]
    DuplicateLlmApi(String),
    #[error("schema_id must not be empty")]
    EmptySchemaId,
    #[error("player_profile_id must not be empty")]
    EmptyPlayerProfileId,
    #[error("character_id must not be empty")]
    EmptyCharacterId,
    #[error("character_id '{expected}' does not match content.id '{got}'")]
    CharacterIdMismatch { expected: String, got: String },
    #[error("api_id must not be empty")]
    EmptyLlmApiId,
    #[error("character '{0}' does not have a cover yet")]
    MissingCharacterCover(String),
    #[error("schema '{0}' is still referenced")]
    SchemaInUse(String),
    #[error("player profile '{0}' is still referenced by a session")]
    PlayerProfileInUse(String),
    #[error("character '{0}' is still referenced by story resources")]
    CharacterInUse(String),
    #[error("llm api '{0}' is still referenced by config")]
    LlmApiInUse(String),
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
            | Self::EmptySchemaId
            | Self::EmptyPlayerProfileId
            | Self::EmptyCharacterId
            | Self::CharacterIdMismatch { .. }
            | Self::EmptyLlmApiId
            | Self::InvalidCharacterCoverPayload(_)
            | Self::MissingSessionApiIds
            | Self::InvalidSessionConfig(_)
            | Self::Archive(_) => ErrorPayload::new(ErrorCode::InvalidRequest, self.to_string()),
            Self::MissingGlobalConfig
            | Self::MissingUpload(_)
            | Self::MissingSchema(_)
            | Self::MissingPlayerProfile(_)
            | Self::MissingCharacter(_)
            | Self::MissingStoryResources(_)
            | Self::MissingStory(_)
            | Self::MissingSession(_)
            | Self::MissingLlmApi(_) => ErrorPayload::new(ErrorCode::NotFound, self.to_string()),
            Self::DuplicateCharacter(_)
            | Self::DuplicateSchema(_)
            | Self::DuplicatePlayerProfile(_)
            | Self::DuplicateLlmApi(_)
            | Self::MissingCharacterCover(_)
            | Self::SchemaInUse(_)
            | Self::PlayerProfileInUse(_)
            | Self::CharacterInUse(_)
            | Self::LlmApiInUse(_)
            | Self::StoryResourcesInUse(_)
            | Self::StoryHasSessions(_) => ErrorPayload::new(ErrorCode::Conflict, self.to_string()),
            Self::Registry(error) => match error {
                RegistryError::UnknownApiId(_) => {
                    ErrorPayload::new(ErrorCode::NotFound, self.to_string())
                }
                RegistryError::Llm(_) => {
                    ErrorPayload::new(ErrorCode::InvalidRequest, self.to_string())
                }
            },
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
            ManagerError::MissingSchema(id) => Self::MissingSchema(id),
            ManagerError::MissingCharacter(id) => Self::MissingCharacter(id),
            ManagerError::MissingPlayerProfile(id) => Self::MissingPlayerProfile(id),
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
