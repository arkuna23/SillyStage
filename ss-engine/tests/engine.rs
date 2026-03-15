mod common;

use std::collections::HashMap;

use futures_util::StreamExt;
use llm::{ChatChunk, LlmError, Role};
use serde_json::json;
use ss_engine::{
    Engine, EngineError, EngineEvent, EngineStage, RuntimeAgentConfigs, RuntimeState,
    StoryGenerationAgentConfigs, StoryResources, generate_story_graph, generate_story_plan,
};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use story::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

use agents::actor::CharacterCard;

use common::{QueuedMockLlm, assistant_response};

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![
        CharacterCard {
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
        CharacterCard {
            id: "guide".to_owned(),
            name: "Yuki".to_owned(),
            personality: "calm local guide".to_owned(),
            style: "measured".to_owned(),
            tendencies: vec!["protects civilians".to_owned()],
            state_schema: HashMap::new(),
            system_prompt: "Stay observant.".to_owned(),
        },
    ]
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

fn sample_player_transition_story_graph() -> StoryGraph {
    StoryGraph::new(
        "checkpoint",
        vec![
            NarrativeNode::new(
                "checkpoint",
                "Checkpoint",
                "A guard watches the floodgate and waits for proof of passage.",
                "Determine whether the courier can pass.",
                vec!["merchant".to_owned()],
                vec![Transition::new(
                    "vip_gate",
                    Condition::for_player("coins", ConditionOperator::Gte, json!(10)),
                )],
                vec![],
            ),
            NarrativeNode::new(
                "vip_gate",
                "VIP Gate",
                "The guard opens the fastest route.",
                "Proceed through the VIP gate.",
                vec!["merchant".to_owned()],
                vec![],
                vec![],
            ),
            NarrativeNode::new(
                "regular_gate",
                "Regular Gate",
                "The guard redirects the courier to the slower path.",
                "Proceed through the regular gate.",
                vec!["merchant".to_owned()],
                vec![],
                vec![],
            ),
        ],
    )
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
        "entered_gate",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    schema
}

fn sample_player_description() -> &'static str {
    "A stubborn courier protecting a sealed satchel of medicine."
}

