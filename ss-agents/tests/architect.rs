mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::architect::{
    Architect, ArchitectDraftContinueRequest, ArchitectDraftInitRequest, ArchitectRequest,
    GraphSummaryNode,
};
use state::schema::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use story::NarrativeNode;

use common::{
    MockLlm, architect_prompt_profiles, assistant_response, context_entry, prompt_profile,
};

fn joined_user_messages(request: &llm::ChatRequest) -> String {
    request
        .messages
        .iter()
        .filter(|message| matches!(message.role, llm::Role::User))
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

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

fn sample_architect_prompt_profiles() -> ss_agents::ArchitectPromptProfiles {
    architect_prompt_profiles(
        prompt_profile(
            "ROLE:\nArchitect Tone\nFavor compact node descriptions.",
            vec![
                context_entry("story-concept", "STORY_CONCEPT", "story_concept"),
                context_entry("planned-story", "PLANNED_STORY", "planned_story"),
                context_entry(
                    "available-characters",
                    "AVAILABLE_CHARACTERS",
                    "available_characters",
                ),
            ],
            vec![
                context_entry(
                    "world-schema-seed",
                    "WORLD_STATE_SCHEMA_SEED",
                    "world_state_schema_seed",
                ),
                context_entry(
                    "player-schema-seed",
                    "PLAYER_STATE_SCHEMA_SEED",
                    "player_state_schema_seed",
                ),
            ],
        ),
        prompt_profile(
            "ROLE:\nArchitect Draft Init\nEvery schema field object must include \"value_type\".",
            vec![
                context_entry("story-concept", "STORY_CONCEPT", "story_concept"),
                context_entry("planned-story", "PLANNED_STORY", "planned_story"),
                context_entry(
                    "available-characters",
                    "AVAILABLE_CHARACTERS",
                    "available_characters",
                ),
                context_entry(
                    "world-schema-seed",
                    "WORLD_STATE_SCHEMA_SEED",
                    "world_state_schema_seed",
                ),
                context_entry(
                    "player-schema-seed",
                    "PLAYER_STATE_SCHEMA_SEED",
                    "player_state_schema_seed",
                ),
            ],
            vec![
                context_entry("current-section", "CURRENT_SECTION", "current_section"),
                context_entry("section-index", "SECTION_INDEX", "section_index"),
                context_entry("total-sections", "TOTAL_SECTIONS", "total_sections"),
                context_entry(
                    "target-node-count",
                    "TARGET_NODE_COUNT",
                    "target_node_count",
                ),
                context_entry("graph-summary", "GRAPH_SUMMARY", "graph_summary"),
                context_entry(
                    "recent-section-detail",
                    "RECENT_SECTION_DETAIL",
                    "recent_section_detail",
                ),
            ],
        ),
        prompt_profile(
            "ROLE:\nArchitect Draft Continue\nKeep transitions internally consistent.",
            vec![
                context_entry("story-concept", "STORY_CONCEPT", "story_concept"),
                context_entry(
                    "available-characters",
                    "AVAILABLE_CHARACTERS",
                    "available_characters",
                ),
                context_entry("world-schema", "WORLD_STATE_SCHEMA", "world_state_schema"),
                context_entry(
                    "player-schema",
                    "PLAYER_STATE_SCHEMA",
                    "player_state_schema",
                ),
                context_entry(
                    "section-summaries",
                    "SECTION_SUMMARIES",
                    "section_summaries",
                ),
            ],
            vec![
                context_entry("current-section", "CURRENT_SECTION", "current_section"),
                context_entry("section-index", "SECTION_INDEX", "section_index"),
                context_entry("total-sections", "TOTAL_SECTIONS", "total_sections"),
                context_entry(
                    "target-node-count",
                    "TARGET_NODE_COUNT",
                    "target_node_count",
                ),
                context_entry("graph-summary", "GRAPH_SUMMARY", "graph_summary"),
                context_entry(
                    "recent-section-detail",
                    "RECENT_SECTION_DETAIL",
                    "recent_section_detail",
                ),
            ],
        ),
        "ROLE:\nArchitect Repair\nReturn valid JSON only.",
    )
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
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());

    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    let player_state_schema = sample_player_state_schema();
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader trust={{trust}}".to_owned(),
        style: "talkative".to_owned(),
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
            lorebook_base: None,
            lorebook_matched: None,
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
    let user_message = joined_user_messages(request);
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");

    assert!(user_message.contains("WORLD_STATE_SCHEMA_SEED"));
    assert!(user_message.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message.contains("merchant | Old Merchant"));
    assert!(user_message.contains("state_schema"));
    assert!(user_message.contains("trust=0"));
    assert!(user_message.contains("role="));
    assert!(user_message.contains("coins:"));
    assert!(!user_message.contains("Stay in character."));
    assert!(system_message.content.contains("Architect Tone"));
    assert!(
        system_message
            .content
            .contains("Favor compact node descriptions.")
    );
    assert!(!system_message.content.contains("PRESET_PROMPT_ENTRIES"));
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
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
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
            lorebook_base: None,
            lorebook_matched: None,
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
    let user_message = joined_user_messages(request);

    assert!(user_message.contains("WORLD_STATE_SCHEMA_SEED"));
    assert!(user_message.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message.contains("null"));
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
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
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
            lorebook_base: None,
            lorebook_matched: None,
        })
        .await
        .expect("graph generation should succeed with planned story");

    assert_eq!(response.introduction, "The courier arrives at the dock.");
    assert!(response.player_state_schema.fields.is_empty());

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = joined_user_messages(request);

    assert!(user_message.contains("PLANNED_STORY"));
    assert!(user_message.contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message.contains(planned_story));
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
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
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
            lorebook_base: None,
            lorebook_matched: None,
        })
        .await
        .expect("draft continue should succeed");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = joined_user_messages(request);
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");

    assert!(user_message.contains("SECTION_SUMMARIES"));
    assert!(user_message.contains("RECENT_SECTION_DETAIL"));
    assert!(!user_message.contains("PLANNED_STORY"));
    assert!(!user_message.contains("Suggested Beats:"));
    assert!(system_message.content.contains("Architect Draft Continue"));
    assert!(!system_message.content.contains("PLANNED_STORY"));
    assert!(!system_message.content.contains("PLAYER_STATE_SCHEMA_SEED"));
}

