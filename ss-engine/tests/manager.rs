mod common;

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;
use serde_json::json;
use ss_engine::{AgentApiIds, EngineEvent, EngineManager, LlmApiRegistry, SessionConfigMode};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use store::{
    CharacterCardDefinition, CharacterCardRecord, InMemoryStore, PlayerProfileRecord, SchemaRecord,
    Store, StoryRecord, StoryResourcesRecord,
};
use story::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

use common::{QueuedMockLlm, assistant_response};

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner".to_owned(),
        architect_api_id: "architect".to_owned(),
        director_api_id: "director".to_owned(),
        actor_api_id: "actor".to_owned(),
        narrator_api_id: "narrator".to_owned(),
        keeper_api_id: "keeper".to_owned(),
        replyer_api_id: "replyer".to_owned(),
    }
}

fn registry(llm: Arc<QueuedMockLlm>) -> LlmApiRegistry {
    let ids = sample_api_ids();
    let llm: Arc<dyn llm::LlmApi> = llm;
    LlmApiRegistry::new()
        .register(ids.planner_api_id, Arc::clone(&llm), "planner-model")
        .register(ids.architect_api_id, Arc::clone(&llm), "architect-model")
        .register(ids.director_api_id, Arc::clone(&llm), "director-model")
        .register(ids.actor_api_id, Arc::clone(&llm), "actor-model")
        .register(ids.narrator_api_id, Arc::clone(&llm), "narrator-model")
        .register(ids.keeper_api_id, Arc::clone(&llm), "keeper-model")
        .register(ids.replyer_api_id, llm, "replyer-model")
}

fn sample_character_record() -> CharacterCardRecord {
    CharacterCardRecord {
        character_id: "merchant".to_owned(),
        content: CharacterCardDefinition {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual".to_owned(),
            tendencies: vec!["likes profitable deals".to_owned()],
            schema_id: "schema-character-merchant".to_owned(),
            system_prompt: "Stay in character.".to_owned(),
        },
        cover_file_name: Some("cover.png".to_owned()),
        cover_mime_type: Some("image/png".to_owned()),
        cover_bytes: Some(b"cover".to_vec()),
    }
}

fn sample_schema_record(schema_id: &str, display_name: &str) -> SchemaRecord {
    let fields = if schema_id.contains("world") {
        sample_world_state_schema().fields
    } else if schema_id.contains("player") {
        sample_player_state_schema().fields
    } else {
        HashMap::from([(
            "trust".to_owned(),
            StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
        )])
    };

    SchemaRecord {
        schema_id: schema_id.to_owned(),
        display_name: display_name.to_owned(),
        tags: vec!["test".to_owned()],
        fields,
    }
}

fn sample_player_profile(id: &str, description: &str) -> PlayerProfileRecord {
    PlayerProfileRecord {
        player_profile_id: id.to_owned(),
        display_name: id.to_owned(),
        description: description.to_owned(),
    }
}

fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

fn sample_world_state_schema() -> WorldStateSchema {
    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    schema
}

fn sample_story_graph() -> StoryGraph {
    StoryGraph::new(
        "dock",
        vec![
            NarrativeNode::new(
                "dock",
                "Flooded Dock",
                "A flooded dock at dusk.",
                "Decide whether to trust the merchant.",
                vec!["merchant".to_owned()],
                vec![Transition::new(
                    "gate",
                    Condition::for_character("merchant", "trust", ConditionOperator::Gte, json!(2)),
                )],
                vec![],
            ),
            NarrativeNode::new(
                "gate",
                "Canal Gate",
                "A narrow ledge beside the gate.",
                "Open the route.",
                vec!["merchant".to_owned()],
                vec![],
                vec![state::StateOp::SetState {
                    key: "entered_gate".to_owned(),
                    value: json!(true),
                }],
            ),
        ],
    )
}

