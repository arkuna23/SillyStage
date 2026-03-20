mod common;

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;
use llm::{ChatChunk, LlmError};
use serde_json::json;
use ss_agents::actor::{Actor, ActorRequest, ActorSegmentKind, ActorStreamEvent, CharacterCard};
use ss_agents::director::ActorPurpose;
use state::schema::{StateFieldSchema, StateValueType};
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};
use story::NarrativeNode;

use common::{MockLlm, context_entry, prompt_profile};

fn sample_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int)
            .with_default(json!(0))
            .with_description("How much this character trusts the player"),
    )])
}

fn sample_card() -> CharacterCard {
    CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative, casual, slightly cunning".to_owned(),
        state_schema: sample_state_schema(),
        system_prompt:
            "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion.".to_owned(),
    }
}

fn sample_node() -> NarrativeNode {
    NarrativeNode::new(
        "merchant_intro",
        "Merchant Intro",
        "The merchant sizes up a new traveler at the dock.",
        "Convince the traveler to consider a deal.",
        vec!["merchant".to_owned()],
        vec![],
        vec![state::StateOp::SetState {
            key: "entered_intro".to_owned(),
            value: json!(true),
        }],
    )
}

fn sample_request<'a>(
    character: &'a CharacterCard,
    cast: &'a [CharacterCard],
    node: &'a NarrativeNode,
) -> ActorRequest<'a> {
    ActorRequest {
        character,
        cast,
        current_cast_ids: &node.characters,
        lorebook_base: None,
        lorebook_matched: None,
        player_name: Some("Courier"),
        player_description: "A cautious courier carrying a sealed satchel and speaking plainly.",
        purpose: ActorPurpose::AdvanceGoal,
        node,
        memory_limit: None,
    }
}

fn sample_world_state() -> WorldState {
    let mut world_state = WorldState::new("merchant_intro");
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_player_state("coins", json!(12));
    world_state
}

#[tokio::test]
async fn perform_streams_thought_then_action_then_dialogue() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<thought>Maybe".to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta: " I can still profit from this.</thought><action>He reaches for a lantern"
                .to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta: " and lifts it high</action><dialogue>Hello, traveler</dialogue>".to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model")
        .expect("actor should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nYou are the Actor agent of a multi-agent narrative system.",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
            ],
            vec![
                context_entry("shared-history", "SHARED_SCENE_HISTORY", "shared_history"),
                context_entry(
                    "private-memory",
                    "PRIVATE_CHARACTER_MEMORY",
                    "private_memory",
                ),
            ],
        ));
    let mut world_state = sample_world_state();
    let character = sample_card();
    let cast = vec![character.clone()];
    let node = sample_node();

    let mut stream = actor
        .perform_stream(sample_request(&character, &cast, &node), &mut world_state)
        .await
        .expect("perform_stream should start");

    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::ThoughtDelta {
            delta: "Maybe".to_owned()
        }
    );
    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::ThoughtDelta {
            delta: " I can still profit from this.".to_owned()
        }
    );
    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::ActionComplete {
            text: "He reaches for a lantern and lifts it high".to_owned()
        }
    );
    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::DialogueDelta {
            delta: "Hello, traveler".to_owned()
        }
    );

    let ActorStreamEvent::Done { response } = stream.next().await.expect("event").expect("ok")
    else {
        panic!("expected final response event");
    };

    assert_eq!(response.speaker_id, "merchant");
    assert_eq!(response.speaker_name, "Old Merchant");
    assert_eq!(response.segments.len(), 3);
    assert_eq!(response.segments[0].kind, ActorSegmentKind::Thought);
    assert_eq!(response.segments[1].kind, ActorSegmentKind::Action);
    assert_eq!(response.segments[2].kind, ActorSegmentKind::Dialogue);
    assert!(stream.next().await.is_none());
    drop(stream);
    assert_eq!(world_state.actor_shared_history().len(), 2);
    assert_eq!(world_state.actor_private_memory("merchant").len(), 1);
    assert_eq!(
        world_state.actor_private_memory("merchant")[0].text,
        "Maybe I can still profit from this."
    );
}

#[tokio::test]
async fn perform_stream_rejects_text_outside_tags() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "hello<dialogue>bad</dialogue>".to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model")
        .expect("actor should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nYou are the Actor agent of a multi-agent narrative system.",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
            ],
            vec![
                context_entry("shared-history", "SHARED_SCENE_HISTORY", "shared_history"),
                context_entry(
                    "private-memory",
                    "PRIVATE_CHARACTER_MEMORY",
                    "private_memory",
                ),
            ],
        ));
    let mut world_state = sample_world_state();
    let character = sample_card();
    let cast = vec![character.clone()];
    let node = sample_node();
    let mut stream = actor
        .perform_stream(sample_request(&character, &cast, &node), &mut world_state)
        .await
        .expect("perform_stream should start");

    let error = stream
        .next()
        .await
        .expect("error event should exist")
        .expect_err("first event should be an error");

    assert!(error.to_string().contains("outside segment tags"));
}

