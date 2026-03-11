use agents::actor::CharacterCard;
use engine::{EngineTurnResult, RuntimeSnapshot};
use serde::{Deserialize, Serialize};
use state::{PlayerStateSchema, WorldStateSchema};
use story::StoryGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseBody {
    StoryPlanned(StoryPlannedPayload),
    StoryGenerated(StoryGeneratedPayload),
    SessionStarted(SessionStartedPayload),
    TurnCompleted(Box<TurnCompletedPayload>),
    PlayerDescriptionUpdated(PlayerDescriptionUpdatedPayload),
    RuntimeSnapshot(RuntimeSnapshotPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryPlannedPayload {
    pub story_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGeneratedPayload {
    pub graph: StoryGraph,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartedPayload {
    pub snapshot: RuntimeSnapshot,
    pub character_cards: Vec<CharacterCard>,
}

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