#[tokio::test]
async fn architect_draft_init_prompt_requires_value_type_in_schema_fields() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"nodes\":[{\"id\":\"start\",\"title\":\"Gate\",\"scene\":\"The courier reaches the city gate.\",\"goal\":\"Open the story.\",\"characters\":[\"merchant\"],\"transitions\":[],\"on_enter_updates\":[]}],\"transition_patches\":[],\"section_summary\":\"The courier reaches the gate and faces the first obstacle.\",\"start_node\":\"start\",\"world_state_schema\":{\"fields\":{\"gate_open\":{\"value_type\":\"bool\",\"default\":false,\"description\":\"Whether the gate is open\"}}},\"player_state_schema\":{\"fields\":{\"coins\":{\"value_type\":\"int\",\"default\":0,\"description\":\"How many coins the player carries\"}}},\"introduction\":\"The courier arrives at the city gate as the merchant watches.\"}",
        Some(json!({
            "nodes": [{
                "id": "start",
                "title": "Gate",
                "scene": "The courier reaches the city gate.",
                "goal": "Open the story.",
                "characters": ["merchant"],
                "transitions": [],
                "on_enter_updates": []
            }],
            "transition_patches": [],
            "section_summary": "The courier reaches the gate and faces the first obstacle.",
            "start_node": "start",
            "world_state_schema": {
                "fields": {
                    "gate_open": {
                        "value_type": "bool",
                        "default": false,
                        "description": "Whether the gate is open"
                    }
                }
            },
            "player_state_schema": {
                "fields": {
                    "coins": {
                        "value_type": "int",
                        "default": 0,
                        "description": "How many coins the player carries"
                    }
                }
            },
            "introduction": "The courier arrives at the city gate as the merchant watches."
        })),
    )));
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];

    architect
        .start_draft(ArchitectDraftInitRequest {
            story_concept: "A courier tries to enter a city.",
            planned_story: "The courier reaches the gate and must negotiate passage.",
            current_section: "Opening at the gate.",
            section_index: 0,
            total_sections: 2,
            graph_summary: &[],
            recent_nodes: &[],
            target_node_count: 3,
            world_state_schema: None,
            player_state_schema: None,
            available_characters: &available_characters,
            lorebook_base: None,
            lorebook_matched: None,
        })
        .await
        .expect("draft init should succeed");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");

    assert!(
        system_message
            .content
            .contains("Every schema field object must include \"value_type\"")
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
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
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
            lorebook_base: None,
            lorebook_matched: None,
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
    assert!(
        repair_user_message
            .content
            .contains("\"value_type\": \"bool\"")
    );
    assert!(
        repair_user_message
            .content
            .contains("\"value_type\": \"int\"")
    );
}