#[tokio::test]
async fn perform_stream_accepts_out_of_order_segments() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<dialogue>Too early.</dialogue><thought>Should have started here.</thought>"
                .to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model")
        .expect("actor should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nYou are the Actor agent of a multi-agent narrative system.",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
            ],
            vec![
                context_entry("shared-history", "SHARED_SCENE_HISTORY", "shared_history"),
                context_entry(
                    "private-memory",
                    "PRIVATE_CHARACTER_MEMORY",
                    "private_memory",
                ),
            ],
        ));
    let mut world_state = sample_world_state();
    let character = sample_card();
    let cast = vec![character.clone()];
    let node = sample_node();
    let mut stream = actor
        .perform_stream(sample_request(&character, &cast, &node), &mut world_state)
        .await
        .expect("perform_stream should start");

    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::DialogueDelta {
            delta: "Too early.".to_owned()
        }
    );
    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::ThoughtDelta {
            delta: "Should have started here.".to_owned()
        }
    );

    let ActorStreamEvent::Done { response } = stream.next().await.expect("event").expect("ok")
    else {
        panic!("expected done event");
    };

    assert_eq!(response.segments.len(), 2);
    assert_eq!(response.segments[0].kind, ActorSegmentKind::Dialogue);
    assert_eq!(response.segments[1].kind, ActorSegmentKind::Thought);
}

#[test]
fn character_summary_excludes_system_prompt() {
    let summary = sample_card().summary();

    assert_eq!(summary.id, "merchant");
    assert_eq!(summary.name, "Old Merchant");
    assert_eq!(summary.personality, "greedy but friendly trader");
    assert!(summary.state_schema.contains_key("trust"));
}

#[tokio::test]
async fn perform_stream_sends_character_specific_system_prompt() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<dialogue>Deal?</dialogue>".to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model")
        .expect("actor should build")
        .with_prompt_profile(prompt_profile(
            "ROLE:\nYou are the Actor agent of a multi-agent narrative system.",
            vec![
                context_entry("player", "PLAYER", "player"),
                context_entry("current-node", "CURRENT_NODE", "current_node"),
            ],
            vec![
                context_entry("shared-history", "SHARED_SCENE_HISTORY", "shared_history"),
                context_entry(
                    "private-memory",
                    "PRIVATE_CHARACTER_MEMORY",
                    "private_memory",
                ),
            ],
        ));
    let mut world_state = sample_world_state();
    let mut character = sample_card();
    character.personality = "{{char}} keeps a careful eye on {{user}}.".to_owned();
    character.style = "Measured when speaking to {{user}}.".to_owned();
    character.system_prompt = "Address {{user}} directly as {{char}}.".to_owned();
    let cast = vec![character.clone()];
    let node = sample_node();
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "guide".to_owned(),
            speaker_name: "Yuki".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "Stay close to the lantern light.".to_owned(),
        },
        8,
    );
    world_state.push_player_input_shared_memory("Can you get us through the flooded gate?", 8);
    world_state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "If I play this right, the route stays mine.".to_owned(),
        },
        8,
    );

    let _ = actor
        .perform(sample_request(&character, &cast, &node), &mut world_state)
        .await
        .expect("perform should work");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    assert_eq!(request.messages.len(), 2);
    assert!(
        request.messages[0]
            .content
            .contains("Old Merchant keeps a careful eye on Courier.")
    );
    assert!(
        request.messages[0]
            .content
            .contains("Measured when speaking to Courier.")
    );
    assert!(
        request.messages[0]
            .content
            .contains("Address Courier directly as Old Merchant.")
    );
    assert!(
        request.messages[0]
            .content
            .contains("You are the Actor agent of a multi-agent narrative system.")
    );
    assert!(!request.messages[1].content.contains("PLAYER_NAME"));
    assert!(request.messages[1].content.contains("CURRENT_NODE"));
    assert!(!request.messages[1].content.contains("on_enter_updates"));
    assert!(!request.messages[1].content.contains("entered_intro"));
    assert!(
        request.messages[1]
            .content
            .contains("A cautious courier carrying a sealed satchel")
    );
    assert!(request.messages[1].content.contains("SHARED_SCENE_HISTORY"));
    assert!(
        request.messages[1]
            .content
            .contains("Stay close to the lantern light.")
    );
    assert!(
        request.messages[1]
            .content
            .contains("Can you get us through the flooded gate?")
    );
    assert!(!request.messages[1].content.contains("\"player_state\""));
    assert!(!request.messages[1].content.contains("\"coins\": 12"));
    assert!(
        request.messages[1]
            .content
            .contains("PRIVATE_CHARACTER_MEMORY")
    );
    assert!(
        request.messages[1]
            .content
            .contains("If I play this right, the route stays mine.")
    );
    assert!(
        !request.messages[1]
            .content
            .contains("\"actor_shared_history\"")
    );
    assert!(
        !request.messages[1]
            .content
            .contains("\"actor_private_memory\"")
    );
}

