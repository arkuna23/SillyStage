mod common;

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::replyer::{
    ReplyHistoryKind, ReplyHistoryMessage, ReplyOption, Replyer, ReplyerRequest,
};
use state::{
    ActorMemoryEntry, ActorMemoryKind, PlayerStateSchema, StateFieldSchema, StateValueType,
    WorldState,
};
use story::{Condition, ConditionOperator, NarrativeNode, Transition};

use common::{MockLlm, assistant_response, context_entry, prompt_profile};

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

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![CharacterCard {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative, casual".to_owned(),
        state_schema: merchant_state_schema(),
        system_prompt: "Stay in character.".to_owned(),
    }]
}

fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

fn sample_world_state() -> WorldState {
    let mut world_state = WorldState::new("dock");
    world_state.set_active_characters(vec!["merchant".to_owned()]);
    world_state.set_player_state("coins", json!(12));
    world_state.push_player_input_shared_memory("Can you lower the price?", 8);
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "If you move quickly, maybe.".to_owned(),
        },
        8,
    );
    world_state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "I can squeeze a little more out of this deal.".to_owned(),
        },
        8,
    );
    world_state
}

fn sample_history() -> Vec<ReplyHistoryMessage> {
    vec![
        ReplyHistoryMessage {
            kind: ReplyHistoryKind::PlayerInput,
            turn_index: 1,
            speaker_id: "player".to_owned(),
            speaker_name: "Player".to_owned(),
            text: "Can you lower the price?".to_owned(),
        },
        ReplyHistoryMessage {
            kind: ReplyHistoryKind::Dialogue,
            turn_index: 1,
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            text: "If you move quickly, maybe.".to_owned(),
        },
    ]
}

fn sample_node() -> NarrativeNode {
    NarrativeNode::new(
        "dock",
        "Flooded Dock",
        "A flooded dock at dusk.",
        "Negotiate passage through the market gate.",
        vec!["merchant".to_owned()],
        vec![
            Transition::new(
                "market_gate",
                Condition::for_player("coins", ConditionOperator::Gte, json!(10)),
            ),
            Transition::new(
                "safe_route",
                Condition::new("route_committed", ConditionOperator::Eq, json!(true)),
            ),
        ],
        vec![state::StateOp::SetState {
            key: "entered_dock".to_owned(),
            value: json!(true),
        }],
    )
}

#[tokio::test]
async fn suggest_returns_sanitized_replies() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{}",
        Some(json!({
            "replies": [
                { "id": "ask_price", "text": "Can you be specific about the price?" },
                { "id": "", "text": "What do I get if I agree now?" },
                { "id": "duplicate", "text": "What do I get if I agree now?" },
                { "id": "blank", "text": "   " },
                { "id": "last", "text": "I need a safer route, not a faster one." }
            ]
        })),
    )));
    let replyer = Replyer::new(llm.clone(), "test-model")
        .expect("replyer should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nReplyer",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-cast", "CURRENT_CAST", "current_cast"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
                context_entry(
                    "player-state-schema",
                    "PLAYER_STATE_SCHEMA",
                    "player_state_schema",
                ),
            ],
            vec![
                context_entry("world-state", "WORLD_STATE", "world_state"),
                context_entry("session-history", "SESSION_HISTORY", "session_history"),
            ],
        ));
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let history = sample_history();
    let node = sample_node();

    let response = replyer
        .suggest(ReplyerRequest {
            current_node: &node,
            character_cards: &character_cards,
            current_cast_ids: &node.characters,
            lorebook_base: None,
            lorebook_matched: None,
            player_name: Some("Courier"),
            player_description: "A cautious courier who negotiates directly.",
            player_state_schema: &player_state_schema,
            world_state: &world_state,
            history: &history,
            limit: 3,
        })
        .await
        .expect("reply suggestions should succeed");

    assert_eq!(
        response.replies,
        vec![
            ReplyOption {
                id: "ask_price".to_owned(),
                text: "Can you be specific about the price?".to_owned(),
            },
            ReplyOption {
                id: "reply-1".to_owned(),
                text: "What do I get if I agree now?".to_owned(),
            },
            ReplyOption {
                id: "last".to_owned(),
                text: "I need a safer route, not a faster one.".to_owned(),
            },
        ]
    );
}

#[tokio::test]
async fn prompt_includes_history_and_world_state_without_private_memory() {
    let llm = Arc::new(MockLlm::with_chat_response(assistant_response(
        "{}",
        Some(json!({
            "replies": [
                { "id": "r1", "text": "Show me the route first." }
            ]
        })),
    )));
    let replyer = Replyer::new(llm.clone(), "test-model")
        .expect("replyer should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nReplyer",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-cast", "CURRENT_CAST", "current_cast"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
                context_entry(
                    "player-state-schema",
                    "PLAYER_STATE_SCHEMA",
                    "player_state_schema",
                ),
            ],
            vec![
                context_entry("world-state", "WORLD_STATE", "world_state"),
                context_entry("session-history", "SESSION_HISTORY", "session_history"),
            ],
        ));
    let character_cards = sample_character_cards();
    let player_state_schema = sample_player_state_schema();
    let world_state = sample_world_state();
    let history = sample_history();
    let node = sample_node();

    let _ = replyer
        .suggest(ReplyerRequest {
            current_node: &node,
            character_cards: &character_cards,
            current_cast_ids: &node.characters,
            lorebook_base: None,
            lorebook_matched: None,
            player_name: Some("Courier"),
            player_description: "A cautious courier who negotiates directly.",
            player_state_schema: &player_state_schema,
            world_state: &world_state,
            history: &history,
            limit: 3,
        })
        .await
        .expect("reply suggestions should succeed");

    let requests = llm.recorded_requests();
    let user_message = joined_user_messages(&requests[0]);

    assert!(user_message.contains("SESSION_HISTORY"));
    assert!(user_message.contains("to node=market_gate when player.coins >= 10"));
    assert!(user_message.contains("to node=safe_route when global.route_committed == true"));
    assert!(!user_message.contains("on_enter_updates"));
    assert!(!user_message.contains("entered_dock"));
    assert!(!user_message.contains("PLAYER_NAME"));
    assert!(user_message.contains("Can you lower the price?"));
    assert!(user_message.contains("coins=12"));
    assert!(!user_message.contains("I can squeeze a little more out of this deal."));
}