async fn seed_story(store: &InMemoryStore) {
    store
        .save_schema(sample_schema_record(
            "schema-character-merchant",
            "Merchant Schema",
        ))
        .await
        .expect("save character schema");
    store
        .save_schema(sample_schema_record("schema-player-default", "Player Seed"))
        .await
        .expect("save player seed");
    store
        .save_schema(sample_schema_record("schema-world-default", "World Seed"))
        .await
        .expect("save world seed");
    store
        .save_schema(sample_schema_record(
            "schema-player-story-1",
            "Player Story Schema",
        ))
        .await
        .expect("save story player schema");
    store
        .save_schema(sample_schema_record(
            "schema-world-story-1",
            "World Story Schema",
        ))
        .await
        .expect("save story world schema");
    store
        .save_player_profile(sample_player_profile(
            "profile-courier-a",
            "A determined courier.",
        ))
        .await
        .expect("save player profile a");
    store
        .save_player_profile(sample_player_profile(
            "profile-courier-b",
            "A cautious courier.",
        ))
        .await
        .expect("save player profile b");
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");
    store
        .save_story_resources(StoryResourcesRecord {
            resource_id: "resource-1".to_owned(),
            story_concept: "A flooded harbor story.".to_owned(),
            character_ids: vec!["merchant".to_owned()],
            player_schema_id_seed: Some("schema-player-default".to_owned()),
            world_schema_id_seed: Some("schema-world-default".to_owned()),
            planned_story: None,
        })
        .await
        .expect("save resources");
    store
        .save_story(StoryRecord {
            story_id: "story-1".to_owned(),
            display_name: "Flooded Harbor".to_owned(),
            resource_id: "resource-1".to_owned(),
            graph: sample_story_graph(),
            world_schema_id: "schema-world-story-1".to_owned(),
            player_schema_id: "schema-player-story-1".to_owned(),
            introduction: "The courier reaches a flooded dock.".to_owned(),
            created_at_ms: Some(1_000),
            updated_at_ms: Some(1_000),
        })
        .await
        .expect("save story");
}

#[tokio::test]
async fn manager_starts_session_from_story_and_exposes_snapshot() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story(&store).await;

    let manager = EngineManager::new(store.clone(), registry(llm.clone()), Some(sample_api_ids()))
        .await
        .expect("manager should build");

    let session = manager
        .start_session_from_story(
            "story-1",
            Some("Courier Run".to_owned()),
            Some("profile-courier-a".to_owned()),
            SessionConfigMode::UseGlobal,
            None,
        )
        .await
        .expect("session should start");

    assert_eq!(session.display_name, "Courier Run");
    let snapshot = manager
        .get_runtime_snapshot(&session.session_id)
        .await
        .expect("snapshot should load");
    assert_eq!(snapshot.story_id, "story-1");
    assert_eq!(snapshot.player_description, "A determined courier.");
    assert_eq!(snapshot.world_state.current_node(), "dock");
    assert!(session.created_at_ms.is_some());
    assert!(session.updated_at_ms.is_some());
}

#[tokio::test]
async fn manager_suggests_replies_without_mutating_session() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
            "{}",
            Some(json!({
                "replies": [
                    { "id": "r1", "text": "Show me the fastest safe route." },
                    { "id": "r2", "text": "What exactly are you charging?" },
                    { "id": "r3", "text": "I need proof before I commit." }
                ]
            })),
        ))],
        vec![],
    ));
    let store = Arc::new(InMemoryStore::new());
    seed_story(&store).await;

    let manager = EngineManager::new(store.clone(), registry(llm.clone()), Some(sample_api_ids()))
        .await
        .expect("manager should build");
    let session = manager
        .start_session_from_story(
            "story-1",
            Some("Courier Run".to_owned()),
            Some("profile-courier-a".to_owned()),
            SessionConfigMode::UseGlobal,
            None,
        )
        .await
        .expect("session should start");

    let before = store
        .get_session(&session.session_id)
        .await
        .expect("session lookup should succeed")
        .expect("session should exist");
    let replies = manager
        .suggest_replies(&session.session_id, 3, None)
        .await
        .expect("reply suggestions should succeed");
    let after = store
        .get_session(&session.session_id)
        .await
        .expect("session lookup should succeed")
        .expect("session should exist");

    assert_eq!(replies.len(), 3);
    assert_eq!(before.story_id, after.story_id);
    assert_eq!(before.display_name, after.display_name);
    assert_eq!(before.player_profile_id, after.player_profile_id);
    assert_eq!(before.player_schema_id, after.player_schema_id);
    assert_eq!(before.snapshot.turn_index, after.snapshot.turn_index);
    assert_eq!(before.snapshot.story_id, after.snapshot.story_id);
    assert_eq!(
        before.snapshot.player_description,
        after.snapshot.player_description
    );
    assert_eq!(
        before.snapshot.world_state.current_node(),
        after.snapshot.world_state.current_node()
    );
    assert_eq!(
        serde_json::to_value(&before.snapshot.world_state).expect("world state should serialize"),
        serde_json::to_value(&after.snapshot.world_state).expect("world state should serialize")
    );
    assert!(
        store
            .list_session_messages(&session.session_id)
            .await
            .expect("message lookup should succeed")
            .is_empty()
    );
}

