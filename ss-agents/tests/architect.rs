mod common;

use std::collections::HashMap;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::architect::{Architect, ArchitectRequest};
use state::schema::{StateFieldSchema, StateValueType, WorldStateSchema};

use common::{MockLlm, assistant_response};

fn sample_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    )])
}

#[tokio::test]
async fn architect_prompt_uses_character_summaries_and_ids() {
    let llm = MockLlm::with_chat_response(assistant_response(
        "{\"start_node\":\"start\",\"nodes\":[]}",
        Some(json!({
            "start_node": "start",
            "nodes": []
        })),
    ));
    let architect = Architect::new(&llm, "test-model");

    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];

    let _ = architect
        .generate_graph(ArchitectRequest {
            story_concept: "Test concept",
            world_state_schema: &schema,
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("\"fields\""));
    assert!(user_message.content.contains("\"id\": \"merchant\""));
    assert!(user_message.content.contains("\"state_schema\""));
    assert!(!user_message.content.contains("PLAYER_STATE_SCHEMA"));
    assert!(!user_message.content.contains("\"player_state\""));
    assert!(!user_message.content.contains("Stay in character."));
}
