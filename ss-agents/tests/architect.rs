mod common;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::architect::{Architect, ArchitectRequest};
use state::schema::{StateFieldSchema, StateValueType, WorldStateSchema};

use common::{assistant_response, MockLlm};

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
    schema.insert_character_field(
        "trust",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );

    let _ = architect
        .generate_graph(ArchitectRequest {
            story_concept: "Test concept".to_owned(),
            world_state_schema: schema,
            available_characters: vec![CharacterCard {
                id: "merchant".to_owned(),
                name: "Old Merchant".to_owned(),
                personality: "greedy but friendly trader".to_owned(),
                style: "talkative".to_owned(),
                tendencies: vec!["likes profitable deals".to_owned()],
                system_prompt: "Stay in character.".to_owned(),
            }],
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
    assert!(user_message.content.contains("\"character_fields\""));
    assert!(user_message.content.contains("\"id\": \"merchant\""));
    assert!(!user_message.content.contains("Stay in character."));
}
