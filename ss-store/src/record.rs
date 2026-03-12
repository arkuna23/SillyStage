use agents::actor::CharacterCard;
use serde::{Deserialize, Serialize};
use state::{PlayerStateSchema, WorldState, WorldStateSchema};
use story::StoryGraph;

use crate::config::SessionEngineConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub story_id: String,
    pub player_description: String,
    pub world_state: WorldState,
    pub turn_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardRecord {
    pub character_id: String,
    pub content: CharacterCard,
    pub cover_file_name: String,
    pub cover_mime_type: String,
    pub cover_bytes: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResourcesRecord {
    pub resource_id: String,
    pub story_concept: String,
    pub character_ids: Vec<String>,
    pub player_state_schema_seed: PlayerStateSchema,
    pub world_state_schema_seed: Option<WorldStateSchema>,
    pub planned_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryRecord {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub graph: StoryGraph,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub display_name: String,
    pub story_id: String,
    pub snapshot: RuntimeSnapshot,
    pub config: SessionEngineConfig,
}
