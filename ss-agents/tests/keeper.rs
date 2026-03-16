mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::{ActorResponse, ActorSegment, ActorSegmentKind, CharacterCard};
use ss_agents::director::{ActorPurpose, NarratorPurpose};
use ss_agents::keeper::{Keeper, KeeperActorSegmentKind, KeeperBeat, KeeperPhase, KeeperRequest};
use state::{
    ActorMemoryEntry, ActorMemoryKind, PlayerStateSchema, StateFieldSchema, StateOp,
    StateValueType, WorldState,
};
use story::NarrativeNode;

use common::{MockLlm, assistant_response};

fn joined_user_messages(request: &llm::ChatRequest) -> String {
    request
        .messages
        .iter()
        .filter(|message| matches!(message.role, llm::Role::User))
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

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
            style: "talkative, casual".to_owned(),
            state_schema: merchant_state_schema(),
            system_prompt: "Stay in character.".to_owned(),
        },
        CharacterCard {
            id: "guide".to_owned(),
            name: "Yuki".to_owned(),
            personality: "calm local guide".to_owned(),
            style: "measured".to_owned(),
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
    world_state
        .push_player_input_shared_memory("I agree to follow the guide toward the canal gate.", 8);
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "We'll reach the canal gate before the tide turns.".to_owned(),
        },
        8,
    );
    world_state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "I should keep the safer route to myself.".to_owned(),
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

fn previous_node() -> NarrativeNode {
    NarrativeNode::new(
        "market",
        "Night Market",
        "A lantern-lit market lane.",
        "Reach the dock.",
        vec!["merchant".to_owned()],
        vec![],
        vec![],
    )
}

fn current_node() -> NarrativeNode {
    NarrativeNode::new(
        "dock",
        "Flooded Dock",
        "A flooded dock at dusk.",
        "Decide whether to trust the guide.",
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![],
        vec![],
    )
}

fn completed_beats() -> Vec<KeeperBeat> {
    vec![
        KeeperBeat::Narrator {
            purpose: NarratorPurpose::DescribeScene,
            text: "Cold water slaps against the dock posts as the two figures weigh their options."
                .to_owned(),
        },
        KeeperBeat::Actor {
            speaker_id: "merchant".to_owned(),
            purpose: ActorPurpose::AdvanceGoal,
            visible_segments: vec![
                ss_agents::keeper::KeeperActorSegment {
                    kind: KeeperActorSegmentKind::Dialogue,
                    text: "We still have time if we move now.".to_owned(),
                },
                ss_agents::keeper::KeeperActorSegment {
                    kind: KeeperActorSegmentKind::Action,
                    text: "Haru lifts the lantern and steps onto the slick planks.".to_owned(),
                },
            ],
        },
    ]
}

fn sample_request<'a>(
    previous_node: Option<&'a NarrativeNode>,
    current_node: &'a NarrativeNode,
    character_cards: &'a [CharacterCard],
    player_state_schema: &'a PlayerStateSchema,
    world_state: &'a WorldState,
    completed_beats: &'a [KeeperBeat],
) -> KeeperRequest<'a> {
    KeeperRequest {
        phase: KeeperPhase::AfterTurnOutputs,
        player_input: "I agree to follow the guide toward the canal gate.",
        player_name: Some("Courier"),
        player_description: "A cautious courier escorting medicine through the flooded district.",
        previous_node,
        current_node,
        character_cards,
        current_cast_ids: &current_node.characters,
        player_state_schema,
        world_state,
        completed_beats,
    }
}

#[tokio::test]
async fn keep_parses_json_state_update() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"ops\":[{\"type\":\"SetState\",\"key\":\"route_committed\",\"value\":true},{\"type\":\"SetPlayerState\",\"key\":\"coins\",\"value\":9},{\"type\":\"RemoveActiveCharacter\",\"character\":\"guide\"},{\"type\":\"SetCharacterState\",\"character\":\"merchant\",\"key\":\"trust\",\"value\":3}]}",
        Some(json!({
            "ops": [
                {
                    "type": "SetState",
                    "key": "route_committed",
                    "value": true
                },
                {
                    "type": "SetPlayerState",
                    "key": "coins",
                    "value": 9
                },
                {
                    "type": "RemoveActiveCharacter",
                    "character": "guide"
                },
                {
                    "type": "SetCharacterState",
                    "character": "merchant",
                    "key": "trust",
                    "value": 3
                }
            ]
        })),
    )));
    let keeper = Keeper::new(llm.clone(), "test-model").expect("keeper should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let previous_node = previous_node();
    let current_node = current_node();
    let completed_beats = completed_beats();

    let response = keeper
        .keep(sample_request(
            Some(&previous_node),
            &current_node,
            &character_cards,
            &player_state_schema,
            &world_state,
            &completed_beats,
        ))
        .await
        .expect("keeper should succeed");

    assert_eq!(response.update.ops.len(), 4);
    assert!(matches!(response.update.ops[0], StateOp::SetState { .. }));
    assert!(matches!(
        response.update.ops[1],
        StateOp::SetPlayerState { .. }
    ));
    assert!(matches!(
        response.update.ops[2],
        StateOp::RemoveActiveCharacter { .. }
    ));
    assert!(matches!(
        response.update.ops[3],
        StateOp::SetCharacterState { .. }
    ));
}

