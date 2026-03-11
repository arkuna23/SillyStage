mod common;

use std::collections::HashMap;

use futures_util::StreamExt;
use llm::ChatChunk;
use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::director::NarratorPurpose;
use ss_agents::narrator::{Narrator, NarratorRequest, NarratorStreamEvent};
use state::{
    ActorMemoryEntry, ActorMemoryKind, PlayerStateSchema, StateFieldSchema, StateValueType,
    WorldState,
};
use story::NarrativeNode;

use common::MockLlm;

fn merchant_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    )])
}

fn guide_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "knows_safe_route".to_owned(),
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    )])
}

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![
        CharacterCard {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual, slightly cunning".to_owned(),
            tendencies: vec![
                "likes profitable deals".to_owned(),
                "avoids danger".to_owned(),
                "tries to maintain good relationships".to_owned(),
            ],
            state_schema: merchant_state_schema(),
            system_prompt: "Stay in character.".to_owned(),
        },
        CharacterCard {
            id: "guide".to_owned(),
            name: "Yuki".to_owned(),
            personality: "calm local guide".to_owned(),
            style: "measured, clear".to_owned(),
            tendencies: vec!["protects civilians".to_owned()],
            state_schema: guide_state_schema(),
            system_prompt: "Stay observant.".to_owned(),
        },
    ]
}

fn sample_world_state() -> WorldState {
    let mut world_state = WorldState::new("dock");
    world_state.set_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]);
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_player_state("coins", json!(12));
    world_state.set_character_state("merchant", "trust", json!(2));
    world_state.push_player_input_shared_memory("Can you open the gate before the tide turns?", 8);
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "The gate is still jammed.".to_owned(),
        },
        8,
    );
    world_state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "I should not reveal the shortcut yet.".to_owned(),
        },
        8,
    );
    world_state
}

fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

fn sample_scene_node() -> NarrativeNode {
    NarrativeNode::new(
        "dock",
        "Flooded Dock",
        "A flooded dock at dusk, with loose planks rocking over dark water.",
        "Decide whether to trust the guide.",
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![],
        vec![],
    )
}

fn scene_request<'a>(
    purpose: NarratorPurpose,
    current_node: &'a NarrativeNode,
    character_cards: &'a [CharacterCard],
    player_state_schema: &'a PlayerStateSchema,
    world_state: &'a WorldState,
) -> NarratorRequest<'a> {
    NarratorRequest {
        purpose,
        previous_node: None,
        current_node,
        character_cards,
        player_description: "A cautious courier trying to get medicine through the flooded district.",
        player_state_schema,
        world_state,
    }
}

#[tokio::test]
async fn narrate_stream_emits_text_deltas_and_done() {
    let llm = MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "Cold water slapped".to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta: " against the dock posts.".to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta: String::new(),
            model: Some("test-model".to_owned()),
            finish_reason: Some("stop".to_owned()),
            done: true,
            usage: None,
        }),
    ]);
    let narrator = Narrator::new(&llm, "test-model").expect("narrator should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let current_node = sample_scene_node();

    let mut stream = narrator
        .narrate_stream(scene_request(
            NarratorPurpose::DescribeScene,
            &current_node,
            &character_cards,
            &player_state_schema,
            &world_state,
        ))
        .await
        .expect("stream should start");

    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        NarratorStreamEvent::TextDelta {
            delta: "Cold water slapped".to_owned()
        }
    );
    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        NarratorStreamEvent::TextDelta {
            delta: " against the dock posts.".to_owned()
        }
    );

    let NarratorStreamEvent::Done { response } = stream.next().await.expect("event").expect("ok")
    else {
        panic!("expected final done event");
    };

    assert_eq!(response.text, "Cold water slapped against the dock posts.");
    assert_eq!(response.raw_output, response.text);
    assert!(stream.next().await.is_none());
}

#[tokio::test]
async fn describe_transition_requires_previous_node() {
    let llm = MockLlm::with_stream_chunks(vec![]);
    let narrator = Narrator::new(&llm, "test-model").expect("narrator should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let current_node = sample_scene_node();

    let error = match narrator
        .narrate_stream(scene_request(
            NarratorPurpose::DescribeTransition,
            &current_node,
            &character_cards,
            &player_state_schema,
            &world_state,
        ))
        .await
    {
        Ok(_) => panic!("transition narration should require previous node"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("previous_node"));
}

#[tokio::test]
async fn narrator_prompt_includes_shared_history_but_not_private_memory() {
    let llm = MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "The dock rocked in the dark.".to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta: String::new(),
            model: Some("test-model".to_owned()),
            finish_reason: Some("stop".to_owned()),
            done: true,
            usage: None,
        }),
    ]);
    let narrator = Narrator::new(&llm, "test-model").expect("narrator should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let previous_node = NarrativeNode::new(
        "market",
        "Night Market",
        "A lantern-lit market lane.",
        "Reach the dock.",
        vec!["merchant".to_owned()],
        vec![],
        vec![],
    );
    let current_node = sample_scene_node();

    let request = NarratorRequest {
        purpose: NarratorPurpose::DescribeTransition,
        previous_node: Some(&previous_node),
        current_node: &current_node,
        character_cards: &character_cards,
        player_description: "A cautious courier trying to get medicine through the flooded district.",
        player_state_schema: &player_state_schema,
        world_state: &world_state,
    };

    let _ = narrator
        .narrate(request)
        .await
        .expect("narration should succeed");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let user_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::User))
        .expect("user message should exist");

    assert!(user_message.content.contains("CURRENT_NODE"));
    assert!(user_message.content.contains("PREVIOUS_NODE"));
    assert!(user_message.content.contains("\"actor_shared_history\""));
    assert!(user_message.content.contains("PLAYER_STATE_SCHEMA"));
    assert!(user_message.content.contains("PLAYER_DESCRIPTION"));
    assert!(
        user_message
            .content
            .contains("A cautious courier trying to get medicine through the flooded district.")
    );
    assert!(user_message.content.contains("\"player_state\""));
    assert!(user_message.content.contains("\"coins\": 12"));
    assert!(user_message.content.contains("\"state_schema\""));
    assert!(
        user_message
            .content
            .contains("Can you open the gate before the tide turns?")
    );
    assert!(user_message.content.contains("The gate is still jammed."));
    assert!(!user_message.content.contains("\"actor_private_memory\""));
    assert!(
        !user_message
            .content
            .contains("I should not reveal the shortcut yet.")
    );
}
