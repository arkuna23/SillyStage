mod common;

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use engine::{AgentApiIdOverrides, AgentApiIds, LlmApiRegistry, SessionConfigMode};
use futures_util::StreamExt;
use protocol::{
    CharacterArchive, CharacterCardContent, CharacterCoverMimeType, CharacterCreateParams,
    CharacterExportChrParams, CharacterGetCoverParams, CharacterGetParams,
    CharacterSetCoverParams, ConfigUpdateGlobalParams, CreateStoryResourcesParams,
    DeleteSessionParams, ErrorCode, GenerateStoryParams, GetSessionParams,
    GetStoryResourcesParams, JsonRpcOutcome, JsonRpcRequestMessage, RequestParams,
    ResponseResult, RunTurnParams, SessionUpdateConfigParams, StartSessionFromStoryParams,
    StreamFrame, UpdatePlayerDescriptionParams, UploadChunkParams, UploadCompleteParams,
    UploadInitParams, UploadTargetKind,
};
use serde_json::json;
use ss_handler::{Handler, HandlerReply};
use store::{InMemoryStore, Store, StoryRecord, StoryResourcesRecord};

use common::{
    QueuedMockLlm, assistant_response, sample_archive, sample_character_record,
    sample_player_state_schema, sample_story_graph, sample_story_payload,
    sample_world_state_schema,
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

fn registry_with_ids(llm: Arc<QueuedMockLlm>) -> LlmApiRegistry {
    let default = default_api_ids();
    let alternate = alternate_api_ids();
    let llm: Arc<dyn llm::LlmApi> = llm;

    LlmApiRegistry::new()
        .register(default.planner_api_id, Arc::clone(&llm), "planner-model")
        .register(default.architect_api_id, Arc::clone(&llm), "architect-model")
        .register(default.director_api_id, Arc::clone(&llm), "director-model")
        .register(default.actor_api_id, Arc::clone(&llm), "actor-model")
        .register(default.narrator_api_id, Arc::clone(&llm), "narrator-model")
        .register(default.keeper_api_id, Arc::clone(&llm), "keeper-model")
        .register(alternate.planner_api_id, Arc::clone(&llm), "planner-alt-model")
        .register(alternate.architect_api_id, Arc::clone(&llm), "architect-alt-model")
        .register(alternate.director_api_id, Arc::clone(&llm), "director-alt-model")
        .register(alternate.actor_api_id, Arc::clone(&llm), "actor-alt-model")
        .register(alternate.narrator_api_id, Arc::clone(&llm), "narrator-alt-model")
        .register(alternate.keeper_api_id, llm, "keeper-alt-model")
}

fn unary_result(reply: HandlerReply) -> ResponseResult {
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
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Handler::with_in_memory_store(registry_with_ids(llm.clone()), default_api_ids())
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

    unary_result(
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

    let got_character = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-3a",
                None::<String>,
                RequestParams::CharacterGet(CharacterGetParams {
                    character_id: character_id.clone(),
                }),
            ))
            .await,
    );
    match got_character {
        ResponseResult::Character(payload) => {
            assert_eq!(payload.character_id, character_id);
            assert_eq!(payload.content.name, "Haru");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let got_cover = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-3b",
                None::<String>,
                RequestParams::CharacterGetCover(CharacterGetCoverParams {
                    character_id: character_id.clone(),
                }),
            ))
            .await,
    );
    match got_cover {
        ResponseResult::CharacterCover(payload) => {
            assert_eq!(payload.character_id, character_id);
            assert_eq!(payload.cover_file_name, "cover.png");
            assert_eq!(
                BASE64_STANDARD
                    .decode(payload.cover_base64)
                    .expect("cover should decode"),
                b"cover-bytes"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let exported_chr = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-3c",
                None::<String>,
                RequestParams::CharacterExportChr(CharacterExportChrParams {
                    character_id: character_id.clone(),
                }),
            ))
            .await,
    );
    match exported_chr {
        ResponseResult::CharacterChrExport(payload) => {
            assert_eq!(payload.character_id, character_id);
            assert_eq!(payload.file_name, "merchant.chr");
            assert_eq!(
                payload.content_type,
                "application/x-sillystage-character-card"
            );

            let chr_bytes = BASE64_STANDARD
                .decode(payload.chr_base64)
                .expect("chr should decode");
            let archive =
                CharacterArchive::from_chr_bytes(&chr_bytes).expect("chr archive should decode");
            assert_eq!(archive.content.id, "merchant");
            assert_eq!(archive.manifest.cover_path, "cover.png");
            assert_eq!(archive.cover_bytes, b"cover-bytes");
        }
        other => panic!("unexpected response: {other:?}"),
    }

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

    let resource_id = match created {
        ResponseResult::StoryResourcesCreated(payload) => {
            assert_eq!(payload.character_ids, vec![character_id]);
            assert_eq!(payload.story_concept, "A flooded harbor story.");
            payload.resource_id.clone()
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let got_resources = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-5",
                None::<String>,
                RequestParams::StoryResourcesGet(GetStoryResourcesParams { resource_id }),
            ))
            .await,
    );

    assert!(matches!(got_resources, ResponseResult::StoryResources(_)));
}

