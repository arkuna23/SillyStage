mod common;

use futures_util::StreamExt;
use llm::{ChatChunk, LlmError};
use serde_json::json;
use ss_agents::actor::{Actor, ActorRequest, ActorSegmentKind, ActorStreamEvent, CharacterCard};
use ss_agents::director::ActorPurpose;
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};
use story::NarrativeNode;

use common::MockLlm;

fn sample_card() -> CharacterCard {
    CharacterCard {
        id: "merchant".to_owned(),
        name: "Old Merchant".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative, casual, slightly cunning".to_owned(),
        tendencies: vec![
            "likes profitable deals".to_owned(),
            "avoids danger".to_owned(),
            "tries to maintain good relationships".to_owned(),
        ],
        system_prompt:
            "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion.".to_owned(),
    }
}

fn sample_request() -> ActorRequest {
    let character = sample_card();
    ActorRequest {
        character: character.clone(),
        cast: vec![character],
        purpose: ActorPurpose::AdvanceGoal,
        node: NarrativeNode::new(
            "merchant_intro",
            "Merchant Intro",
            "The merchant sizes up a new traveler at the dock.",
            "Convince the traveler to consider a deal.",
            vec!["merchant".to_owned()],
            vec![],
            vec![],
        ),
        memory_limit: None,
    }
}

fn sample_world_state() -> WorldState {
    let mut world_state = WorldState::new("merchant_intro");
    world_state.set_state("flood_gate_open", json!(false));
    world_state
}

#[tokio::test]
async fn perform_streams_dialogue_and_thought_but_buffers_action() {
    let llm = MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<dialogue>Hello".to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta: ", traveler</dialogue><action>He reaches for a lantern".to_owned(),
            model: Some("test-model".to_owned()),
            finish_reason: None,
            done: false,
            usage: None,
        }),
        Ok(ChatChunk {
            delta:
                " and lifts it high</action><thought>Maybe I can still profit from this.</thought>"
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
    ]);
    let actor = Actor::new(&llm, "test-model").expect("actor should build");
    let mut world_state = sample_world_state();

    let mut stream = actor
        .perform_stream(sample_request(), &mut world_state)
        .await
        .expect("perform_stream should start");

    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::DialogueDelta {
            delta: "Hello".to_owned()
        }
    );
    assert_eq!(
        stream.next().await.expect("event").expect("ok"),
        ActorStreamEvent::DialogueDelta {
            delta: ", traveler".to_owned()
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
        ActorStreamEvent::ThoughtDelta {
            delta: "Maybe I can still profit from this.".to_owned()
        }
    );

    let ActorStreamEvent::Done { response } = stream.next().await.expect("event").expect("ok")
    else {
        panic!("expected final response event");
    };

    assert_eq!(response.speaker_id, "merchant");
    assert_eq!(response.speaker_name, "Old Merchant");
    assert_eq!(response.segments.len(), 3);
    assert_eq!(response.segments[0].kind, ActorSegmentKind::Dialogue);
    assert_eq!(response.segments[1].kind, ActorSegmentKind::Action);
    assert_eq!(response.segments[2].kind, ActorSegmentKind::Thought);
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
    let llm = MockLlm::with_stream_chunks(vec![
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
    ]);
    let actor = Actor::new(&llm, "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    let mut stream = actor
        .perform_stream(sample_request(), &mut world_state)
        .await
        .expect("perform_stream should start");

    let error = stream
        .next()
        .await
        .expect("error event should exist")
        .expect_err("first event should be an error");

    assert!(error.to_string().contains("outside segment tags"));
}

#[test]
fn character_summary_excludes_system_prompt() {
    let summary = sample_card().summary();

    assert_eq!(summary.id, "merchant");
    assert_eq!(summary.name, "Old Merchant");
    assert_eq!(summary.tendencies.len(), 3);
}

#[tokio::test]
async fn perform_stream_sends_character_specific_system_prompt() {
    let llm = MockLlm::with_stream_chunks(vec![
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
    ]);
    let actor = Actor::new(&llm, "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "guide".to_owned(),
            speaker_name: "Yuki".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "Stay close to the lantern light.".to_owned(),
        },
        8,
    );
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
        .perform(sample_request(), &mut world_state)
        .await
        .expect("perform should work");

    let requests = llm.recorded_requests();
    let request = requests.first().expect("request should be recorded");
    assert_eq!(request.messages.len(), 3);
    assert!(request.messages[1].content.contains("traveling merchant"));
    assert!(request.messages[2].content.contains("SHARED_SCENE_HISTORY"));
    assert!(
        request.messages[2]
            .content
            .contains("Stay close to the lantern light.")
    );
    assert!(
        request.messages[2]
            .content
            .contains("PRIVATE_CHARACTER_MEMORY")
    );
    assert!(
        request.messages[2]
            .content
            .contains("If I play this right, the route stays mine.")
    );
    assert!(
        !request.messages[2]
            .content
            .contains("\"actor_shared_history\"")
    );
    assert!(
        !request.messages[2]
            .content
            .contains("\"actor_private_memory\"")
    );
}

#[tokio::test]
async fn llm_stream_errors_surface_through_actor() {
    let llm = MockLlm::with_stream_chunks(vec![Err(LlmError::RateLimited)]);
    let actor = Actor::new(&llm, "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
    let mut stream = actor
        .perform_stream(sample_request(), &mut world_state)
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
    let llm = MockLlm::with_stream_chunks(vec![
        Ok(ChatChunk {
            delta: "<dialogue>First offer.</dialogue><thought>Keep the better margin hidden.</thought><action>He slides a small crate forward.</action>".to_owned(),
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
    let actor = Actor::new(&llm, "test-model").expect("actor should build");
    let mut world_state = sample_world_state();
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

    let mut request = sample_request();
    request.memory_limit = Some(2);
    let response = actor
        .perform(request, &mut world_state)
        .await
        .expect("perform should work");

    assert_eq!(response.segments.len(), 3);
    assert_eq!(world_state.actor_shared_history().len(), 2);
    assert_eq!(world_state.actor_shared_history()[0].text, "First offer.");
    assert_eq!(
        world_state.actor_shared_history()[1].text,
        "He slides a small crate forward."
    );
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
