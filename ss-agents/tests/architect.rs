mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::architect::{
    Architect, ArchitectDraftContinueRequest, ArchitectRequest, GraphSummaryNode,
};
use state::schema::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use story::NarrativeNode;

use common::{MockLlm, assistant_response};

fn sample_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    )])
}

fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

#[tokio::test]
async fn architect_prompt_uses_character_summaries_and_ids() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
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
    )));
    let architect = Architect::new(llm.clone(), "test-model");

    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    let player_state_schema = sample_player_state_schema();
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
            player_state_schema: Some(&player_state_schema),
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed");

    assert_eq!(
        response.introduction,
        "The courier arrives at a flooded market gate where a merchant is waiting."
    );
    assert!(response.player_state_schema.has_field("coins"));

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");

    assert!(user_message.content.contains("WORLD_STATE_SCHEMA_SEED"));
    assert!(user_message.content.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message.content.contains("\"id\": \"merchant\""));
    assert!(user_message.content.contains("\"state_schema_keys\""));
    assert!(user_message.content.contains("\"role_summary\""));
    assert!(user_message.content.contains("\"coins\""));
    assert!(!user_message.content.contains("Stay in character."));
    assert!(system_message.content.contains("\"type\": \"SetState\""));
    assert!(
        system_message
            .content
            .contains("\"scope\": \"global_or_player_or_character\"")
    );
    assert!(
        system_message
            .content
            .contains("Use player_state_schema keys for player-scoped conditions")
    );
    assert!(
        system_message
            .content
            .contains("\"type\": \"SetCharacterState\"")
    );
}

#[tokio::test]
async fn architect_can_generate_schema_without_seed() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{\"trust_level\":{\"value_type\":\"int\",\"default\":0,\"description\":\"How much the protagonist trusts the guide\"}}},\"player_state_schema\":{\"fields\":{\"reputation\":{\"value_type\":\"int\",\"default\":0,\"description\":\"How much the district trusts the player\"}}},\"introduction\":\"The courier reaches the flooded dock and must decide whether to trust the guide.\"}",
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
            "player_state_schema": {
                "fields": {
                    "reputation": {
                        "value_type": "int",
                        "default": 0,
                        "description": "How much the district trusts the player"
                    }
                }
            },
            "introduction": "The courier reaches the flooded dock and must decide whether to trust the guide."
        })),
    )));
    let architect = Architect::new(llm.clone(), "test-model");
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
            player_state_schema: None,
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed without seed");

    assert_eq!(response.graph.start_node, "dock");
    assert!(response.world_state_schema.has_field("trust_level"));
    assert!(response.player_state_schema.has_field("reputation"));
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
    assert!(user_message.content.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message.content.contains("null"));
}

#[tokio::test]
async fn architect_prefers_planned_story_when_provided() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
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
    )));
    let architect = Architect::new(llm.clone(), "test-model");
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
            player_state_schema: None,
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should succeed with planned story");

    assert_eq!(response.introduction, "The courier arrives at the dock.");
    assert!(response.player_state_schema.fields.is_empty());

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("PLANNED_STORY"));
    assert!(user_message.content.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message.content.contains(planned_story));
}

#[tokio::test]
async fn architect_draft_continue_prompt_uses_section_summaries_and_omits_full_planned_story() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"nodes\":[{\"id\":\"dock_choice\",\"title\":\"Dock Choice\",\"scene\":\"The courier weighs two offers at the dock.\",\"goal\":\"Offer the next branch.\",\"characters\":[\"merchant\"],\"transitions\":[],\"on_enter_updates\":[]}],\"transition_patches\":[],\"section_summary\":\"The courier faces the first real tradeoff at the dock.\"}",
        Some(json!({
            "nodes": [{
                "id": "dock_choice",
                "title": "Dock Choice",
                "scene": "The courier weighs two offers at the dock.",
                "goal": "Offer the next branch.",
                "characters": ["merchant"],
                "transitions": [],
                "on_enter_updates": []
            }],
            "transition_patches": [],
            "section_summary": "The courier faces the first real tradeoff at the dock."
        })),
    )));
    let architect = Architect::new(llm.clone(), "test-model");
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];
    let mut world_schema = WorldStateSchema::new();
    world_schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    let player_state_schema = sample_player_state_schema();

    architect
        .continue_draft(ArchitectDraftContinueRequest {
            story_concept: "A courier must escape a flooded market district.",
            current_section: "The courier reaches a fork between the flooded dock and the watchtower.",
            section_index: 1,
            total_sections: 3,
            section_summaries: &[
                "The courier entered the district and found the dock half-submerged.".to_owned(),
            ],
            graph_summary: &[GraphSummaryNode {
                id: "start".to_owned(),
                title: "Flooded Gate".to_owned(),
                scene_summary: "The courier arrives at the gate.".to_owned(),
                goal: "Open the story.".to_owned(),
                characters: vec!["merchant".to_owned()],
                transition_targets: vec!["dock_choice".to_owned()],
            }],
            recent_nodes: &[NarrativeNode {
                id: "start".to_owned(),
                title: "Flooded Gate".to_owned(),
                scene: "The courier arrives at the flooded gate while the merchant waits nearby.".to_owned(),
                goal: "Open the story.".to_owned(),
                characters: vec!["merchant".to_owned()],
                transitions: vec![],
                on_enter_updates: vec![],
            }],
            target_node_count: 3,
            world_state_schema: &world_schema,
            player_state_schema: &player_state_schema,
            available_characters: &available_characters,
        })
        .await
        .expect("draft continue should succeed");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");

    assert!(user_message.content.contains("SECTION_SUMMARIES"));
    assert!(user_message.content.contains("RECENT_SECTION_DETAIL"));
    assert!(!user_message.content.contains("PLANNED_STORY"));
    assert!(!user_message.content.contains("Suggested Beats:"));
    assert!(!system_message.content.contains("PLANNED_STORY"));
    assert!(!system_message.content.contains("also return introduction"));
    assert!(!system_message.content.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(
        system_message
            .content
            .contains("\"scope\": \"global_or_player_or_character\"")
    );
    assert!(
        system_message
            .content
            .contains("\"type\": \"SetPlayerState\"")
    );
}

#[tokio::test]
async fn architect_attempts_repair_after_invalid_json_output() {
    let llm = Arc::new(MockLlm::with_chat_responses(vec![
        Err(llm::LlmError::StructuredOutputParse {
            message: "expected value".to_owned(),
            raw_content: "not valid json".to_owned(),
        }),
        Ok(assistant_response(
            "{\"graph\":{\"start_node\":\"start\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{}},\"player_state_schema\":{\"fields\":{}},\"introduction\":\"A repaired introduction.\"}",
            Some(json!({
                "graph": {
                    "start_node": "start",
                    "nodes": []
                },
                "world_state_schema": {
                    "fields": {}
                },
                "player_state_schema": {
                    "fields": {}
                },
                "introduction": "A repaired introduction."
            })),
        )),
    ]));
    let architect = Architect::new(llm.clone(), "test-model");
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
            player_state_schema: None,
            available_characters: &available_characters,
        })
        .await
        .expect("graph generation should be repaired");

    assert_eq!(response.introduction, "A repaired introduction.");

    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 2, "initial request plus repair request");
    let repair_request = requests.last().expect("repair request should exist");
    let repair_user_message = repair_request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("repair user message should exist");
    assert!(repair_user_message.content.contains("RAW_OUTPUT"));
    assert!(repair_user_message.content.contains("not valid json"));
}