#[tokio::test]
async fn manager_runs_turn_and_keeps_sessions_isolated() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[{\"type\":\"SetCharacterState\",\"character\":\"merchant\",\"key\":\"trust\",\"value\":3}]}",
                Some(json!({
                    "ops": [
                        {
                            "type": "SetCharacterState",
                            "character": "merchant",
                            "key": "trust",
                            "value": 3
                        }
                    ]
                })),
            )),
            Ok(assistant_response(
                "{\"beats\":[{\"type\":\"Narrator\",\"purpose\":\"DescribeTransition\"}]}",
                Some(json!({
                    "beats": [
                        {
                            "type": "Narrator",
                            "purpose": "DescribeTransition"
                        }
                    ]
                })),
            )),
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({ "ops": [] })),
            )),
        ],
        vec![Ok(vec![
            Ok(llm::ChatChunk {
                delta: "Water churned beneath the old gate.".to_owned(),
                model: Some("test-model".to_owned()),
                finish_reason: None,
                done: false,
                usage: None,
            }),
            Ok(llm::ChatChunk {
                delta: String::new(),
                model: Some("test-model".to_owned()),
                finish_reason: Some("stop".to_owned()),
                done: true,
                usage: None,
            }),
        ])],
    ));
    let store = Arc::new(InMemoryStore::new());
    seed_story(&store).await;

    let manager = EngineManager::new(store.clone(), registry(llm.clone()), Some(sample_api_ids()))
        .await
        .expect("manager should build");

    let session_a = manager
        .start_session_from_story(
            "story-1",
            Some("Run A".to_owned()),
            Some("profile-courier-a".to_owned()),
            SessionConfigMode::UseGlobal,
            None,
        )
        .await
        .expect("session should start");
    let session_b = manager
        .start_session_from_story(
            "story-1",
            Some("Run B".to_owned()),
            Some("profile-courier-b".to_owned()),
            SessionConfigMode::UseGlobal,
            None,
        )
        .await
        .expect("session should start");

    let mut stream = manager
        .run_turn_stream(
            &session_a.session_id,
            "Open the canal gate.".to_owned(),
            None,
        )
        .await
        .expect("turn stream should start");

    let mut completed = false;
    while let Some(event) = stream.next().await {
        match event.expect("managed stream event should succeed") {
            EngineEvent::TurnCompleted { result } => {
                completed = true;
                assert_eq!(result.snapshot.world_state.current_node(), "gate");
            }
            EngineEvent::TurnFailed { error, .. } => {
                panic!("unexpected failure: {error}");
            }
            _ => {}
        }
    }

    assert!(completed);

    let updated_a = store
        .get_session(&session_a.session_id)
        .await
        .expect("load session a")
        .expect("session a should exist");
    let updated_b = store
        .get_session(&session_b.session_id)
        .await
        .expect("load session b")
        .expect("session b should exist");

    assert_eq!(updated_a.snapshot.turn_index, 1);
    assert_eq!(updated_a.snapshot.world_state.current_node(), "gate");
    assert!(updated_a.updated_at_ms >= updated_a.created_at_ms);
    assert_eq!(updated_b.snapshot.turn_index, 0);
    assert_eq!(updated_b.snapshot.world_state.current_node(), "dock");
}
