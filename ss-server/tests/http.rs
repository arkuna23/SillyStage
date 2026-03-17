mod common;

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use handler::Handler;
use protocol::{
    CharacterCoverMimeType, CharacterCreateParams, CharacterExportChrParams,
    CharacterGetCoverParams, CharacterSetCoverParams, CreateStoryResourcesParams,
    DashboardGetParams, ErrorPayload, GenerateStoryParams, JsonRpcRequestMessage,
    JsonRpcResponseMessage, RequestParams, RunTurnParams, StartSessionFromStoryParams,
    SuggestRepliesParams, UploadChunkParams, UploadCompleteParams, UploadInitParams,
    UploadTargetKind,
};
use serde_json::json;
use ss_server::http::build_router;
use store::{InMemoryStore, Store};
use tower::util::ServiceExt;

use common::{
    QueuedMockLlm, assistant_response, registry_with_ids, sample_api_group_record,
    sample_api_record, sample_archive, sample_character_content, sample_character_record,
    sample_player_profile, sample_player_state_schema, sample_preset_record, sample_schema_record,
    sample_story_graph, sample_world_state_schema,
};

async fn seed_schema_records(store: &InMemoryStore) {
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
}

async fn seed_player_profiles(store: &InMemoryStore) {
    store
        .save_player_profile(sample_player_profile(
            "profile-courier-a",
            "A cautious courier.",
        ))
        .await
        .expect("save player profile");
}

async fn seed_runtime_bindings(store: &InMemoryStore) {
    for api_id in [
        "default-planner",
        "default-architect",
        "default-director",
        "default-actor",
        "default-narrator",
        "default-keeper",
        "default-replyer",
    ] {
        store
            .save_api(sample_api_record(api_id, "default"))
            .await
            .expect("save default api");
    }
    store
        .save_api_group(sample_api_group_record("group-default", "default"))
        .await
        .expect("save default api group");
    store
        .save_preset(sample_preset_record("preset-default", 512))
        .await
        .expect("save default preset");
}

#[tokio::test]
async fn rpc_unary_request_returns_json_rpc_response() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Arc::new(
        Handler::with_in_memory_store(registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-1",
        None::<String>,
        RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
            story_concept: "A flooded harbor story.".to_owned(),
            character_ids: vec![],
            lorebook_ids: vec![],
            player_schema_id_seed: None,
            world_schema_id_seed: None,
            planned_story: None,
        }),
    ))
    .expect("request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let response: JsonRpcResponseMessage =
        serde_json::from_slice(&bytes).expect("response should deserialize");

    assert_eq!(response.id, "req-1");
}

#[tokio::test]
async fn rpc_character_get_cover_returns_json_rpc_response() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    store
        .save_character(sample_character_record())
        .await
        .expect("character should save");
    let handler = Arc::new(
        Handler::new(store, registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-cover",
        None::<String>,
        RequestParams::CharacterGetCover(CharacterGetCoverParams {
            character_id: "merchant".to_owned(),
        }),
    ))
    .expect("request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response should deserialize");

    assert_eq!(value["id"], "req-cover");
    assert_eq!(value["result"]["type"], "character_cover");
    assert_eq!(value["result"]["character_id"], "merchant");
    assert_eq!(value["result"]["cover_file_name"], "cover.png");
}

#[tokio::test]
async fn rpc_character_export_chr_returns_json_rpc_response() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    store
        .save_character(sample_character_record())
        .await
        .expect("character should save");
    let handler = Arc::new(
        Handler::new(store, registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-export",
        None::<String>,
        RequestParams::CharacterExportChr(CharacterExportChrParams {
            character_id: "merchant".to_owned(),
        }),
    ))
    .expect("request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response should deserialize");

    assert_eq!(value["id"], "req-export");
    assert_eq!(value["result"]["type"], "character_chr_export");
    assert_eq!(value["result"]["character_id"], "merchant");
    assert_eq!(value["result"]["file_name"], "merchant.chr");
    assert_eq!(
        value["result"]["content_type"],
        "application/x-sillystage-character-card"
    );
}