#[tokio::test]
async fn keeper_prompt_includes_shared_history_but_not_private_memory() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"ops\":[]}",
        Some(json!({
            "ops": []
        })),
    )));
    let keeper = Keeper::new(llm.clone(), "test-model").expect("keeper should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let previous_node = previous_node();
    let current_node = current_node();
    let completed_beats = completed_beats();

    let _ = keeper
        .keep(sample_request(
            Some(&previous_node),
            &current_node,
            &character_cards,
            &player_state_schema,
            &world_state,
            &completed_beats,
        ))
        .await
        .expect("keep should work");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    let system_message = request
        .messages
        .iter()
        .find(|message| matches!(message.role, llm::Role::System))
        .expect("system message should exist");
    let user_message = joined_user_messages(request);

    assert!(user_message.contains("KEEPER_PHASE"));
    assert!(user_message.contains("PLAYER_INPUT"));
    assert!(!user_message.contains("PLAYER_NAME"));
    assert!(user_message.contains("PLAYER_DESCRIPTION"));
    assert!(user_message.contains("COMPLETED_BEATS"));
    assert!(user_message.contains("shared_history"));
    assert!(user_message.contains("PLAYER_STATE_SCHEMA"));
    assert!(user_message.contains("player_state"));
    assert!(user_message.contains("coins=12"));
    assert!(user_message.contains("coins:"));
    assert!(!user_message.contains("StateUpdate schema"));
    assert!(
        system_message
            .content
            .contains("\"type\": \"SetCharacterState\"")
    );
    assert!(
        system_message
            .content
            .contains("\"character\": \"merchant\"")
    );
    assert!(
        system_message
            .content
            .contains("\"type\": \"RemoveCharacterState\"")
    );
    assert!(user_message.contains("We'll reach the canal gate before the tide turns."));
    assert!(user_message.contains("I agree to follow the guide toward the canal gate."));
    assert!(
        user_message
            .contains("A cautious courier escorting medicine through the flooded district.")
    );
    assert!(!user_message.contains("actor_private_memory"));
    assert!(!user_message.contains("I should keep the safer route to myself."));
}

#[tokio::test]
async fn keeper_rejects_character_state_without_character_field() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"ops\":[{\"type\":\"SetCharacterState\",\"key\":\"trust\",\"value\":3}]}",
        Some(json!({
            "ops": [
                {
                    "type": "SetCharacterState",
                    "key": "trust",
                    "value": 3
                }
            ]
        })),
    )));
    let keeper = Keeper::new(llm.clone(), "test-model").expect("keeper should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let previous_node = previous_node();
    let current_node = current_node();
    let completed_beats = completed_beats();

    let error = keeper
        .keep(sample_request(
            Some(&previous_node),
            &current_node,
            &character_cards,
            &player_state_schema,
            &world_state,
            &completed_beats,
        ))
        .await
        .expect_err("keeper should reject malformed character-scoped ops");

    assert!(error.to_string().contains("missing field `character`"));
}

#[tokio::test]
async fn keeper_rejects_disallowed_ops() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{\"ops\":[{\"type\":\"SetCurrentNode\",\"node_id\":\"canal_gate\"}]}",
        Some(json!({
            "ops": [
                {
                    "type": "SetCurrentNode",
                    "node_id": "canal_gate"
                }
            ]
        })),
    )));
    let keeper = Keeper::new(llm.clone(), "test-model").expect("keeper should build");
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let previous_node = previous_node();
    let current_node = current_node();
    let completed_beats = completed_beats();

    let error = keeper
        .keep(sample_request(
            Some(&previous_node),
            &current_node,
            &character_cards,
            &player_state_schema,
            &world_state,
            &completed_beats,
        ))
        .await
        .expect_err("keeper should reject disallowed ops");

    assert!(error.to_string().contains("SetCurrentNode"));
}

#[test]
fn actor_helper_excludes_thought_segments() {
    let response = ActorResponse {
        speaker_id: "merchant".to_owned(),
        speaker_name: "Haru".to_owned(),
        segments: vec![
            ActorSegment {
                kind: ActorSegmentKind::Dialogue,
                text: "We can still make it.".to_owned(),
            },
            ActorSegment {
                kind: ActorSegmentKind::Thought,
                text: "I hope the guide trusts me.".to_owned(),
            },
            ActorSegment {
                kind: ActorSegmentKind::Action,
                text: "Haru raises the lantern.".to_owned(),
            },
        ],
        raw_output: String::new(),
    };

    let KeeperBeat::Actor {
        visible_segments, ..
    } = KeeperBeat::from_actor_response(ActorPurpose::AdvanceGoal, &response)
    else {
        panic!("expected actor beat");
    };

    assert_eq!(visible_segments.len(), 2);
    assert_eq!(visible_segments[0].kind, KeeperActorSegmentKind::Dialogue);
    assert_eq!(visible_segments[1].kind, KeeperActorSegmentKind::Action);
    assert_eq!(visible_segments[0].text, "We can still make it.");
    assert_eq!(visible_segments[1].text, "Haru raises the lantern.");
}
