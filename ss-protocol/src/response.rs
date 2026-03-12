use engine::{EngineTurnResult, RuntimeSnapshot};
use serde::{Deserialize, Serialize};
use state::{PlayerStateSchema, WorldStateSchema};
use story::StoryGraph;

use crate::character::{CharacterCardContent, CharacterCardSummaryPayload, CharacterCoverMimeType};
use crate::config::{GlobalConfigPayload, SessionConfigPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseResult {
    UploadInitialized(UploadInitializedPayload),
    UploadChunkAccepted(UploadChunkAcceptedPayload),
    CharacterCardUploaded(CharacterCardUploadedPayload),
    Character(Box<CharacterDetailPayload>),
    CharactersListed(CharactersListedPayload),
    CharacterDeleted(CharacterDeletedPayload),
    StoryResourcesCreated(Box<StoryResourcesPayload>),
    StoryResources(Box<StoryResourcesPayload>),
    StoryResourcesListed(StoryResourcesListedPayload),
    StoryResourcesUpdated(Box<StoryResourcesPayload>),
    StoryResourcesDeleted(StoryResourcesDeletedPayload),
    StoryPlanned(StoryPlannedPayload),
    StoryGenerated(Box<StoryGeneratedPayload>),
    Story(Box<StoryDetailPayload>),
    StoriesListed(StoriesListedPayload),
    StoryDeleted(StoryDeletedPayload),
    SessionStarted(Box<SessionStartedPayload>),
    Session(Box<SessionDetailPayload>),
    SessionsListed(SessionsListedPayload),
    SessionDeleted(SessionDeletedPayload),
    GlobalConfig(GlobalConfigPayload),
    SessionConfig(SessionConfigPayload),
    TurnStreamAccepted(TurnStreamAcceptedPayload),
    TurnCompleted(Box<TurnCompletedPayload>),
    PlayerDescriptionUpdated(Box<PlayerDescriptionUpdatedPayload>),
    RuntimeSnapshot(Box<RuntimeSnapshotPayload>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadInitializedPayload {
    pub upload_id: String,
    pub chunk_size_hint: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadChunkAcceptedPayload {
    pub upload_id: String,
    pub received_chunk_index: u64,
    pub received_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterCardUploadedPayload {
    pub character_id: String,
    pub character_summary: CharacterCardSummaryPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDetailPayload {
    pub character_id: String,
    pub content: CharacterCardContent,
    pub cover_file_name: String,
    pub cover_mime_type: CharacterCoverMimeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharactersListedPayload {
    pub characters: Vec<CharacterCardSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterDeletedPayload {
    pub character_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResourcesPayload {
    pub resource_id: String,
    pub story_concept: String,
    pub character_ids: Vec<String>,
    pub player_state_schema_seed: PlayerStateSchema,
    pub world_state_schema_seed: Option<WorldStateSchema>,
    pub planned_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResourcesListedPayload {
    pub resources: Vec<StoryResourcesPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryResourcesDeletedPayload {
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryPlannedPayload {
    pub resource_id: String,
    pub story_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGeneratedPayload {
    pub resource_id: String,
    pub story_id: String,
    pub display_name: String,
    pub graph: StoryGraph,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySummaryPayload {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryDetailPayload {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub graph: StoryGraph,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoriesListedPayload {
    pub stories: Vec<StorySummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryDeletedPayload {
    pub story_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartedPayload {
    pub story_id: String,
    pub display_name: String,
    pub snapshot: RuntimeSnapshot,
    pub character_summaries: Vec<CharacterCardSummaryPayload>,
    pub config: SessionConfigPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryPayload {
    pub session_id: String,
    pub story_id: String,
    pub display_name: String,
    pub turn_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailPayload {
    pub session_id: String,
    pub story_id: String,
    pub display_name: String,
    pub snapshot: RuntimeSnapshot,
    pub config: SessionConfigPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsListedPayload {
    pub sessions: Vec<SessionSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionDeletedPayload {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnStreamAcceptedPayload {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnCompletedPayload {
    pub result: EngineTurnResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDescriptionUpdatedPayload {
    pub snapshot: RuntimeSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshotPayload {
    pub snapshot: RuntimeSnapshot,
}