#[tokio::test]
async fn character_create_then_set_cover_enables_cover_and_chr_export() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Handler::with_in_memory_store(registry_with_ids(llm), default_api_ids())
        .await
        .expect("handler should build");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-create",
                None::<String>,
                RequestParams::CharacterCreate(CharacterCreateParams {
                    content: CharacterCardContent {
                        id: "merchant".to_owned(),
                        name: "Haru".to_owned(),
                        personality: "greedy but friendly trader".to_owned(),
                        style: "talkative, casual".to_owned(),
                        tendencies: vec!["likes profitable deals".to_owned()],
                        state_schema: Default::default(),
                        system_prompt: "Stay in character.".to_owned(),
                    },
                }),
            ))
            .await,
    );
    match created {
        ResponseResult::CharacterCreated(payload) => {
            assert_eq!(payload.character_id, "merchant");
            assert!(payload.character_summary.cover_file_name.is_none());
            assert!(payload.character_summary.cover_mime_type.is_none());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let created_character = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-get",
                None::<String>,
                RequestParams::CharacterGet(CharacterGetParams {
                    character_id: "merchant".to_owned(),
                }),
            ))
            .await,
    );
    match created_character {
        ResponseResult::Character(payload) => {
            assert!(payload.cover_file_name.is_none());
            assert!(payload.cover_mime_type.is_none());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let missing_cover_response = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-get-cover",
            None::<String>,
            RequestParams::CharacterGetCover(CharacterGetCoverParams {
                character_id: "merchant".to_owned(),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };
    assert!(matches!(
        missing_cover_response.outcome,
        JsonRpcOutcome::Err(error) if error.code == ErrorCode::Conflict.rpc_code()
    ));

    let missing_chr_response = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-export",
            None::<String>,
            RequestParams::CharacterExportChr(CharacterExportChrParams {
                character_id: "merchant".to_owned(),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };
    assert!(matches!(
        missing_chr_response.outcome,
        JsonRpcOutcome::Err(error) if error.code == ErrorCode::Conflict.rpc_code()
    ));

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-set-cover",
                None::<String>,
                RequestParams::CharacterSetCover(CharacterSetCoverParams {
                    character_id: "merchant".to_owned(),
                    cover_mime_type: CharacterCoverMimeType::Png,
                    cover_base64: BASE64_STANDARD.encode(b"cover-bytes"),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::CharacterCoverUpdated(payload) => {
            assert_eq!(payload.character_id, "merchant");
            assert_eq!(payload.cover_file_name, "cover.png");
            assert_eq!(payload.cover_mime_type, CharacterCoverMimeType::Png);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let got_cover = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-get-cover-success",
                None::<String>,
                RequestParams::CharacterGetCover(CharacterGetCoverParams {
                    character_id: "merchant".to_owned(),
                }),
            ))
            .await,
    );
    match got_cover {
        ResponseResult::CharacterCover(payload) => {
            assert_eq!(
                BASE64_STANDARD
                    .decode(payload.cover_base64)
                    .expect("cover should decode"),
                b"cover-bytes"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let exported_chr = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-export-success",
                None::<String>,
                RequestParams::CharacterExportChr(CharacterExportChrParams {
                    character_id: "merchant".to_owned(),
                }),
            ))
            .await,
    );
    match exported_chr {
        ResponseResult::CharacterChrExport(payload) => {
            let chr_bytes = BASE64_STANDARD
                .decode(payload.chr_base64)
                .expect("chr should decode");
            let archive =
                CharacterArchive::from_chr_bytes(&chr_bytes).expect("chr archive should decode");
            assert_eq!(archive.content.id, "merchant");
            assert_eq!(archive.manifest.cover_path, "cover.png");
            assert_eq!(archive.cover_bytes, b"cover-bytes");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn story_and_session_crud_follow_store_objects() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
            "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{}},\"player_state_schema\":{\"fields\":{}},\"introduction\":\"At the dock.\"}",
            Some(json!({
                "graph": sample_story_graph(),
                "world_state_schema": sample_world_state_schema(),
                "player_state_schema": sample_player_state_schema(),
                "introduction": "At the dock."
            })),
        ))],
        vec![],
    ));
    let store = Arc::new(InMemoryStore::new());
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");

    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()), default_api_ids())
        .await
        .expect("handler should build");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-1",
                None::<String>,
                RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
                    story_concept: "A flooded harbor story.".to_owned(),
                    character_ids: vec!["merchant".to_owned()],
                    player_state_schema_seed: sample_player_state_schema(),
                    world_state_schema_seed: Some(sample_world_state_schema()),
                    planned_story: None,
                }),
            ))
            .await,
    );
    let resource_id = match created {
        ResponseResult::StoryResourcesCreated(payload) => payload.resource_id.clone(),
        other => panic!("unexpected response: {other:?}"),
    };

    let generated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-2",
                None::<String>,
                RequestParams::StoryGenerate(GenerateStoryParams {
                    resource_id,
                    display_name: Some("Flooded Harbor".to_owned()),
                    architect_api_id: None,
                }),
            ))
            .await,
    );
    let story_id = match generated {
        ResponseResult::StoryGenerated(payload) => {
            assert_eq!(payload.display_name, "Flooded Harbor");
            payload.story_id.clone()
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-3",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: story_id.clone(),
                display_name: Some("Courier Run".to_owned()),
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

    let session_id = started.session_id.clone().expect("session id should exist");
    match started.outcome {
        JsonRpcOutcome::Ok(result) => match *result {
            ResponseResult::SessionStarted(payload) => {
                assert_eq!(payload.story_id, story_id);
                assert_eq!(payload.display_name, "Courier Run");
            }
            other => panic!("unexpected response: {other:?}"),
        },
        JsonRpcOutcome::Err(error) => panic!("unexpected error response: {}", error.message),
    }

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-4",
                Some(session_id.clone()),
                RequestParams::SessionGet(GetSessionParams::default()),
            ))
            .await,
    );
    match fetched {
        ResponseResult::Session(payload) => {
            assert_eq!(payload.session_id, session_id);
            assert_eq!(payload.display_name, "Courier Run");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let deleted = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-5",
                Some(session_id.clone()),
                RequestParams::SessionDelete(DeleteSessionParams::default()),
            ))
            .await,
    );
    assert!(matches!(
        deleted,
        ResponseResult::SessionDeleted(payload) if payload.session_id == session_id
    ));
}

