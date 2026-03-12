use engine::{EngineTurnResult, RuntimeSnapshot};
use serde::{Deserialize, Serialize};
use state::{PlayerStateSchema, WorldStateSchema};
use story::StoryGraph;

use crate::character::CharacterCardSummaryPayload;
use crate::config::{GlobalConfigPayload, SessionConfigPayload};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseResult {
    UploadInitialized(UploadInitializedPayload),
    UploadChunkAccepted(UploadChunkAcceptedPayload),
    CharacterCardUploaded(CharacterCardUploadedPayload),
    StoryResourcesCreated(Box<StoryResourcesPayload>),
    StoryResourcesUpdated(Box<StoryResourcesPayload>),
    StoryPlanned(StoryPlannedPayload),
    StoryGenerated(Box<StoryGeneratedPayload>),
    SessionStarted(Box<SessionStartedPayload>),
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
pub struct StoryResourcesPayload {
    pub resource_id: String,
    pub story_concept: String,
    pub character_ids: Vec<String>,
    pub player_state_schema_seed: PlayerStateSchema,
    pub world_state_schema_seed: Option<WorldStateSchema>,
    pub planned_story: Option<String>,
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
    pub graph: StoryGraph,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartedPayload {
    pub snapshot: RuntimeSnapshot,
    pub character_summaries: Vec<CharacterCardSummaryPayload>,
    pub config: SessionConfigPayload,
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