#[tokio::test]
async fn rpc_dashboard_get_returns_json_rpc_response() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Arc::new(
        Handler::with_in_memory_store(registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-dashboard",
        None::<String>,
        RequestParams::DashboardGet(DashboardGetParams::default()),
    ))
    .expect("request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response should deserialize");

    assert_eq!(value["id"], "req-dashboard");
    assert_eq!(value["result"]["type"], "dashboard");
    assert_eq!(value["result"]["health"]["status"], "ok");
}

#[tokio::test]
async fn rpc_dashboard_get_returns_null_global_config_when_unconfigured() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Arc::new(
        Handler::with_in_memory_store(registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-dashboard-empty",
        None::<String>,
        RequestParams::DashboardGet(DashboardGetParams::default()),
    ))
    .expect("request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response should deserialize");

    assert_eq!(value["result"]["type"], "dashboard");
    assert!(value["result"]["global_config"]["api_group_id"].is_null());
    assert!(value["result"]["global_config"]["preset_id"].is_null());
}

#[tokio::test]
async fn rpc_session_suggest_replies_returns_json_rpc_response() {
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
    seed_schema_records(&store).await;
    seed_player_profiles(&store).await;
    seed_runtime_bindings(&store).await;
    store
        .save_character(sample_character_record())
        .await
        .expect("character should save");
    store
        .save_story_resources(store::StoryResourcesRecord {
            resource_id: "resource-1".to_owned(),
            story_concept: "A flooded harbor story.".to_owned(),
            character_ids: vec!["merchant".to_owned()],
            lorebook_ids: vec![],
            player_schema_id_seed: Some("schema-player-default".to_owned()),
            world_schema_id_seed: Some("schema-world-default".to_owned()),
            planned_story: None,
        })
        .await
        .expect("resources should save");
    store
        .save_story(store::StoryRecord {
            story_id: "story-1".to_owned(),
            display_name: "Flooded Harbor".to_owned(),
            resource_id: "resource-1".to_owned(),
            graph: sample_story_graph(),
            world_schema_id: "schema-world-story-1".to_owned(),
            player_schema_id: "schema-player-story-1".to_owned(),
            introduction: "The courier reaches a flooded dock.".to_owned(),
            common_variables: vec![],
            created_at_ms: Some(1_000),
            updated_at_ms: Some(1_000),
        })
        .await
        .expect("story should save");
    let handler = Arc::new(
        Handler::new(store.clone(), registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler.clone());

    let start_body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-session-start",
        None::<String>,
        RequestParams::StoryStartSession(StartSessionFromStoryParams {
            story_id: "story-1".to_owned(),
            display_name: Some("Courier Run".to_owned()),
            player_profile_id: Some("profile-courier-a".to_owned()),
            api_group_id: None,
            preset_id: None,
        }),
    ))
    .expect("request should serialize");
    let start_response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(start_body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    let start_bytes = to_bytes(start_response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let start_value: serde_json::Value =
        serde_json::from_slice(&start_bytes).expect("response should deserialize");
    let session_id = start_value["session_id"]
        .as_str()
        .expect("session id should be present")
        .to_owned();

    let body = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-suggest",
        Some(session_id),
        RequestParams::SessionSuggestReplies(SuggestRepliesParams { limit: Some(3) }),
    ))
    .expect("request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        "application/json"
    );

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).expect("response should deserialize");

    assert_eq!(value["id"], "req-suggest");
    assert_eq!(value["result"]["type"], "suggested_replies");
    assert_eq!(value["result"]["replies"][0]["reply_id"], "r1");
}