#[tokio::test]
async fn perform_stream_uses_user_fallback_for_character_templates() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<dialogue>Understood.</dialogue>".to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    let mut character = sample_card();
    character.system_prompt = "Speak to {{user}} as {{char}}.".to_owned();
    let cast = vec![character.clone()];
    let node = sample_node();
    let mut request = sample_request(&character, &cast, &node);
    request.player_name = None;

    let _ = actor
        .perform(request, &mut world_state)
        .await
        .expect("perform should work");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");

    assert!(
        request.messages[0]
            .content
            .contains("Speak to User as Old Merchant.")
    );
}

#[tokio::test]
async fn perform_stream_renders_character_schema_templates_from_runtime_state() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<dialogue>Understood.</dialogue>".to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    world_state.set_character_state("merchant", "trust", json!(3));
    world_state.set_character_state("merchant", "inventory", json!(["lantern", "rope"]));
    world_state.set_character_state("merchant", "profile", json!({"mood": "alert"}));

    let mut character = sample_card();
    character.state_schema = HashMap::from([
        (
            "trust".to_owned(),
            StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
        ),
        (
            "inventory".to_owned(),
            StateFieldSchema::new(StateValueType::Array).with_default(json!(["ledger"])),
        ),
        (
            "profile".to_owned(),
            StateFieldSchema::new(StateValueType::Object).with_default(json!({"mood": "wary"})),
        ),
    ]);
    character.system_prompt =
        "trust={{trust}} inventory={{inventory}} profile={{profile}} missing={{missing}}"
            .to_owned();
    let cast = vec![character.clone()];
    let node = sample_node();

    let _ = actor
        .perform(sample_request(&character, &cast, &node), &mut world_state)
        .await
        .expect("perform should work");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");

    assert!(request.messages[0].content.contains("trust=3"));
    assert!(
        request.messages[0]
            .content
            .contains("inventory=[\"lantern\",\"rope\"]")
    );
    assert!(
        request.messages[0]
            .content
            .contains("profile={\"mood\":\"alert\"}")
    );
    assert!(request.messages[0].content.contains("missing={{missing}}"));
}

#[tokio::test]
async fn llm_stream_errors_surface_through_actor() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![Err(
        LlmError::RateLimited,
    )]));
    let actor = Actor::new(llm.clone(), "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    let character = sample_card();
    let cast = vec![character.clone()];
    let node = sample_node();
    let mut stream = actor
        .perform_stream(sample_request(&character, &cast, &node), &mut world_state)
        .await
        .expect("perform_stream should start");

    let error = stream
        .next()
        .await
        .expect("error event should exist")
        .expect_err("first event should be an error");

    assert!(matches!(
        error,
        ss_agents::actor::ActorError::Llm(LlmError::RateLimited)
    ));
}

#[tokio::test]
async fn perform_respects_memory_limit_and_only_shares_visible_segments() {
    let llm = Arc::new(MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<thought>Keep the better margin hidden.</thought><action>He slides a small crate forward.</action><dialogue>First offer.</dialogue>".to_owned(),
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
    ]));
    let actor = Actor::new(llm.clone(), "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    let character = sample_card();
    let cast = vec![character.clone()];
    let node = sample_node();
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "Older line".to_owned(),
        },
        2,
    );
    world_state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "Older thought".to_owned(),
        },
        2,
    );

    let mut request = sample_request(&character, &cast, &node);
    request.memory_limit = Some(2);
    let response = actor
        .perform(request, &mut world_state)
        .await
        .expect("perform should work");

    assert_eq!(response.segments.len(), 3);
    assert_eq!(world_state.actor_shared_history().len(), 2);
    assert_eq!(
        world_state.actor_shared_history()[0].text,
        "He slides a small crate forward."
    );
    assert_eq!(world_state.actor_shared_history()[1].text, "First offer.");
    assert_eq!(world_state.actor_private_memory("merchant").len(), 2);
    assert_eq!(
        world_state.actor_private_memory("merchant")[0].text,
        "Older thought"
    );
    assert_eq!(
        world_state.actor_private_memory("merchant")[1].text,
        "Keep the better margin hidden."
    );
}