fn sample_runtime_state() -> RuntimeState {
    RuntimeState::from_story_graph(
        "flooded_city_demo",
        sample_story_graph(),
        sample_character_cards(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect("runtime state should build")
}

fn sample_runtime_state_with_story_graph(story_graph: StoryGraph) -> RuntimeState {
    RuntimeState::from_story_graph(
        "flooded_city_demo",
        story_graph,
        sample_character_cards(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect("runtime state should build")
}

fn sample_story_resources() -> StoryResources {
    StoryResources::new(
        "flooded_city_demo",
        "A flooded dock story",
        sample_character_cards(),
        Some(sample_player_state_schema()),
    )
    .expect("story resources should build")
    .with_world_state_schema_seed(sample_world_state_schema())
}

fn user_message_content(request: &llm::ChatRequest) -> String {
    request
        .messages
        .iter()
        .filter(|message| matches!(message.role, Role::User))
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[tokio::test]
async fn run_turn_stream_emits_full_pipeline_and_updates_state() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[{\"type\":\"SetCharacterState\",\"character\":\"merchant\",\"key\":\"trust\",\"value\":3},{\"type\":\"SetPlayerState\",\"key\":\"coins\",\"value\":9}]}",
                Some(json!({
                    "ops": [
                        {
                            "type": "SetCharacterState",
                            "character": "merchant",
                            "key": "trust",
                            "value": 3
                        },
                        {
                            "type": "SetPlayerState",
                            "key": "coins",
                            "value": 9
                        }
                    ]
                })),
            )),
            Ok(assistant_response(
                "{\"beats\":[{\"type\":\"Narrator\",\"purpose\":\"DescribeTransition\"},{\"type\":\"Actor\",\"speaker_id\":\"merchant\",\"purpose\":\"AdvanceGoal\"}]}",
                Some(json!({
                    "beats": [
                        {
                            "type": "Narrator",
                            "purpose": "DescribeTransition"
                        },
                        {
                            "type": "Actor",
                            "speaker_id": "merchant",
                            "purpose": "AdvanceGoal"
                        }
                    ]
                })),
            )),
            Ok(assistant_response(
                "{\"ops\":[{\"type\":\"SetState\",\"key\":\"gate_open\",\"value\":true}]}",
                Some(json!({
                    "ops": [
                        {
                            "type": "SetState",
                            "key": "gate_open",
                            "value": true
                        }
                    ]
                })),
            )),
        ],
        vec![
            Ok(vec![
                Ok(ChatChunk {
                    delta: "Water churned beneath the old gate.".to_owned(),
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
            ]),
            Ok(vec![
                Ok(ChatChunk {
                    delta: "<thought>I can still profit from this.</thought>".to_owned(),
                    model: Some("test-model".to_owned()),
                    finish_reason: None,
                    done: false,
                    usage: None,
                }),
                Ok(ChatChunk {
                    delta:
                        "<action>Haru lifts the lantern.</action><dialogue>Follow me.</dialogue>"
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
            ]),
        ],
    ));
    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        sample_runtime_state(),
    )
    .expect("engine");
    let mut stream = engine
        .run_turn_stream("Open the canal gate.")
        .await
        .expect("stream should start");

    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert!(matches!(events[0], EngineEvent::TurnStarted { .. }));
    assert!(matches!(events[1], EngineEvent::PlayerInputRecorded { .. }));
    assert!(matches!(
        events[2],
        EngineEvent::KeeperApplied {
            phase: agents::keeper::KeeperPhase::AfterPlayerInput,
            ..
        }
    ));
    assert!(matches!(events[3], EngineEvent::DirectorCompleted { .. }));
    assert!(matches!(events[4], EngineEvent::NarratorStarted { .. }));
    assert!(matches!(events[5], EngineEvent::NarratorTextDelta { .. }));
    assert!(matches!(events[6], EngineEvent::NarratorCompleted { .. }));
    assert!(matches!(events[7], EngineEvent::ActorStarted { .. }));
    assert!(matches!(events[8], EngineEvent::ActorThoughtDelta { .. }));
    assert!(matches!(events[9], EngineEvent::ActorActionComplete { .. }));
    assert!(matches!(events[10], EngineEvent::ActorDialogueDelta { .. }));
    assert!(matches!(events[11], EngineEvent::ActorCompleted { .. }));
    assert!(matches!(
        events[12],
        EngineEvent::KeeperApplied {
            phase: agents::keeper::KeeperPhase::AfterTurnOutputs,
            ..
        }
    ));

    let EngineEvent::TurnCompleted { result } =
        events.last().expect("final event should exist").clone()
    else {
        panic!("expected completed event");
    };
    drop(stream);

    assert_eq!(result.turn_index, 1);
    assert_eq!(result.director.current_node_id, "gate");
    assert_eq!(result.completed_beats.len(), 2);
    assert_eq!(result.snapshot.world_state.current_node, "gate");
    assert_eq!(
        result.snapshot.world_state.player_state("coins"),
        Some(&json!(9))
    );
    assert_eq!(
        result.snapshot.world_state.state("gate_open"),
        Some(&json!(true))
    );
    assert_eq!(
        result.snapshot.world_state.state("entered_gate"),
        Some(&json!(true))
    );
    assert_eq!(engine.runtime_state().turn_index(), 1);
    assert_eq!(engine.runtime_state().world_state().current_node(), "gate");
    assert_eq!(
        engine.runtime_state().world_state().state("gate_open"),
        Some(&json!(true))
    );

    let requests = llm.recorded_requests();
    let final_keeper_request = requests.last().expect("final keeper request");
    let final_keeper_user = user_message_content(final_keeper_request);
    assert!(final_keeper_user.contains("COMPLETED_BEATS"));
    assert!(final_keeper_user.contains("Follow me."));
    assert!(!final_keeper_user.contains("I can still profit from this."));
}

#[tokio::test]
async fn run_turn_returns_result_and_records_completed_beats() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response("{\"ops\":[]}", Some(json!({ "ops": [] })))),
            Ok(assistant_response(
                "{\"beats\":[{\"type\":\"Actor\",\"speaker_id\":\"merchant\",\"purpose\":\"AdvanceGoal\"}]}",
                Some(json!({
                    "beats": [
                        {
                            "type": "Actor",
                            "speaker_id": "merchant",
                            "purpose": "AdvanceGoal"
                        }
                    ]
                })),
            )),
            Ok(assistant_response("{\"ops\":[]}", Some(json!({ "ops": [] })))),
        ],
        vec![Ok(vec![
            Ok(ChatChunk {
                delta: "<thought>Keep calm.</thought><action>Haru steadies himself.</action><dialogue>We move now.</dialogue>".to_owned(),
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
        ])],
    ));
    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        sample_runtime_state(),
    )
    .expect("engine");

    let result = engine
        .run_turn("Stay close to the dock.")
        .await
        .expect("run_turn should succeed");

    assert_eq!(result.turn_index, 1);
    assert_eq!(result.completed_beats.len(), 1);
    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 4);
}

