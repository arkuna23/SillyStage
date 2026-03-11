use std::collections::HashMap;

use agents::actor::CharacterCard;
use serde_json::json;
use ss_protocol::{RequestBody, StoryGeneratedPayload};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use story::{NarrativeNode, StoryGraph};

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative".to_owned(),
        tendencies: vec!["likes profitable deals".to_owned()],
        state_schema: HashMap::new(),
        system_prompt: "Stay in character.".to_owned(),
    }]
}

fn sample_story_resources() -> engine::StoryResources {
    let mut player_state_schema = PlayerStateSchema::new();
    player_state_schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );

    engine::StoryResources::new(
        "demo_story",
        "A flooded harbor story.",
        sample_character_cards(),
        player_state_schema,
    )
    .expect("story resources should build")
}

fn sample_generated_story() -> StoryGeneratedPayload {
    let mut world_state_schema = WorldStateSchema::new();
    world_state_schema.insert_field(
        "gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );

    let mut player_state_schema = PlayerStateSchema::new();
    player_state_schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );

    StoryGeneratedPayload {
        graph: StoryGraph::new(
            "dock",
            vec![NarrativeNode::new(
                "dock",
                "Flooded Dock",
                "A flooded dock at dusk.",
                "Convince the merchant to help.",
                vec!["merchant".to_owned()],
                vec![],
                vec![],
            )],
        ),
        world_state_schema,
        player_state_schema,
        introduction: "The courier reaches the flooded dock.".to_owned(),
    }
}

#[test]
fn start_session_from_generated_story_round_trips() {
    let request = RequestBody::StartSessionFromGeneratedStory {
        resources: sample_story_resources(),
        generated: sample_generated_story(),
        player_description: "A stubborn courier carrying medicine.".to_owned(),
    };

    let json = serde_json::to_string_pretty(&request).expect("request should serialize");
    let round_trip: RequestBody = serde_json::from_str(&json).expect("request should deserialize");

    let RequestBody::StartSessionFromGeneratedStory {
        generated,
        player_description,
        ..
    } = round_trip
    else {
        panic!("expected generated story session request");
    };

    assert_eq!(generated.graph.start_node, "dock");
    assert!(generated.player_state_schema.has_field("coins"));
    assert_eq!(player_description, "A stubborn courier carrying medicine.");
}

#[test]
fn direct_story_and_runtime_requests_use_stable_tags() {
    let request = RequestBody::StartSessionFromDirectStory {
        story_id: "direct_story".to_owned(),
        graph: sample_generated_story().graph,
        character_cards: sample_character_cards(),
        player_state_schema: sample_story_resources().player_state_schema().clone(),
        player_description: "A disguised courier posing as a dock clerk.".to_owned(),
    };
    let json = serde_json::to_string_pretty(&request).expect("request should serialize");
    assert!(json.contains("\"type\": \"start_session_from_direct_story\""));

    let run_turn = RequestBody::RunTurn {
        player_input: "Open the gate.".to_owned(),
    };
    let run_turn_json =
        serde_json::to_string_pretty(&run_turn).expect("run_turn request should serialize");
    assert!(run_turn_json.contains("\"type\": \"run_turn\""));

    let get_snapshot = RequestBody::GetRuntimeSnapshot;
    let get_snapshot_json =
        serde_json::to_string_pretty(&get_snapshot).expect("snapshot request should serialize");
    assert!(get_snapshot_json.contains("\"type\": \"get_runtime_snapshot\""));
}
