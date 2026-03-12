mod common;

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use engine::{AgentApiIdOverrides, AgentApiIds, LlmApiRegistry, SessionConfigMode};
use futures_util::StreamExt;
use protocol::{
    ConfigUpdateGlobalParams, CreateStoryResourcesParams, JsonRpcOutcome, JsonRpcRequestMessage,
    RequestParams, ResponseResult, RunTurnParams, SessionUpdateConfigParams,
    StartSessionFromStoryParams, StoryGeneratedPayload, StoryResourcesPayload, StreamFrame,
    UploadChunkParams, UploadCompleteParams, UploadInitParams, UploadTargetKind,
};
use serde_json::json;
use ss_handler::{Handler, HandlerReply, HandlerStore, InMemoryHandlerStore, StoryRecord};

use common::{
    QueuedMockLlm, assistant_response, sample_archive, sample_character_record,
    sample_player_state_schema, sample_story_graph, sample_world_state_schema,
};

fn default_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner-default".to_owned(),
        architect_api_id: "architect-default".to_owned(),
        director_api_id: "director-default".to_owned(),
        actor_api_id: "actor-default".to_owned(),
        narrator_api_id: "narrator-default".to_owned(),
        keeper_api_id: "keeper-default".to_owned(),
    }
}

fn alternate_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner-alt".to_owned(),
        architect_api_id: "architect-alt".to_owned(),
        director_api_id: "director-alt".to_owned(),
        actor_api_id: "actor-alt".to_owned(),
        narrator_api_id: "narrator-alt".to_owned(),
        keeper_api_id: "keeper-alt".to_owned(),
    }
}

fn registry_with_ids<'a>(llm: &'a QueuedMockLlm) -> LlmApiRegistry<'a> {
    let default = default_api_ids();
    let alternate = alternate_api_ids();

    LlmApiRegistry::new()
        .register(default.planner_api_id, llm, "planner-model")
        .register(default.architect_api_id, llm, "architect-model")
        .register(default.director_api_id, llm, "director-model")
        .register(default.actor_api_id, llm, "actor-model")
        .register(default.narrator_api_id, llm, "narrator-model")
        .register(default.keeper_api_id, llm, "keeper-model")
        .register(alternate.planner_api_id, llm, "planner-alt-model")
        .register(alternate.architect_api_id, llm, "architect-alt-model")
        .register(alternate.director_api_id, llm, "director-alt-model")
        .register(alternate.actor_api_id, llm, "actor-alt-model")
        .register(alternate.narrator_api_id, llm, "narrator-alt-model")
        .register(alternate.keeper_api_id, llm, "keeper-alt-model")
}

fn unary_result(reply: HandlerReply<'_>) -> ResponseResult {
    match reply {
        HandlerReply::Unary(response) => match response.outcome {
            JsonRpcOutcome::Ok(result) => *result,
            JsonRpcOutcome::Err(error) => panic!("unexpected error response: {}", error.message),
        },
        HandlerReply::Stream { .. } => panic!("expected unary reply"),
    }
}