#[tokio::test]
async fn rpc_stream_request_returns_sse_with_ack_and_messages() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"nodes\":[],\"start_node\":\"dock\",\"world_state_schema\":{\"fields\":{}},\"player_state_schema\":{\"fields\":{}},\"introduction\":\"At the dock.\",\"section_summary\":\"Opening dock scene.\"}",
                Some(json!({
                    "nodes": sample_story_graph().nodes,
                    "transition_patches": [],
                    "start_node": "dock",
                    "world_state_schema": sample_world_state_schema(),
                    "player_state_schema": sample_player_state_schema(),
                    "introduction": "At the dock.",
                    "section_summary": "Opening dock scene."
                })),
            )),
            Ok(assistant_response("{\"ops\":[]}", Some(json!({"ops": []})))),
            Ok(assistant_response(
                "{\"beats\":[{\"type\":\"Narrator\",\"purpose\":\"DescribeScene\"}]}",
                Some(json!({"beats":[{"type":"Narrator","purpose":"DescribeScene"}]})),
            )),
            Ok(assistant_response("{\"ops\":[]}", Some(json!({"ops": []})))),
        ],
        vec![Ok(vec![
            Ok(llm::ChatChunk {
                delta: "The dock groans under the flood.".to_owned(),
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
    seed_schema_records(&store).await;
    seed_player_profiles(&store).await;
    seed_runtime_bindings(&store).await;
    let handler = Arc::new(
        Handler::new(store, registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(Arc::clone(&handler));
    let archive_bytes = sample_archive()
        .to_chr_bytes()
        .expect("archive should serialize");

    let upload_init = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-upload-init",
            None::<String>,
            RequestParams::UploadInit(UploadInitParams {
                target_kind: UploadTargetKind::CharacterCard,
                file_name: "merchant.chr".to_owned(),
                content_type: "application/x-sillystage-character-card".to_owned(),
                total_size: archive_bytes.len() as u64,
                sha256: "sha".to_owned(),
            }),
        ),
    )
    .await;
    let upload_id = upload_init["result"]["upload_id"]
        .as_str()
        .expect("upload id should exist")
        .to_owned();

    request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-upload-chunk",
            None::<String>,
            RequestParams::UploadChunk(UploadChunkParams {
                upload_id: upload_id.clone(),
                chunk_index: 0,
                offset: 0,
                payload_base64: {
                    use base64::Engine as _;
                    base64::engine::general_purpose::STANDARD.encode(archive_bytes)
                },
                is_last: true,
            }),
        ),
    )
    .await;

    let upload_complete = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-upload-complete",
            None::<String>,
            RequestParams::UploadComplete(UploadCompleteParams { upload_id }),
        ),
    )
    .await;
    let character_id = upload_complete["result"]["character_id"]
        .as_str()
        .expect("character id should exist")
        .to_owned();

    let resources = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-resources",
            None::<String>,
            RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
                story_concept: "A flooded harbor story.".to_owned(),
                character_ids: vec![character_id],
                lorebook_ids: vec![],
                player_schema_id_seed: Some("schema-player-default".to_owned()),
                world_schema_id_seed: Some("schema-world-default".to_owned()),
                planned_story: Some(
                    "Title:\nFlooded Harbor\n\nOpening Situation:\nA courier arrives at a flooded dock.\n\nCore Conflict:\nTrade routes are collapsing.\n\nCharacter Roles:\nHaru (merchant) watches the tide.\n\nSuggested Beats:\n- The courier arrives at the dock.\n\nState Hints:\nTrack the flood level."
                        .to_owned(),
                ),
            }),
        ),
    )
    .await;
    let resource_id = resources["result"]["resource_id"]
        .as_str()
        .expect("resource id should exist")
        .to_owned();

    let story = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-generate",
            None::<String>,
            RequestParams::StoryGenerate(GenerateStoryParams {
                resource_id,
                display_name: Some("Flooded Harbor".to_owned()),
                api_group_id: None,
                preset_id: None,
                common_variables: None,
            }),
        ),
    )
    .await;
    let story_id = story["result"]["story_id"]
        .as_str()
        .expect("story id should exist")
        .to_owned();

    let session = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-start-session",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id,
                display_name: None,
                player_profile_id: Some("profile-courier-a".to_owned()),
                api_group_id: None,
                preset_id: None,
            }),
        ),
    )
    .await;
    let session_id = session["result"]["snapshot"]["story_id"]
        .as_str()
        .expect("session snapshot story id should exist");
    assert_eq!(session_id, "story-0");

    let stream_request = serde_json::to_vec(&JsonRpcRequestMessage::new(
        "req-turn",
        Some("session-0"),
        RequestParams::SessionRunTurn(RunTurnParams {
            player_input: "Ask about the dock.".to_owned(),
        }),
    ))
    .expect("stream request should serialize");

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(stream_request))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::OK);
    assert!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/event-stream"))
    );

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("stream body should collect");
    let text = String::from_utf8(bytes.to_vec()).expect("sse body should be utf8");

    assert!(text.contains("event: ack"));
    assert!(text.contains("\"type\":\"turn_stream_accepted\""));
    assert!(text.contains("event: message"));
    assert!(text.contains("\"type\":\"turn_started\""));
    assert!(text.contains("\"type\":\"completed\""));
}