#[tokio::test]
async fn run_turn_transitions_using_player_state_conditions() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({ "ops": [] })),
            )),
            Ok(assistant_response(
                "{\"beats\":[]}",
                Some(json!({ "beats": [] })),
            )),
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({ "ops": [] })),
            )),
        ],
        Vec::new(),
    ));
    let mut runtime_state =
        sample_runtime_state_with_story_graph(sample_player_transition_story_graph());
    runtime_state
        .world_state_mut()
        .set_player_state("coins", json!(12));

    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        runtime_state,
    )
    .expect("engine");

    let rich_result = engine
        .run_turn("Show the permit.")
        .await
        .expect("run_turn should succeed");

    assert_eq!(rich_result.director.current_node_id, "vip_gate");

    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({ "ops": [] })),
            )),
            Ok(assistant_response(
                "{\"beats\":[]}",
                Some(json!({ "beats": [] })),
            )),
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({ "ops": [] })),
            )),
        ],
        Vec::new(),
    ));
    let mut runtime_state =
        sample_runtime_state_with_story_graph(sample_player_transition_story_graph());
    runtime_state
        .world_state_mut()
        .set_player_state("coins", json!(3));

    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm, "test-model"),
        runtime_state,
    )
    .expect("engine");

    let poor_result = engine
        .run_turn("Show the permit.")
        .await
        .expect("run_turn should succeed");

    assert_eq!(poor_result.director.current_node_id, "checkpoint");
}

