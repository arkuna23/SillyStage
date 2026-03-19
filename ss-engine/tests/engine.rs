mod common;

use std::collections::HashMap;

use futures_util::StreamExt;
use llm::{ChatChunk, LlmError, Role};
use serde_json::json;
use ss_engine::{
    AgentModelConfig, ArchitectModelConfig, Engine, EngineError, EngineEvent, EngineStage,
    PromptAgentKind, RuntimeAgentConfigs, RuntimeState, StoryGenerationAgentConfigs,
    StoryResources, compile_architect_prompt_profiles, compile_prompt_profile,
    default_agent_preset_config, generate_story_graph, generate_story_plan,
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

fn sample_eq_transition_story_graph() -> StoryGraph {
    StoryGraph::new(
        "crossroads",
        vec![
            NarrativeNode::new(
                "crossroads",
                "Crossroads",
                "Two flooded lanes split around a toppled shrine.",
                "Commit to one route.",
                vec!["merchant".to_owned()],
                vec![Transition::new(
                    "canal_gate",
                    Condition::new("route", ConditionOperator::Eq, json!("canal_path")),
                )],
                vec![],
            ),
            NarrativeNode::new(
                "canal_gate",
                "Canal Gate",
                "The courier reaches the canal gate.",
                "Continue through the gate.",
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

fn runtime_agent_configs(llm: Arc<QueuedMockLlm>) -> RuntimeAgentConfigs {
    let llm_api: Arc<dyn llm::LlmApi> = llm;
    let profile = |agent| {
        compile_prompt_profile(agent, &default_agent_preset_config(agent))
            .expect("default prompt profile should compile")
    };

    RuntimeAgentConfigs {
        director: AgentModelConfig::new(Arc::clone(&llm_api), "test-model")
            .with_prompt_profile(profile(PromptAgentKind::Director)),
        actor: AgentModelConfig::new(Arc::clone(&llm_api), "test-model")
            .with_prompt_profile(profile(PromptAgentKind::Actor)),
        narrator: AgentModelConfig::new(Arc::clone(&llm_api), "test-model")
            .with_prompt_profile(profile(PromptAgentKind::Narrator)),
        keeper: AgentModelConfig::new(llm_api, "test-model")
            .with_prompt_profile(profile(PromptAgentKind::Keeper)),
    }
}

fn story_generation_agent_configs(llm: Arc<QueuedMockLlm>) -> StoryGenerationAgentConfigs {
    let llm_api: Arc<dyn llm::LlmApi> = llm;
    let planner_profile = compile_prompt_profile(
        PromptAgentKind::Planner,
        &default_agent_preset_config(PromptAgentKind::Planner),
    )
    .expect("default planner prompt profile should compile");
    let architect_profiles =
        compile_architect_prompt_profiles(&default_agent_preset_config(PromptAgentKind::Architect))
            .expect("default architect prompt profile should compile");

    StoryGenerationAgentConfigs {
        planner: AgentModelConfig::new(Arc::clone(&llm_api), "test-model")
            .with_prompt_profile(planner_profile),
        architect: ArchitectModelConfig::new(llm_api, "test-model")
            .with_max_tokens(Some(8_192))
            .with_prompt_profiles(architect_profiles),
    }
}

#[test]
fn default_runtime_prompt_profiles_include_output_contracts() {
    let director = compile_prompt_profile(
        PromptAgentKind::Director,
        &default_agent_preset_config(PromptAgentKind::Director),
    )
    .expect("default director prompt profile should compile");
    assert!(director.system_prompt.contains("ResponsePlan schema:"));
    assert!(
        director
            .system_prompt
            .contains("\"role_actions\": [SessionCharacterAction]")
    );
    assert!(director.system_prompt.contains("\"type\":\"Narrator\""));
    assert!(
        director
            .system_prompt
            .contains("Every Actor beat speaker_id must be either a CURRENT_CAST id")
    );

    let keeper = compile_prompt_profile(
        PromptAgentKind::Keeper,
        &default_agent_preset_config(PromptAgentKind::Keeper),
    )
    .expect("default keeper prompt profile should compile");
    assert!(keeper.system_prompt.contains("StateUpdate schema:"));
    assert!(
        keeper
            .system_prompt
            .contains("\"type\":\"SetCharacterState\"")
    );
    assert!(keeper.system_prompt.contains("Always include \"ops\""));
    assert!(
        keeper
            .system_prompt
            .contains("Never introduce a brand-new character id in active-character ops")
    );

    let replyer = compile_prompt_profile(
        PromptAgentKind::Replyer,
        &default_agent_preset_config(PromptAgentKind::Replyer),
    )
    .expect("default replyer prompt profile should compile");
    assert!(
        replyer.system_prompt.contains(
            "Suggest several player reply options that fit the current state of the scene"
        )
    );
    assert!(
        replyer
            .system_prompt
            .contains("Offer concise, distinct reply options grounded in the visible conversation history and current world state")
    );
    assert!(
        replyer
            .system_prompt
            .contains("Let the options vary naturally in tone, intent, and commitment level")
    );
    assert!(replyer.system_prompt.contains("Reply suggestion schema:"));
    assert!(replyer.system_prompt.contains("\"replies\": ["));

    let actor = compile_prompt_profile(
        PromptAgentKind::Actor,
        &default_agent_preset_config(PromptAgentKind::Actor),
    )
    .expect("default actor prompt profile should compile");
    assert!(actor.system_prompt.contains("Allowed tags:"));
    assert!(actor.system_prompt.contains("<thought>...</thought>"));
    assert!(actor.system_prompt.contains("Tags may appear in any order"));
}

#[test]
fn default_architect_prompt_profiles_include_output_schemas() {
    let profiles =
        compile_architect_prompt_profiles(&default_agent_preset_config(PromptAgentKind::Architect))
            .expect("default architect prompt profiles should compile");

    assert!(
        profiles
            .graph
            .system_prompt
            .contains("Common nested schemas:")
    );
    assert!(profiles.graph.system_prompt.contains("\"graph\": {"));
    assert!(
        profiles
            .graph
            .system_prompt
            .contains("\"introduction\": \"short player-facing opening paragraph\"")
    );
    assert!(
        profiles
            .graph
            .system_prompt
            .contains("If \"enum_values\" is present, omit \"default\" or make \"default\" exactly equal to one item from \"enum_values\"")
    );
    assert!(
        profiles
            .graph
            .system_prompt
            .contains("Use only the exact StateOp type names listed above")
    );
    assert!(
        profiles
            .graph
            .system_prompt
            .contains("Every returned NarrativeNode id must be unique within the current response")
    );
    assert!(
        profiles
            .graph
            .system_prompt
            .contains("Use only AVAILABLE_CHARACTERS ids in NarrativeNode.characters")
    );

    assert!(
        profiles
            .draft_init
            .system_prompt
            .contains("\"transition_patches\": [NodeTransitionPatch]")
    );
    assert!(
        profiles
            .draft_init
            .system_prompt
            .contains("\"start_node\": \"node_id\"")
    );
    assert!(
        profiles
            .draft_init
            .system_prompt
            .contains("Returned node ids must all be new and unique within this response")
    );
    assert!(
        profiles.draft_init.system_prompt.contains(
            "All transition and transition_patches targets must use returned node ids only"
        )
    );
    assert!(profiles.draft_init.system_prompt.contains(
        "Do not point to future chunk nodes; add those links later with transition_patches"
    ));

    assert!(
        profiles
            .draft_continue
            .system_prompt
            .contains("\"section_summary\": \"one short sentence\"")
    );
    assert!(
        profiles
            .draft_continue
            .system_prompt
            .contains("must not reuse node ids that already exist in GRAPH_SUMMARY")
    );
    assert!(
        profiles
            .draft_continue
            .system_prompt
            .contains("All transition and transition_patches targets must use either GRAPH_SUMMARY node ids or returned node ids")
    );
    assert!(profiles.draft_continue.system_prompt.contains(
        "Do not point to future chunk nodes; add those links later with transition_patches"
    ));
    assert!(
        profiles
            .repair_system_prompt
            .contains("omit default or make default exactly match one enum_values item")
    );
    assert!(
        profiles
            .repair_system_prompt
            .contains("do not return any node whose id already exists in GRAPH_SUMMARY")
    );
    assert!(
        profiles
            .repair_system_prompt
            .contains("transition and transition_patches targets must use GRAPH_SUMMARY node ids or returned node ids only")
    );
    assert!(profiles.repair_system_prompt.contains(
        "Remove links to future chunk nodes; later chunks can add them via transition_patches"
    ));
}

fn keeper_update_for_phase(
    events: &[EngineEvent],
    phase: agents::keeper::KeeperPhase,
) -> state::StateUpdate {
    events
        .iter()
        .find_map(|event| match event {
            EngineEvent::KeeperApplied {
                phase: event_phase,
                update,
                ..
            } if *event_phase == phase => Some(update.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing keeper update for phase {phase:?}"))
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
    let mut engine =
        Engine::new(runtime_agent_configs(llm.clone()), sample_runtime_state()).expect("engine");
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
    assert!(final_keeper_user.contains("NODE_CHANGE"));
    assert!(final_keeper_user.contains("PROGRESSION_HINTS"));
    assert!(final_keeper_user.contains("matched_transition_hints"));
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
    let mut engine =
        Engine::new(runtime_agent_configs(llm.clone()), sample_runtime_state()).expect("engine");

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

    let mut engine =
        Engine::new(runtime_agent_configs(llm.clone()), runtime_state).expect("engine");

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

    let mut engine = Engine::new(runtime_agent_configs(llm), runtime_state).expect("engine");

    let poor_result = engine
        .run_turn("Show the permit.")
        .await
        .expect("run_turn should succeed");

    assert_eq!(poor_result.director.current_node_id, "checkpoint");
}

#[tokio::test]
async fn second_keeper_fallback_restates_simple_eq_transition_fact() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[{\"type\":\"SetState\",\"key\":\"route\",\"value\":\"canal_path\"}]}",
                Some(json!({
                    "ops": [
                        {
                            "type": "SetState",
                            "key": "route",
                            "value": "canal_path"
                        }
                    ]
                })),
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
    let mut engine = Engine::new(
        runtime_agent_configs(llm),
        sample_runtime_state_with_story_graph(sample_eq_transition_story_graph()),
    )
    .expect("engine");

    let mut stream = engine
        .run_turn_stream("Take the canal path.")
        .await
        .expect("stream should start");
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    let after_turn_update =
        keeper_update_for_phase(&events, agents::keeper::KeeperPhase::AfterTurnOutputs);
    assert_eq!(after_turn_update.ops.len(), 1);
    assert!(matches!(
        &after_turn_update.ops[0],
        state::StateOp::SetState { key, value }
            if key == "route" && value == &json!("canal_path")
    ));

    let EngineEvent::TurnCompleted { result } = events.last().expect("completed event").clone()
    else {
        panic!("expected completed event");
    };
    assert_eq!(result.director.current_node_id, "canal_gate");
}

#[tokio::test]
async fn second_keeper_fallback_skips_non_eq_transition_fact() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[{\"type\":\"SetPlayerState\",\"key\":\"coins\",\"value\":12}]}",
                Some(json!({
                    "ops": [
                        {
                            "type": "SetPlayerState",
                            "key": "coins",
                            "value": 12
                        }
                    ]
                })),
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
    let mut engine = Engine::new(
        runtime_agent_configs(llm),
        sample_runtime_state_with_story_graph(sample_player_transition_story_graph()),
    )
    .expect("engine");

    let mut stream = engine
        .run_turn_stream("Show the permit.")
        .await
        .expect("stream should start");
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    let after_turn_update =
        keeper_update_for_phase(&events, agents::keeper::KeeperPhase::AfterTurnOutputs);
    assert!(after_turn_update.ops.is_empty());

    let EngineEvent::TurnCompleted { result } = events.last().expect("completed event").clone()
    else {
        panic!("expected completed event");
    };
    assert_eq!(result.director.current_node_id, "vip_gate");
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
    let mut engine =
        Engine::new(runtime_agent_configs(llm.clone()), sample_runtime_state()).expect("engine");

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
    let mut engine =
        Engine::new(runtime_agent_configs(llm.clone()), sample_runtime_state()).expect("engine");
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
    let mut engine =
        Engine::new(runtime_agent_configs(llm.clone()), sample_runtime_state()).expect("engine");

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

    let response = generate_story_graph(&story_generation_agent_configs(llm.clone()), &resources)
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
    let engine = Engine::new(runtime_agent_configs(llm.clone()), runtime_state)
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

    let response = generate_story_plan(&story_generation_agent_configs(llm.clone()), &resources)
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

    let response = generate_story_graph(&story_generation_agent_configs(llm.clone()), &resources)
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
