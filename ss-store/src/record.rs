use serde::{Deserialize, Serialize};
use state::{StateFieldSchema, WorldState};
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
pub struct SchemaRecord {
    pub schema_id: String,
    pub display_name: String,
    pub tags: Vec<String>,
    pub fields: std::collections::HashMap<String, StateFieldSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfileRecord {
    pub player_profile_id: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardDefinition {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub style: String,
    pub tendencies: Vec<String>,
    pub schema_id: String,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardRecord {
    pub character_id: String,
    pub content: CharacterCardDefinition,
    pub cover_file_name: Option<String>,
    pub cover_mime_type: Option<String>,
    pub cover_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmApiRecord {
    pub api_id: String,
    pub provider: crate::config::LlmProvider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResourcesRecord {
    pub resource_id: String,
    pub story_concept: String,
    pub character_ids: Vec<String>,
    pub player_schema_id_seed: Option<String>,
    pub world_schema_id_seed: Option<String>,
    pub planned_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryRecord {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub graph: StoryGraph,
    pub world_schema_id: String,
    pub player_schema_id: String,
    pub introduction: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub session_id: String,
    pub display_name: String,
    pub story_id: String,
    pub player_profile_id: Option<String>,
    pub player_schema_id: String,
    pub snapshot: RuntimeSnapshot,
    pub config: SessionEngineConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at_ms: Option<u64>,
}
