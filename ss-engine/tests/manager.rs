mod common;

use std::collections::HashMap;
use std::sync::Arc;

use futures_util::StreamExt;
use serde_json::json;
use ss_engine::{AgentApiIds, EngineEvent, EngineManager, LlmApiRegistry, SessionConfigMode};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use store::{CharacterCardRecord, InMemoryStore, Store, StoryRecord, StoryResourcesRecord};
use story::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

use agents::actor::CharacterCard;
use common::{QueuedMockLlm, assistant_response};

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner".to_owned(),
        architect_api_id: "architect".to_owned(),
        director_api_id: "director".to_owned(),
        actor_api_id: "actor".to_owned(),
        narrator_api_id: "narrator".to_owned(),
        keeper_api_id: "keeper".to_owned(),
    }
}

fn registry<'a>(llm: &'a QueuedMockLlm) -> LlmApiRegistry<'a> {
    let ids = sample_api_ids();
    LlmApiRegistry::new()
        .register(ids.planner_api_id, llm, "planner-model")
        .register(ids.architect_api_id, llm, "architect-model")
        .register(ids.director_api_id, llm, "director-model")
        .register(ids.actor_api_id, llm, "actor-model")
        .register(ids.narrator_api_id, llm, "narrator-model")
        .register(ids.keeper_api_id, llm, "keeper-model")
}

fn sample_character_record() -> CharacterCardRecord {
    CharacterCardRecord {
        character_id: "merchant".to_owned(),
        content: CharacterCard {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual".to_owned(),
            tendencies: vec!["likes profitable deals".to_owned()],
            state_schema: HashMap::from([(
                "trust".to_owned(),
                StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
            )]),
            system_prompt: "Stay in character.".to_owned(),
        },
        cover_file_name: "cover.png".to_owned(),
        cover_mime_type: "image/png".to_owned(),
        cover_bytes: b"cover".to_vec(),
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
        .save_character(sample_character_record())
        .await
        .expect("save character");
    store
        .save_story_resources(StoryResourcesRecord {
            resource_id: "resource-1".to_owned(),
            story_concept: "A flooded harbor story.".to_owned(),
            character_ids: vec!["merchant".to_owned()],
            player_state_schema_seed: sample_player_state_schema(),
            world_state_schema_seed: Some(sample_world_state_schema()),
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
            world_state_schema: sample_world_state_schema(),
            player_state_schema: sample_player_state_schema(),
            introduction: "The courier reaches a flooded dock.".to_owned(),
        })
        .await
        .expect("save story");
}

#[tokio::test]
async fn manager_starts_session_from_story_and_exposes_snapshot() {
    let llm = QueuedMockLlm::new(vec![], vec![]);
    let store = Arc::new(InMemoryStore::new());
    seed_story(&store).await;

    let manager = EngineManager::new(store.clone(), registry(&llm), sample_api_ids())
        .await
        .expect("manager should build");

    let session = manager
        .start_session_from_story(
            "story-1",
            Some("Courier Run".to_owned()),
            "A determined courier.".to_owned(),
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
}

#[tokio::test]
async fn manager_runs_turn_and_keeps_sessions_isolated() {
    let llm = QueuedMockLlm::new(
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
    );
    let store = Arc::new(InMemoryStore::new());
    seed_story(&store).await;

    let manager = EngineManager::new(store.clone(), registry(&llm), sample_api_ids())
        .await
        .expect("manager should build");

    let session_a = manager
        .start_session_from_story(
            "story-1",
            Some("Run A".to_owned()),
            "A determined courier.".to_owned(),
            SessionConfigMode::UseGlobal,
            None,
        )
        .await
        .expect("session should start");
    let session_b = manager
        .start_session_from_story(
            "story-1",
            Some("Run B".to_owned()),
            "A cautious courier.".to_owned(),
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
    assert_eq!(updated_b.snapshot.turn_index, 0);
    assert_eq!(updated_b.snapshot.world_state.current_node(), "dock");
}