#[tokio::test]
async fn upload_character_card_and_create_resources_via_character_id() {
    let llm = QueuedMockLlm::new(vec![], vec![]);
    let handler = Handler::with_in_memory_store(registry_with_ids(&llm), default_api_ids())
        .await
        .expect("handler should build");
    let archive_bytes = sample_archive()
        .to_chr_bytes()
        .expect("archive should serialize");

    let upload_init = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-1",
                None::<String>,
                RequestParams::UploadInit(UploadInitParams {
                    target_kind: UploadTargetKind::CharacterCard,
                    file_name: "merchant.chr".to_owned(),
                    content_type: "application/x-sillystage-character-card".to_owned(),
                    total_size: archive_bytes.len() as u64,
                    sha256: "demo-sha".to_owned(),
                }),
            ))
            .await,
    );
    let upload_id = match upload_init {
        ResponseResult::UploadInitialized(payload) => payload.upload_id,
        other => panic!("unexpected response: {other:?}"),
    };

    let chunk = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-2",
                None::<String>,
                RequestParams::UploadChunk(UploadChunkParams {
                    upload_id: upload_id.clone(),
                    chunk_index: 0,
                    offset: 0,
                    payload_base64: BASE64_STANDARD.encode(&archive_bytes),
                    is_last: true,
                }),
            ))
            .await,
    );
    assert!(matches!(chunk, ResponseResult::UploadChunkAccepted(_)));

    let uploaded = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-3",
                None::<String>,
                RequestParams::UploadComplete(UploadCompleteParams { upload_id }),
            ))
            .await,
    );
    let character_id = match uploaded {
        ResponseResult::CharacterCardUploaded(payload) => payload.character_id,
        other => panic!("unexpected response: {other:?}"),
    };

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-4",
                None::<String>,
                RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
                    story_concept: "A flooded harbor story.".to_owned(),
                    character_ids: vec![character_id.clone()],
                    player_state_schema_seed: sample_player_state_schema(),
                    world_state_schema_seed: Some(sample_world_state_schema()),
                    planned_story: Some(
                        "Opening Situation:\nA courier arrives at dusk.".to_owned(),
                    ),
                }),
            ))
            .await,
    );

    match created {
        ResponseResult::StoryResourcesCreated(payload) => {
            assert_eq!(payload.character_ids, vec![character_id]);
            assert_eq!(payload.story_concept, "A flooded harbor story.");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn session_config_can_switch_between_session_and_global_modes() {
    let llm = QueuedMockLlm::new(vec![], vec![]);
    let store = Arc::new(InMemoryHandlerStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(&llm), default_api_ids())
        .await
        .expect("handler should build");
    let character = sample_character_record();
    let resource = StoryResourcesPayload {
        resource_id: "resource-1".to_owned(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec![character.character_id.clone()],
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
        planned_story: None,
    };
    let story = StoryRecord {
        story_id: "story-1".to_owned(),
        resource_id: resource.resource_id.clone(),
        generated: StoryGeneratedPayload {
            resource_id: resource.resource_id.clone(),
            story_id: "story-1".to_owned(),
            graph: sample_story_graph(),
            world_state_schema: sample_world_state_schema(),
            player_state_schema: sample_player_state_schema(),
            introduction: "The courier reaches a flooded dock.".to_owned(),
        },
    };

    store
        .save_character(character)
        .await
        .expect("save character");
    store
        .save_story_resources(resource)
        .await
        .expect("save resources");
    store.save_story(story).await.expect("save story");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-1",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                player_description: "A determined courier.".to_owned(),
                config_mode: SessionConfigMode::UseSession,
                session_api_ids: Some(alternate_api_ids()),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary session start"),
    };

    let session_id = started
        .session_id
        .clone()
        .expect("session id should be assigned");
    let config = match started.outcome {
        JsonRpcOutcome::Ok(result) => match *result {
            ResponseResult::SessionStarted(payload) => payload.config.clone(),
            other => panic!("unexpected response: {other:?}"),
        },
        JsonRpcOutcome::Err(error) => panic!("unexpected error response: {}", error.message),
    };
    assert_eq!(config.mode, SessionConfigMode::UseSession);
    assert_eq!(config.effective_api_ids, alternate_api_ids());

    let updated_global = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-2",
                None::<String>,
                RequestParams::ConfigUpdateGlobal(ConfigUpdateGlobalParams {
                    api_overrides: AgentApiIdOverrides {
                        actor_api_id: Some("actor-alt".to_owned()),
                        ..AgentApiIdOverrides::default()
                    },
                }),
            ))
            .await,
    );
    assert!(matches!(updated_global, ResponseResult::GlobalConfig(_)));

    let session_after_global_change = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-3",
                Some(session_id.clone()),
                RequestParams::SessionGetConfig(protocol::SessionGetConfigParams::default()),
            ))
            .await,
    );
    match session_after_global_change {
        ResponseResult::SessionConfig(payload) => {
            assert_eq!(payload.mode, SessionConfigMode::UseSession);
            assert_eq!(payload.effective_api_ids, alternate_api_ids());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let session_use_global = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-4",
                Some(session_id.clone()),
                RequestParams::SessionUpdateConfig(SessionUpdateConfigParams {
                    mode: SessionConfigMode::UseGlobal,
                    session_api_ids: None,
                    api_overrides: None,
                }),
            ))
            .await,
    );
    match session_use_global {
        ResponseResult::SessionConfig(payload) => {
            assert_eq!(payload.mode, SessionConfigMode::UseGlobal);
            assert_eq!(payload.effective_api_ids.actor_api_id, "actor-alt");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn run_turn_stream_emits_started_and_persists_session_snapshot() {
    let llm = QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[{\"type\":\"SetPlayerState\",\"key\":\"coins\",\"value\":5}]}",
                Some(json!({
                    "ops": [
                        {
                            "type": "SetPlayerState",
                            "key": "coins",
                            "value": 5
                        }
                    ]
                })),
            )),
            Ok(assistant_response(
                "{\"beats\":[{\"type\":\"Narrator\",\"purpose\":\"DescribeScene\"}]}",
                Some(json!({
                    "beats": [
                        {
                            "type": "Narrator",
                            "purpose": "DescribeScene"
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
        vec![Ok(vec![
            Ok(llm::ChatChunk {
                delta: "Water churned beneath the dock.".to_owned(),
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
    let store = Arc::new(InMemoryHandlerStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(&llm), default_api_ids())
        .await
        .expect("handler should build");
    let character = sample_character_record();
    let resource = StoryResourcesPayload {
        resource_id: "resource-1".to_owned(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec![character.character_id.clone()],
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
        planned_story: None,
    };
    let story = StoryRecord {
        story_id: "story-1".to_owned(),
        resource_id: resource.resource_id.clone(),
        generated: StoryGeneratedPayload {
            resource_id: resource.resource_id.clone(),
            story_id: "story-1".to_owned(),
            graph: sample_story_graph(),
            world_state_schema: sample_world_state_schema(),
            player_state_schema: sample_player_state_schema(),
            introduction: "The courier reaches a flooded dock.".to_owned(),
        },
    };

    store
        .save_character(character)
        .await
        .expect("save character");
    store
        .save_story_resources(resource)
        .await
        .expect("save resources");
    store.save_story(story).await.expect("save story");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-1",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                player_description: "A determined courier.".to_owned(),
                config_mode: SessionConfigMode::UseGlobal,
                session_api_ids: None,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary session start"),
    };
    let session_id = started.session_id.expect("session id should exist");

    let reply = handler
        .handle(JsonRpcRequestMessage::new(
            "req-2",
            Some(session_id.clone()),
            RequestParams::SessionRunTurn(RunTurnParams {
                player_input: "Open the gate.".to_owned(),
                api_overrides: None,
            }),
        ))
        .await;

    let (ack, events) = match reply {
        HandlerReply::Stream { ack, events } => (ack, events),
        HandlerReply::Unary(_) => panic!("expected stream reply"),
    };
    assert!(matches!(
        ack.outcome,
        JsonRpcOutcome::Ok(result) if matches!(*result, ResponseResult::TurnStreamAccepted(_))
    ));

    let frames = events.collect::<Vec<_>>().await;
    assert!(matches!(frames[0].frame, StreamFrame::Started));
    assert!(matches!(
        frames.last().expect("final frame").frame,
        StreamFrame::Completed { .. }
    ));

    let session = store
        .get_session(&session_id)
        .await
        .expect("load session")
        .expect("session should exist");
    assert_eq!(session.snapshot.turn_index, 1);
    assert_eq!(
        session.snapshot.world_state.player_state("coins"),
        Some(&json!(5))
    );
    assert_eq!(
        session.snapshot.world_state.state("gate_open"),
        Some(&json!(true))
    );
}
