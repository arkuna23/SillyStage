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
        "{\"graph\":{\"start_node\":\"start\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{\"flood_gate_open\":{\"value_type\":\"bool\",\"default\":false,\"description\":\"Whether the flood gate has been opened\"}}},\"introduction\":\"The courier arrives at a flooded market gate where a merchant is waiting.\"}",
        Some(json!({
            "graph": {
                "start_node": "start",
                "nodes": []
            },
            "world_state_schema": {
                "fields": {
                    "flood_gate_open": {
                        "value_type": "bool",
                        "default": false,
                        "description": "Whether the flood gate has been opened"
                    }
                }
            },
            "introduction": "The courier arrives at a flooded market gate where a merchant is waiting."
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

    let response = architect
        .generate_graph(ArchitectRequest {
            story_concept: "Test concept",
            planned_story: None,
            world_state_schema: Some(&schema),
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed");

    assert_eq!(
        response.introduction,
        "The courier arrives at a flooded market gate where a merchant is waiting."
    );

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("WORLD_STATE_SCHEMA_SEED"));
    assert!(user_message.content.contains("\"fields\""));
    assert!(user_message.content.contains("\"id\": \"merchant\""));
    assert!(user_message.content.contains("\"state_schema\""));
    assert!(!user_message.content.contains("PLAYER_STATE_SCHEMA"));
    assert!(!user_message.content.contains("\"player_state\""));
    assert!(!user_message.content.contains("Stay in character."));
}

#[tokio::test]
async fn architect_can_generate_schema_without_seed() {
    let llm = MockLlm::with_chat_response(assistant_response(
        "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{\"trust_level\":{\"value_type\":\"int\",\"default\":0,\"description\":\"How much the protagonist trusts the guide\"}}},\"introduction\":\"The courier reaches the flooded dock and must decide whether to trust the guide.\"}",
        Some(json!({
            "graph": {
                "start_node": "dock",
                "nodes": []
            },
            "world_state_schema": {
                "fields": {
                    "trust_level": {
                        "value_type": "int",
                        "default": 0,
                        "description": "How much the protagonist trusts the guide"
                    }
                }
            },
            "introduction": "The courier reaches the flooded dock and must decide whether to trust the guide."
        })),
    ));
    let architect = Architect::new(&llm, "test-model");
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];

    let response = architect
        .generate_graph(ArchitectRequest {
            story_concept: "Test concept",
            planned_story: None,
            world_state_schema: None,
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed without seed");

    assert_eq!(response.graph.start_node, "dock");
    assert!(response.world_state_schema.has_field("trust_level"));
    assert_eq!(
        response.introduction,
        "The courier reaches the flooded dock and must decide whether to trust the guide."
    );

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("WORLD_STATE_SCHEMA_SEED"));
    assert!(user_message.content.contains("null"));
}

#[tokio::test]
async fn architect_prefers_planned_story_when_provided() {
    let llm = MockLlm::with_chat_response(assistant_response(
        "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{}},\"introduction\":\"The courier arrives at the dock.\"}",
        Some(json!({
            "graph": {
                "start_node": "dock",
                "nodes": []
            },
            "world_state_schema": {
                "fields": {}
            },
            "introduction": "The courier arrives at the dock."
        })),
    ));
    let architect = Architect::new(&llm, "test-model");
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];
    let planned_story = "Title:\nFlooded Dock Bargain\n\nOpening Situation:\nThe courier arrives at a flooded dock.";

    let response = architect
        .generate_graph(ArchitectRequest {
            story_concept: "Test concept",
            planned_story: Some(planned_story),
            world_state_schema: None,
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed with planned story");

    assert_eq!(response.introduction, "The courier arrives at the dock.");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("PLANNED_STORY"));
    assert!(user_message.content.contains(planned_story));
}
