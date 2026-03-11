use agents::actor::CharacterCard;
use serde::{Deserialize, Serialize};
use state::PlayerStateSchema;
use story::StoryGraph;

use crate::response::StoryGeneratedPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestBody {
    GenerateStoryPlan {
        resources: engine::StoryResources,
    },
    GenerateStoryGraph {
        resources: engine::StoryResources,
    },
    StartSessionFromGeneratedStory {
        resources: engine::StoryResources,
        generated: StoryGeneratedPayload,
        player_description: String,
    },
    StartSessionFromDirectStory {
        story_id: String,
        graph: StoryGraph,
        character_cards: Vec<CharacterCard>,
        player_state_schema: PlayerStateSchema,
        player_description: String,
    },
    RunTurn {
        player_input: String,
    },
    UpdatePlayerDescription {
        player_description: String,
    },
    GetRuntimeSnapshot,
}