#[tokio::test]
async fn architect_draft_continue_repairs_missing_future_transition_targets() {
    let llm = Arc::new(MockLlm::with_chat_responses(vec![
        Ok(assistant_response(
            "{\"nodes\":[{\"id\":\"node-12\",\"title\":\"Dock Choice\",\"scene\":\"The courier reaches a choice at the dock.\",\"goal\":\"Choose a path.\",\"characters\":[\"merchant\"],\"transitions\":[{\"to\":\"node-13\"}],\"on_enter_updates\":[]}],\"transition_patches\":[],\"section_summary\":\"The courier faces a branching choice.\"}",
            Some(json!({
                "nodes": [{
                    "id": "node-12",
                    "title": "Dock Choice",
                    "scene": "The courier reaches a choice at the dock.",
                    "goal": "Choose a path.",
                    "characters": ["merchant"],
                    "transitions": [{
                        "to": "node-13"
                    }],
                    "on_enter_updates": []
                }],
                "transition_patches": [],
                "section_summary": "The courier faces a branching choice."
            })),
        )),
        Ok(assistant_response(
            "{\"nodes\":[{\"id\":\"node-12\",\"title\":\"Dock Choice\",\"scene\":\"The courier reaches a choice at the dock.\",\"goal\":\"Choose a path.\",\"characters\":[\"merchant\"],\"transitions\":[{\"to\":\"start\"}],\"on_enter_updates\":[]}],\"transition_patches\":[],\"section_summary\":\"The courier faces a branching choice.\"}",
            Some(json!({
                "nodes": [{
                    "id": "node-12",
                    "title": "Dock Choice",
                    "scene": "The courier reaches a choice at the dock.",
                    "goal": "Choose a path.",
                    "characters": ["merchant"],
                    "transitions": [{
                        "to": "start"
                    }],
                    "on_enter_updates": []
                }],
                "transition_patches": [],
                "section_summary": "The courier faces a branching choice."
            })),
        )),
    ]));
    let architect = Architect::new(llm.clone(), "test-model")
        .with_prompt_profiles(sample_architect_prompt_profiles());
    let available_characters = vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        state_schema: sample_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }];
    let mut world_schema = WorldStateSchema::new();
    world_schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    let player_state_schema = sample_player_state_schema();

    let chunk = architect
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
                transition_targets: vec!["node-12".to_owned()],
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
            lorebook_base: None,
            lorebook_matched: None,
        })
        .await
        .expect("draft continue should be repaired");

    assert_eq!(chunk.nodes[0].transitions[0].to, "start");

    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 2, "initial request plus repair request");
    let repair_request = requests.last().expect("repair request should exist");
    let repair_user_message = repair_request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("repair user message should exist");
    assert!(repair_user_message.content.contains("node-13"));
    assert!(
        repair_user_message.content.contains(
            "allowed targets are existing graph nodes [start] or returned nodes [node-12]"
        )
    );
}