#[tokio::test]
async fn run_turn_executes_mixed_beats_in_director_order() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response("{\"ops\":[]}", Some(json!({ "ops": [] })))),
            Ok(assistant_response(
                "{\"beats\":[{\"type\":\"Narrator\",\"purpose\":\"DescribeScene\"},{\"type\":\"Actor\",\"speaker_id\":\"merchant\",\"purpose\":\"AdvanceGoal\"},{\"type\":\"Narrator\",\"purpose\":\"DescribeResult\"},{\"type\":\"Actor\",\"speaker_id\":\"merchant\",\"purpose\":\"CommentOnScene\"}]}",
                Some(json!({
                    "beats": [
                        {
                            "type": "Narrator",
                            "purpose": "DescribeScene"
                        },
                        {
                            "type": "Actor",
                            "speaker_id": "merchant",
                            "purpose": "AdvanceGoal"
                        },
                        {
                            "type": "Narrator",
                            "purpose": "DescribeResult"
                        },
                        {
                            "type": "Actor",
                            "speaker_id": "merchant",
                            "purpose": "CommentOnScene"
                        }
                    ]
                })),
            )),
            Ok(assistant_response("{\"ops\":[]}", Some(json!({ "ops": [] })))),
        ],
        vec![
            Ok(vec![
                Ok(ChatChunk {
                    delta: "The dock groans under the floodwater.".to_owned(),
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
            ]),
            Ok(vec![
                Ok(ChatChunk {
                    delta: "<thought>The courier is listening.</thought><action>Haru points toward the gate.</action><dialogue>We should move before the tide rises.</dialogue>".to_owned(),
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
            ]),
            Ok(vec![
                Ok(ChatChunk {
                    delta: "The waterline creeps higher along the stone posts.".to_owned(),
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
            ]),
            Ok(vec![
                Ok(ChatChunk {
                    delta: "<action>Haru tightens his grip on the lantern.</action><dialogue>Say the word and I will lead.</dialogue>".to_owned(),
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
            ]),
        ],
    ));
    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        sample_runtime_state(),
    )
    .expect("engine");

    let result = engine
        .run_turn("Tell me what happens next.")
        .await
        .expect("run_turn should succeed");

    assert_eq!(result.completed_beats.len(), 4);
    assert!(matches!(
        result.completed_beats[0],
        ss_engine::ExecutedBeat::Narrator { .. }
    ));
    assert!(matches!(
        result.completed_beats[1],
        ss_engine::ExecutedBeat::Actor { .. }
    ));
    assert!(matches!(
        result.completed_beats[2],
        ss_engine::ExecutedBeat::Narrator { .. }
    ));
    assert!(matches!(
        result.completed_beats[3],
        ss_engine::ExecutedBeat::Actor { .. }
    ));

    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 7);
}

#[tokio::test]
async fn first_keeper_failure_preserves_recorded_player_input_and_emits_failure() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Err(LlmError::Provider {
            status: 500,
            message: "keeper down".to_owned(),
        })],
        Vec::new(),
    ));
    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        sample_runtime_state(),
    )
    .expect("engine");
    let mut stream = engine
        .run_turn_stream("Count my coins.")
        .await
        .expect("stream should start");

    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert!(matches!(events[0], EngineEvent::TurnStarted { .. }));
    assert!(matches!(events[1], EngineEvent::PlayerInputRecorded { .. }));
    let EngineEvent::TurnFailed { stage, error, .. } = events[2].clone() else {
        panic!("expected failure event");
    };
    drop(stream);

    assert_eq!(stage, EngineStage::KeeperAfterPlayerInput);
    assert!(error.contains("keeper down"));
    assert_eq!(engine.runtime_state().turn_index(), 0);
    assert_eq!(
        engine
            .runtime_state()
            .world_state()
            .actor_shared_history()
            .len(),
        1
    );
    assert_eq!(
        engine.runtime_state().world_state().actor_shared_history()[0].text,
        "Count my coins."
    );
}

#[tokio::test]
async fn invalid_actor_speaker_fails_after_preserving_prior_state_changes() {
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
                "{\"beats\":[{\"type\":\"Actor\",\"speaker_id\":\"guide\",\"purpose\":\"AdvanceGoal\"}]}",
                Some(json!({
                    "beats": [
                        {
                            "type": "Actor",
                            "speaker_id": "guide",
                            "purpose": "AdvanceGoal"
                        }
                    ]
                })),
            )),
        ],
        Vec::new(),
    ));
    let mut engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        sample_runtime_state(),
    )
    .expect("engine");

    let error = engine
        .run_turn("Go to the gate.")
        .await
        .expect_err("invalid speaker should fail");

    let EngineError::TurnFailed { stage, message } = error else {
        panic!("expected turn failure");
    };

    assert_eq!(stage, EngineStage::Actor);
    assert!(message.contains("guide"));
    assert_eq!(engine.runtime_state().turn_index(), 1);
    assert_eq!(engine.runtime_state().world_state().current_node(), "gate");
}