#[tokio::test]
async fn session_config_can_switch_between_session_and_global_modes() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()), default_api_ids())
        .await
        .expect("handler should build");

    let character = sample_character_record();
    let resource = StoryResourcesRecord {
        resource_id: "resource-1".to_owned(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec![character.character_id.clone()],
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
        planned_story: None,
    };
    let story = StoryRecord {
        story_id: "story-1".to_owned(),
        display_name: "Flooded Harbor".to_owned(),
        resource_id: resource.resource_id.clone(),
        graph: sample_story_graph(),
        world_state_schema: sample_world_state_schema(),
        player_state_schema: sample_player_state_schema(),
        introduction: "The courier reaches a flooded dock.".to_owned(),
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
                display_name: None,
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

    unary_result(
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
    let llm = Arc::new(QueuedMockLlm::new(
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
    ));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()), default_api_ids())
        .await
        .expect("handler should build");
    let character = sample_character_record();
    let resource = StoryResourcesRecord {
        resource_id: "resource-1".to_owned(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec![character.character_id.clone()],
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
        planned_story: None,
    };
    let story = StoryRecord {
        story_id: "story-1".to_owned(),
        display_name: "Flooded Harbor".to_owned(),
        resource_id: resource.resource_id.clone(),
        graph: sample_story_graph(),
        world_state_schema: sample_world_state_schema(),
        player_state_schema: sample_player_state_schema(),
        introduction: "The courier reaches a flooded dock.".to_owned(),
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
                display_name: None,
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

#[tokio::test]
async fn update_player_description_persists_to_session_snapshot() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()), default_api_ids())
        .await
        .expect("handler should build");

    let character = sample_character_record();
    let resource = StoryResourcesRecord {
        resource_id: "resource-1".to_owned(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec![character.character_id.clone()],
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
        planned_story: None,
    };
    let story = StoryRecord {
        story_id: "story-1".to_owned(),
        display_name: "Flooded Harbor".to_owned(),
        resource_id: resource.resource_id.clone(),
        graph: sample_story_graph(),
        world_state_schema: sample_world_state_schema(),
        player_state_schema: sample_player_state_schema(),
        introduction: sample_story_payload("resource-1", "story-1").introduction,
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
                display_name: None,
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

    let result = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-2",
                Some(session_id.clone()),
                RequestParams::SessionUpdatePlayerDescription(UpdatePlayerDescriptionParams {
                    player_description: "A bold courier carrying medicine.".to_owned(),
                }),
            ))
            .await,
    );
    assert!(matches!(
        result,
        ResponseResult::PlayerDescriptionUpdated(payload)
            if payload.snapshot.player_description == "A bold courier carrying medicine."
    ));
}