#[tokio::test]
async fn rpc_character_create_and_set_cover_return_json_rpc_response() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_schema_records(&store).await;
    let handler = Arc::new(
        Handler::new(store, registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let create = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-create",
            None::<String>,
            RequestParams::CharacterCreate(CharacterCreateParams {
                content: sample_character_content(),
            }),
        ),
    )
    .await;
    assert_eq!(create["id"], "req-create");
    assert_eq!(create["result"]["type"], "character_created");
    assert!(create["result"]["character_summary"]["cover_file_name"].is_null());

    let set_cover = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-set-cover",
            None::<String>,
            RequestParams::CharacterSetCover(CharacterSetCoverParams {
                character_id: "merchant".to_owned(),
                cover_mime_type: CharacterCoverMimeType::Png,
                cover_base64: {
                    use base64::Engine as _;
                    base64::engine::general_purpose::STANDARD.encode(b"cover-bytes")
                },
            }),
        ),
    )
    .await;
    assert_eq!(set_cover["id"], "req-set-cover");
    assert_eq!(set_cover["result"]["type"], "character_cover_updated");
    assert_eq!(set_cover["result"]["cover_file_name"], "cover.png");
}

#[tokio::test]
async fn rpc_character_export_chr_without_cover_returns_conflict() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_schema_records(&store).await;
    let handler = Arc::new(
        Handler::new(store, registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let _ = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-create",
            None::<String>,
            RequestParams::CharacterCreate(CharacterCreateParams {
                content: sample_character_content(),
            }),
        ),
    )
    .await;

    let response = request_json(
        &router,
        JsonRpcRequestMessage::new(
            "req-export",
            None::<String>,
            RequestParams::CharacterExportChr(CharacterExportChrParams {
                character_id: "merchant".to_owned(),
            }),
        ),
    )
    .await;

    assert_eq!(response["id"], "req-export");
    assert_eq!(
        response["error"]["code"],
        protocol::ErrorCode::Conflict.rpc_code()
    );
}

#[tokio::test]
async fn invalid_json_returns_standard_error_payload() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Arc::new(
        Handler::with_in_memory_store(registry_with_ids(Arc::clone(&llm)))
            .await
            .expect("handler should build"),
    );
    let router = build_router(handler);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{".as_bytes().to_vec()))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    let error: ErrorPayload = serde_json::from_slice(&bytes).expect("error should deserialize");
    assert_eq!(error.code, protocol::ErrorCode::ParseError.rpc_code());
}

async fn request_json(router: &axum::Router, message: JsonRpcRequestMessage) -> serde_json::Value {
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/rpc")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::to_vec(&message).expect("request should serialize"),
                ))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should collect");
    serde_json::from_slice(&bytes).expect("response should deserialize")
}