#[tokio::test]
async fn generate_story_graph_uses_architect_independently() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
            "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[{\"id\":\"dock\",\"title\":\"Flooded Dock\",\"scene\":\"A flooded dock at dusk.\",\"goal\":\"Decide whether to trust the merchant.\",\"characters\":[\"merchant\"],\"transitions\":[],\"on_enter_updates\":[]}]},\"world_state_schema\":{\"fields\":{\"entered_gate\":{\"value_type\":\"bool\",\"default\":false,\"description\":\"Whether the party has entered the gate\"}}},\"introduction\":\"The courier arrives at the flooded dock while the merchant watches from under a lantern.\"}",
            Some(json!({
                "graph": {
                    "start_node": "dock",
                    "nodes": [
                        {
                            "id": "dock",
                            "title": "Flooded Dock",
                            "scene": "A flooded dock at dusk.",
                            "goal": "Decide whether to trust the merchant.",
                            "characters": ["merchant"],
                            "transitions": [],
                            "on_enter_updates": []
                        }
                    ]
                },
                "world_state_schema": {
                    "fields": {
                        "entered_gate": {
                            "value_type": "bool",
                            "default": false,
                            "description": "Whether the party has entered the gate"
                        }
                    }
                },
                "introduction": "The courier arrives at the flooded dock while the merchant watches from under a lantern."
            })),
        ))],
        Vec::new(),
    ));
    let resources = sample_story_resources();

    let response = generate_story_graph(
        &StoryGenerationAgentConfigs::shared(llm.clone(), "test-model"),
        &resources,
    )
    .await
    .expect("architect wrapper should succeed");

    assert_eq!(response.graph.start_node, "dock");
    assert!(response.world_state_schema.has_field("entered_gate"));
    assert!(response.player_state_schema.has_field("coins"));
    assert_eq!(
        response.introduction,
        "The courier arrives at the flooded dock while the merchant watches from under a lantern."
    );
    let runtime_state = RuntimeState::from_story_resources(
        &resources,
        response.graph.clone(),
        sample_player_description(),
        response.player_state_schema.clone(),
    )
    .expect("runtime state should build from story resources");
    let engine = Engine::new(
        RuntimeAgentConfigs::shared(llm.clone(), "test-model"),
        runtime_state,
    )
    .expect("engine should build");
    assert_eq!(engine.runtime_state().story_id(), "flooded_city_demo");
    assert_eq!(
        engine.runtime_state().player_description(),
        sample_player_description()
    );
    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].max_tokens, Some(8_192));
    assert_eq!(requests[0].temperature, Some(0.0));
    assert!(user_message_content(&requests[0]).contains("STORY_CONCEPT"));
    assert!(user_message_content(&requests[0]).contains("PLAYER_STATE_SCHEMA_SEED"));
}

#[tokio::test]
async fn generate_story_plan_uses_planner_independently() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
            "Title:\nFlooded Dock Bargain\n\nOpening Situation:\nThe courier arrives at a flooded dock.",
            None,
        ))],
        Vec::new(),
    ));
    let resources = sample_story_resources();

    let response = generate_story_plan(
        &StoryGenerationAgentConfigs::shared(llm.clone(), "test-model"),
        &resources,
    )
    .await
    .expect("planner wrapper should succeed");

    assert!(response.story_script.contains("Title:"));
    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].max_tokens, None);
    assert_eq!(requests[0].temperature, None);
    assert!(user_message_content(&requests[0]).contains("AVAILABLE_CHARACTERS"));
}

#[tokio::test]
async fn generate_story_graph_passes_planned_story_when_present() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
            "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[{\"id\":\"dock\",\"title\":\"Flooded Dock\",\"scene\":\"A flooded dock at dusk.\",\"goal\":\"Decide whether to trust the merchant.\",\"characters\":[\"merchant\"],\"transitions\":[],\"on_enter_updates\":[]}]},\"world_state_schema\":{\"fields\":{}},\"introduction\":\"The courier arrives at the flooded dock.\"}",
            Some(json!({
                "graph": {
                    "start_node": "dock",
                    "nodes": [
                        {
                            "id": "dock",
                            "title": "Flooded Dock",
                            "scene": "A flooded dock at dusk.",
                            "goal": "Decide whether to trust the merchant.",
                            "characters": ["merchant"],
                            "transitions": [],
                            "on_enter_updates": []
                        }
                    ]
                },
                "world_state_schema": {
                    "fields": {}
                },
                "introduction": "The courier arrives at the flooded dock."
            })),
        ))],
        Vec::new(),
    ));
    let resources = sample_story_resources().with_planned_story(
        "Title:\nFlooded Dock Bargain\n\nOpening Situation:\nThe courier arrives at a flooded dock.",
    );

    let response = generate_story_graph(
        &StoryGenerationAgentConfigs::shared(llm.clone(), "test-model"),
        &resources,
    )
    .await
    .expect("architect wrapper should succeed");

    let requests = llm.recorded_requests();
    assert_eq!(requests.len(), 1);
    assert!(user_message_content(&requests[0]).contains("PLANNED_STORY"));
    assert!(user_message_content(&requests[0]).contains("PLAYER_STATE_SCHEMA_SEED"));
    assert!(user_message_content(&requests[0]).contains("Flooded Dock Bargain"));
    assert!(response.player_state_schema.has_field("coins"));
}
use std::sync::Arc;
