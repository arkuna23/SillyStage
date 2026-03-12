mod common;

use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use engine::SessionConfigMode;
use handler::Handler;
use protocol::{
    CreateStoryResourcesParams, ErrorPayload, GenerateStoryParams, JsonRpcRequestMessage,
    JsonRpcResponseMessage, RequestParams, RunTurnParams, StartSessionFromStoryParams,
    UploadChunkParams, UploadCompleteParams, UploadInitParams, UploadTargetKind,
};
use serde_json::json;
use ss_server::http::build_router;
use store::InMemoryStore;
use tower::util::ServiceExt;

use common::{
    QueuedMockLlm, assistant_response, default_api_ids, registry_with_ids, sample_archive,
    sample_player_state_schema, sample_story_graph, sample_world_state_schema,
};

#[tokio::test]
async fn rpc_unary_request_returns_json_rpc_response() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Arc::new(
        Handler::with_in_memory_store(registry_with_ids(Arc::clone(&llm)), default_api_ids())
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
            player_state_schema_seed: sample_player_state_schema(),
            world_state_schema_seed: Some(sample_world_state_schema()),
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
async fn rpc_stream_request_returns_sse_with_ack_and_messages() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"graph\":{\"start_node\":\"dock\",\"nodes\":[]},\"world_state_schema\":{\"fields\":{}},\"player_state_schema\":{\"fields\":{}},\"introduction\":\"At the dock.\"}",
                Some(json!({
                    "graph": sample_story_graph(),
                    "world_state_schema": sample_world_state_schema(),
                    "player_state_schema": sample_player_state_schema(),
                    "introduction": "At the dock."
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
    let handler = Arc::new(
        Handler::new(
            Arc::new(InMemoryStore::new()),
            registry_with_ids(Arc::clone(&llm)),
            default_api_ids(),
        )
        .await
        .expect("handler should build"),
    );
    let router = build_router(Arc::clone(&handler));
    let archive_bytes = sample_archive().to_chr_bytes().expect("archive should serialize");

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
                player_state_schema_seed: sample_player_state_schema(),
                world_state_schema_seed: Some(sample_world_state_schema()),
                planned_story: None,
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
                architect_api_id: None,
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
                player_description: "A cautious courier.".to_owned(),
                config_mode: SessionConfigMode::UseGlobal,
                session_api_ids: None,
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
            api_overrides: None,
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
async fn invalid_json_returns_standard_error_payload() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let handler = Arc::new(
        Handler::with_in_memory_store(registry_with_ids(Arc::clone(&llm)), default_api_ids())
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

async fn request_json(
    router: &axum::Router,
    message: JsonRpcRequestMessage,
) -> serde_json::Value {
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
