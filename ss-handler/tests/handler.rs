mod common;

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use engine::{AgentApiIdOverrides, AgentApiIds, LlmApiRegistry, SessionConfigMode};
use futures_util::StreamExt;
use protocol::{
    CharacterArchive, CharacterCardContent, CharacterCoverMimeType, CharacterCreateParams,
    CharacterExportChrParams, CharacterGetCoverParams, CharacterGetParams, CharacterSetCoverParams,
    CharacterUpdateParams, ConfigGetGlobalParams, ConfigUpdateGlobalParams,
    CreateSessionMessageParams, CreateStoryResourcesParams, DashboardGetParams,
    DefaultLlmConfigGetParams, DefaultLlmConfigUpdateParams, DeleteSessionMessageParams,
    DeleteSessionParams, ErrorCode, GenerateStoryParams, GetSessionMessageParams, GetSessionParams,
    GetStoryResourcesParams, JsonRpcOutcome, JsonRpcRequestMessage, ListSessionMessagesParams,
    LlmApiCreateParams, LlmApiDeleteParams, LlmApiGetParams, LlmApiUpdateParams, RequestParams,
    ResponseResult, RunTurnParams, SessionMessageKind, SessionUpdateConfigParams,
    StartSessionFromStoryParams, StreamFrame, SuggestRepliesParams, UpdatePlayerDescriptionParams,
    UpdateSessionMessageParams, UpdateSessionParams, UpdateStoryParams, UploadChunkParams,
    UploadCompleteParams, UploadInitParams, UploadTargetKind,
};
use serde_json::json;
use ss_handler::{Handler, HandlerReply};
use store::{
    InMemoryStore, LlmApiRecord, LlmProvider, SessionConfigMode as StoreSessionConfigMode,
    SessionEngineConfig, SessionRecord, Store, StoryRecord, StoryResourcesRecord,
};

use common::{
    QueuedMockLlm, assistant_response, sample_archive, sample_character_content,
    sample_character_record, sample_player_profile, sample_player_state_schema,
    sample_schema_record, sample_story_graph, sample_story_record, sample_world_state_schema,
};

fn default_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner-default".to_owned(),
        architect_api_id: "architect-default".to_owned(),
        director_api_id: "director-default".to_owned(),
        actor_api_id: "actor-default".to_owned(),
        narrator_api_id: "narrator-default".to_owned(),
        keeper_api_id: "keeper-default".to_owned(),
        replyer_api_id: "replyer-default".to_owned(),
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
        replyer_api_id: "replyer-alt".to_owned(),
    }
}

fn registry_with_ids(llm: Arc<QueuedMockLlm>) -> LlmApiRegistry {
    let default = default_api_ids();
    let alternate = alternate_api_ids();
    let llm: Arc<dyn llm::LlmApi> = llm;

    LlmApiRegistry::new()
        .register(default.planner_api_id, Arc::clone(&llm), "planner-model")
        .register(
            default.architect_api_id,
            Arc::clone(&llm),
            "architect-model",
        )
        .register(default.director_api_id, Arc::clone(&llm), "director-model")
        .register(default.actor_api_id, Arc::clone(&llm), "actor-model")
        .register(default.narrator_api_id, Arc::clone(&llm), "narrator-model")
        .register(default.keeper_api_id, Arc::clone(&llm), "keeper-model")
        .register(default.replyer_api_id, Arc::clone(&llm), "replyer-model")
        .register(
            alternate.planner_api_id,
            Arc::clone(&llm),
            "planner-alt-model",
        )
        .register(
            alternate.architect_api_id,
            Arc::clone(&llm),
            "architect-alt-model",
        )
        .register(
            alternate.director_api_id,
            Arc::clone(&llm),
            "director-alt-model",
        )
        .register(alternate.actor_api_id, Arc::clone(&llm), "actor-alt-model")
        .register(
            alternate.narrator_api_id,
            Arc::clone(&llm),
            "narrator-alt-model",
        )
        .register(
            alternate.keeper_api_id,
            Arc::clone(&llm),
            "keeper-alt-model",
        )
        .register(alternate.replyer_api_id, llm, "replyer-alt-model")
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

fn sample_llm_api_record(api_id: &str, model: &str) -> LlmApiRecord {
    LlmApiRecord {
        api_id: api_id.to_owned(),
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: "sk-secret-token".to_owned(),
        model: model.to_owned(),
        temperature: Some(0.4),
        max_tokens: Some(768),
    }
}

async fn build_handler(llm: Arc<QueuedMockLlm>) -> (Arc<InMemoryStore>, Handler) {
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");
    (store, handler)
}

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
}

async fn seed_story_records(store: &InMemoryStore) {
    seed_schema_records(store).await;
    seed_player_profiles(store).await;
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
        .save_story(sample_story_record("resource-1", "story-1"))
        .await
        .expect("save story");
}

#[tokio::test]
async fn upload_character_card_and_create_resources_via_character_id() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm.clone()).await;
    seed_schema_records(&store).await;
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
                    player_schema_id_seed: Some("schema-player-default".to_owned()),
                    world_schema_id_seed: Some("schema-world-default".to_owned()),
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
    let (store, handler) = build_handler(llm).await;
    seed_schema_records(&store).await;

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-create",
                None::<String>,
                RequestParams::CharacterCreate(CharacterCreateParams {
                    content: sample_character_content(),
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
async fn character_update_replaces_content_and_preserves_cover() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm).await;
    seed_schema_records(&store).await;

    unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-create",
                None::<String>,
                RequestParams::CharacterCreate(CharacterCreateParams {
                    content: sample_character_content(),
                }),
            ))
            .await,
    );
    unary_result(
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

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-update",
                None::<String>,
                RequestParams::CharacterUpdate(CharacterUpdateParams {
                    character_id: "merchant".to_owned(),
                    content: CharacterCardContent {
                        id: "merchant".to_owned(),
                        name: "Haru of the Flooded Dock".to_owned(),
                        personality: "more cautious after the storm".to_owned(),
                        style: "measured, observant".to_owned(),
                        tendencies: vec![
                            "likes profitable deals".to_owned(),
                            "keeps an eye on the tide".to_owned(),
                        ],
                        schema_id: "schema-character-merchant".to_owned(),
                        system_prompt: "Stay in character and watch the waterline.".to_owned(),
                    },
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::Character(payload) => {
            assert_eq!(payload.character_id, "merchant");
            assert_eq!(payload.content.name, "Haru of the Flooded Dock");
            assert_eq!(payload.cover_file_name.as_deref(), Some("cover.png"));
            assert_eq!(payload.cover_mime_type, Some(CharacterCoverMimeType::Png));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let got_cover = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-get-cover",
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
}

#[tokio::test]
async fn character_update_rejects_mismatched_content_id() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm).await;
    seed_schema_records(&store).await;

    unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-create",
                None::<String>,
                RequestParams::CharacterCreate(CharacterCreateParams {
                    content: sample_character_content(),
                }),
            ))
            .await,
    );

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-update",
            None::<String>,
            RequestParams::CharacterUpdate(CharacterUpdateParams {
                character_id: "merchant".to_owned(),
                content: CharacterCardContent {
                    id: "guard".to_owned(),
                    name: "Haru".to_owned(),
                    personality: "greedy but friendly trader".to_owned(),
                    style: "talkative, casual".to_owned(),
                    tendencies: vec!["likes profitable deals".to_owned()],
                    schema_id: "schema-character-merchant".to_owned(),
                    system_prompt: "Stay in character.".to_owned(),
                },
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };
    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error) if error.code == ErrorCode::InvalidRequest.rpc_code()
    ));
}

#[tokio::test]
async fn story_and_session_crud_follow_store_objects() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
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
        ))],
        vec![],
    ));
    let store = Arc::new(InMemoryStore::new());
    seed_schema_records(&store).await;
    seed_player_profiles(&store).await;
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");

    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm.clone()),
        Some(default_api_ids()),
        None,
    )
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
                    player_schema_id_seed: Some("schema-player-default".to_owned()),
                    world_schema_id_seed: Some("schema-world-default".to_owned()),
                    planned_story: Some(
                        "Title:\nFlooded Harbor\n\nOpening Situation:\nA courier arrives at a flooded dock.\n\nCore Conflict:\nTrade routes are collapsing.\n\nCharacter Roles:\nHaru (merchant) watches the tide.\n\nSuggested Beats:\n- The courier arrives at the dock.\n\nState Hints:\nTrack the flood level."
                            .to_owned(),
                    ),
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
                player_profile_id: Some("profile-courier-a".to_owned()),
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
                assert!(payload.history.is_empty());
                assert!(payload.created_at_ms.is_some());
                assert!(payload.updated_at_ms.is_some());
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
            assert!(payload.history.is_empty());
            assert!(payload.created_at_ms.is_some());
            assert!(payload.updated_at_ms.is_some());
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
async fn story_update_changes_display_name() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-update",
                None::<String>,
                RequestParams::StoryUpdate(UpdateStoryParams {
                    story_id: "story-1".to_owned(),
                    display_name: "Updated Flooded Harbor".to_owned(),
                }),
            ))
            .await,
    );

    match updated {
        ResponseResult::Story(payload) => {
            assert_eq!(payload.story_id, "story-1");
            assert_eq!(payload.display_name, "Updated Flooded Harbor");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn session_update_changes_display_name() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "session-start",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: Some("Courier Run".to_owned()),
                player_profile_id: Some("profile-courier-a".to_owned()),
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

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-update",
                Some(session_id.clone()),
                RequestParams::SessionUpdate(UpdateSessionParams {
                    display_name: "Updated Courier Run".to_owned(),
                }),
            ))
            .await,
    );

    match updated {
        ResponseResult::Session(payload) => {
            assert_eq!(payload.session_id, session_id);
            assert_eq!(payload.display_name, "Updated Courier Run");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn session_message_crud_round_trips_and_updates_session_history() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "session-start",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: Some("Courier Run".to_owned()),
                player_profile_id: Some("profile-courier-a".to_owned()),
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

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-message-create",
                Some(session_id.clone()),
                RequestParams::SessionMessageCreate(CreateSessionMessageParams {
                    kind: SessionMessageKind::Narration,
                    speaker_id: "narrator".to_owned(),
                    speaker_name: "Narrator".to_owned(),
                    text: "Fog spills over the flooded dock.".to_owned(),
                }),
            ))
            .await,
    );
    let message_id = match created {
        ResponseResult::SessionMessage(payload) => {
            assert_eq!(payload.kind, SessionMessageKind::Narration);
            assert_eq!(payload.sequence, 0);
            assert_eq!(payload.text, "Fog spills over the flooded dock.");
            payload.message_id.clone()
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-message-get",
                Some(session_id.clone()),
                RequestParams::SessionMessageGet(GetSessionMessageParams {
                    message_id: message_id.clone(),
                }),
            ))
            .await,
    );
    match fetched {
        ResponseResult::SessionMessage(payload) => {
            assert_eq!(payload.message_id, message_id);
            assert_eq!(payload.speaker_id, "narrator");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-message-update",
                Some(session_id.clone()),
                RequestParams::SessionMessageUpdate(UpdateSessionMessageParams {
                    message_id: message_id.clone(),
                    kind: SessionMessageKind::Dialogue,
                    speaker_id: "merchant".to_owned(),
                    speaker_name: "Haru".to_owned(),
                    text: "The tide will turn soon.".to_owned(),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::SessionMessage(payload) => {
            assert_eq!(payload.message_id, message_id);
            assert_eq!(payload.kind, SessionMessageKind::Dialogue);
            assert_eq!(payload.speaker_name, "Haru");
            assert_eq!(payload.text, "The tide will turn soon.");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let listed = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-message-list",
                Some(session_id.clone()),
                RequestParams::SessionMessageList(ListSessionMessagesParams::default()),
            ))
            .await,
    );
    match listed {
        ResponseResult::SessionMessagesListed(payload) => {
            assert_eq!(payload.messages.len(), 1);
            assert_eq!(payload.messages[0].message_id, message_id);
            assert_eq!(payload.messages[0].sequence, 0);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let session = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-get",
                Some(session_id.clone()),
                RequestParams::SessionGet(GetSessionParams::default()),
            ))
            .await,
    );
    match session {
        ResponseResult::Session(payload) => {
            assert_eq!(payload.history.len(), 1);
            assert_eq!(payload.history[0].message_id, message_id);
            assert_eq!(payload.history[0].speaker_name, "Haru");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let deleted = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-message-delete",
                Some(session_id.clone()),
                RequestParams::SessionMessageDelete(DeleteSessionMessageParams {
                    message_id: message_id.clone(),
                }),
            ))
            .await,
    );
    match deleted {
        ResponseResult::SessionMessageDeleted(payload) => {
            assert_eq!(payload.message_id, message_id);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let listed_after_delete = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-message-list",
                Some(session_id.clone()),
                RequestParams::SessionMessageList(ListSessionMessagesParams::default()),
            ))
            .await,
    );
    match listed_after_delete {
        ResponseResult::SessionMessagesListed(payload) => {
            assert!(payload.messages.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn session_suggest_replies_returns_unary_payload() {
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
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "session-start",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: Some("Courier Run".to_owned()),
                player_profile_id: Some("profile-courier-a".to_owned()),
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

    let suggested = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "suggest-replies",
                Some(session_id),
                RequestParams::SessionSuggestReplies(SuggestRepliesParams {
                    limit: Some(3),
                    api_overrides: None,
                }),
            ))
            .await,
    );

    match suggested {
        ResponseResult::SuggestedReplies(payload) => {
            assert_eq!(payload.replies.len(), 3);
            assert_eq!(payload.replies[0].reply_id, "r1");
            assert_eq!(payload.replies[0].text, "Show me the fastest safe route.");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    assert!(
        store
            .list_sessions()
            .await
            .expect("sessions should load")
            .len()
            == 1
    );
}

#[tokio::test]
async fn session_config_can_switch_between_session_and_global_modes() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm.clone()),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-1",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: None,
                player_profile_id: Some("profile-courier-a".to_owned()),
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
async fn dashboard_get_returns_counts_global_config_and_recent_lists() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_schema_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let character_with_cover = sample_character_record();
    let mut character_without_cover = sample_character_record();
    character_without_cover.character_id = "guard".to_owned();
    character_without_cover.content.id = "guard".to_owned();
    character_without_cover.content.name = "Mina".to_owned();
    character_without_cover.cover_file_name = None;
    character_without_cover.cover_mime_type = None;
    character_without_cover.cover_bytes = None;

    store
        .save_character(character_with_cover)
        .await
        .expect("save covered character");
    store
        .save_character(character_without_cover)
        .await
        .expect("save uncovered character");

    for index in 0_u64..6 {
        store
            .save_story_resources(StoryResourcesRecord {
                resource_id: format!("resource-{index}"),
                story_concept: format!("Story concept {index}"),
                character_ids: vec!["merchant".to_owned()],
                player_schema_id_seed: Some("schema-player-default".to_owned()),
                world_schema_id_seed: Some("schema-world-default".to_owned()),
                planned_story: None,
            })
            .await
            .expect("save resources");

        store
            .save_story(StoryRecord {
                story_id: format!("story-{index}"),
                display_name: format!("Story {index}"),
                resource_id: format!("resource-{index}"),
                graph: sample_story_graph(),
                world_schema_id: "schema-world-story-1".to_owned(),
                player_schema_id: "schema-player-story-1".to_owned(),
                introduction: format!("Intro {index}"),
                created_at_ms: Some(index),
                updated_at_ms: if index == 0 { None } else { Some(index * 100) },
            })
            .await
            .expect("save story");

        store
            .save_session(SessionRecord {
                session_id: format!("session-{index}"),
                display_name: format!("Session {index}"),
                story_id: format!("story-{index}"),
                player_profile_id: None,
                player_schema_id: "schema-player-story-1".to_owned(),
                snapshot: engine::RuntimeSnapshot {
                    story_id: format!("story-{index}"),
                    player_description: format!("Player {index}"),
                    world_state: state::WorldState::new("dock"),
                    turn_index: index,
                },
                config: SessionEngineConfig {
                    mode: StoreSessionConfigMode::UseGlobal,
                    session_api_ids: None,
                },
                created_at_ms: Some(index),
                updated_at_ms: if index == 1 { None } else { Some(index * 200) },
            })
            .await
            .expect("save session");
    }

    let result = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-dashboard",
                None::<String>,
                RequestParams::DashboardGet(DashboardGetParams::default()),
            ))
            .await,
    );

    match result {
        ResponseResult::Dashboard(payload) => {
            assert_eq!(payload.health.status, protocol::DashboardHealthStatus::Ok);
            assert_eq!(payload.counts.characters_total, 2);
            assert_eq!(payload.counts.characters_with_cover, 1);
            assert_eq!(payload.counts.story_resources_total, 6);
            assert_eq!(payload.counts.stories_total, 6);
            assert_eq!(payload.counts.sessions_total, 6);
            assert_eq!(payload.global_config.api_ids, Some(default_api_ids()));
            assert_eq!(payload.recent_stories.len(), 5);
            assert_eq!(payload.recent_sessions.len(), 5);

            let story_ids = payload
                .recent_stories
                .iter()
                .map(|story| story.story_id.as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                story_ids,
                vec!["story-5", "story-4", "story-3", "story-2", "story-1"]
            );

            let session_ids = payload
                .recent_sessions
                .iter()
                .map(|session| session.session_id.as_str())
                .collect::<Vec<_>>();
            assert_eq!(
                session_ids,
                vec![
                    "session-5",
                    "session-4",
                    "session-3",
                    "session-2",
                    "session-1"
                ]
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn dashboard_and_global_config_are_empty_when_llm_is_unconfigured() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store, registry_with_ids(llm), None, None)
        .await
        .expect("handler should build");

    let dashboard = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-dashboard-empty",
                None::<String>,
                RequestParams::DashboardGet(DashboardGetParams::default()),
            ))
            .await,
    );

    match dashboard {
        ResponseResult::Dashboard(payload) => {
            assert_eq!(payload.global_config.api_ids, None);
            assert_eq!(payload.counts.characters_total, 0);
            assert_eq!(payload.counts.story_resources_total, 0);
            assert_eq!(payload.counts.stories_total, 0);
            assert_eq!(payload.counts.sessions_total, 0);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let global_config = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-global-empty",
                None::<String>,
                RequestParams::ConfigGetGlobal(ConfigGetGlobalParams::default()),
            ))
            .await,
    );

    match global_config {
        ResponseResult::GlobalConfig(payload) => {
            assert_eq!(payload.api_ids, None);
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
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm.clone()),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-1",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: None,
                player_profile_id: Some("profile-courier-a".to_owned()),
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
    let mut messages = store
        .list_session_messages(&session_id)
        .await
        .expect("load session messages");
    messages.sort_by_key(|message| message.sequence);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].speaker_id, "player");
    assert_eq!(messages[0].text, "Open the gate.");
    assert_eq!(messages[1].kind, store::SessionMessageKind::Narration);
    assert_eq!(messages[1].speaker_id, "narrator");
    assert_eq!(messages[1].text, "Water churned beneath the dock.");
}

#[tokio::test]
async fn update_player_description_persists_to_session_snapshot() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm.clone()),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let started = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-1",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: None,
                player_profile_id: Some("profile-courier-a".to_owned()),
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

#[tokio::test]
async fn llm_api_crud_masks_keys_and_preserves_secret_on_partial_update() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        None,
    )
    .await
    .expect("handler should build");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-create",
                None::<String>,
                RequestParams::LlmApiCreate(LlmApiCreateParams {
                    api_id: "managed".to_owned(),
                    provider: Some(LlmProvider::OpenAi),
                    base_url: Some("https://api.openai.example/v1".to_owned()),
                    api_key: Some("sk-secret-token".to_owned()),
                    model: Some("gpt-4.1-mini".to_owned()),
                    temperature: Some(0.3),
                    max_tokens: Some(1_024),
                }),
            ))
            .await,
    );
    match created {
        ResponseResult::LlmApi(payload) => {
            assert_eq!(payload.api_id, "managed");
            assert_eq!(payload.temperature, Some(0.3));
            assert_eq!(payload.max_tokens, Some(1_024));
            assert!(payload.has_api_key);
            assert_eq!(payload.api_key_masked.as_deref(), Some("sk****en"));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-get",
                None::<String>,
                RequestParams::LlmApiGet(LlmApiGetParams {
                    api_id: "managed".to_owned(),
                }),
            ))
            .await,
    );
    match fetched {
        ResponseResult::LlmApi(payload) => {
            assert_eq!(payload.model, "gpt-4.1-mini");
            assert_eq!(payload.temperature, Some(0.3));
            assert_eq!(payload.max_tokens, Some(1_024));
            assert_eq!(payload.api_key_masked.as_deref(), Some("sk****en"));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let listed = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-list",
                None::<String>,
                RequestParams::LlmApiList(protocol::LlmApiListParams::default()),
            ))
            .await,
    );
    match listed {
        ResponseResult::LlmApisListed(payload) => {
            assert!(payload.apis.iter().any(|api| api.api_id == "managed"));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-update",
                None::<String>,
                RequestParams::LlmApiUpdate(LlmApiUpdateParams {
                    api_id: "managed".to_owned(),
                    provider: None,
                    base_url: Some("https://api.alt.example/v1".to_owned()),
                    api_key: None,
                    model: Some("gpt-4.1".to_owned()),
                    temperature: Some(0.6),
                    max_tokens: Some(2_048),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::LlmApi(payload) => {
            assert_eq!(payload.base_url, "https://api.alt.example/v1");
            assert_eq!(payload.model, "gpt-4.1");
            assert_eq!(payload.temperature, Some(0.6));
            assert_eq!(payload.max_tokens, Some(2_048));
            assert_eq!(payload.api_key_masked.as_deref(), Some("sk****en"));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let stored = store
        .get_llm_api("managed")
        .await
        .expect("llm api should load")
        .expect("llm api should exist");
    assert_eq!(stored.api_key, "sk-secret-token");
    assert_eq!(stored.temperature, Some(0.6));
    assert_eq!(stored.max_tokens, Some(2_048));

    let deleted = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-delete",
                None::<String>,
                RequestParams::LlmApiDelete(LlmApiDeleteParams {
                    api_id: "managed".to_owned(),
                }),
            ))
            .await,
    );
    match deleted {
        ResponseResult::LlmApiDeleted(payload) => assert_eq!(payload.api_id, "managed"),
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn first_llm_api_create_auto_initializes_global_config() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm), None, None)
        .await
        .expect("handler should build");

    let _ = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-create-first",
                None::<String>,
                RequestParams::LlmApiCreate(LlmApiCreateParams {
                    api_id: "managed".to_owned(),
                    provider: Some(LlmProvider::OpenAi),
                    base_url: Some("https://api.openai.example/v1".to_owned()),
                    api_key: Some("sk-secret-token".to_owned()),
                    model: Some("gpt-4.1-mini".to_owned()),
                    temperature: Some(0.3),
                    max_tokens: Some(1_024),
                }),
            ))
            .await,
    );

    assert_eq!(
        store
            .get_global_config()
            .await
            .expect("global config should load"),
        Some(AgentApiIds {
            planner_api_id: "managed".to_owned(),
            architect_api_id: "managed".to_owned(),
            director_api_id: "managed".to_owned(),
            actor_api_id: "managed".to_owned(),
            narrator_api_id: "managed".to_owned(),
            keeper_api_id: "managed".to_owned(),
            replyer_api_id: "managed".to_owned(),
        })
    );
}

#[tokio::test]
async fn default_llm_config_get_and_update_work_with_runtime_override() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        Some(store::DefaultLlmConfigRecord {
            provider: LlmProvider::OpenAi,
            base_url: "https://runtime.example/v1".to_owned(),
            api_key: "sk-runtime".to_owned(),
            model: "runtime-model".to_owned(),
            temperature: Some(0.6),
            max_tokens: Some(4_096),
        }),
    )
    .await
    .expect("handler should build");

    let initial = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "default-get-1",
                None::<String>,
                RequestParams::DefaultLlmConfigGet(DefaultLlmConfigGetParams::default()),
            ))
            .await,
    );
    match initial {
        ResponseResult::DefaultLlmConfig(payload) => {
            assert!(payload.saved.is_none());
            let effective = payload.effective.expect("effective config should exist");
            assert_eq!(effective.base_url, "https://runtime.example/v1");
            assert_eq!(effective.model, "runtime-model");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "default-update",
                None::<String>,
                RequestParams::DefaultLlmConfigUpdate(DefaultLlmConfigUpdateParams {
                    provider: LlmProvider::OpenAi,
                    base_url: "https://saved.example/v1".to_owned(),
                    api_key: "sk-saved".to_owned(),
                    model: "saved-model".to_owned(),
                    temperature: Some(0.2),
                    max_tokens: Some(1_024),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::DefaultLlmConfig(payload) => {
            let saved = payload.saved.expect("saved config should exist");
            assert_eq!(saved.base_url, "https://saved.example/v1");
            let effective = payload.effective.expect("effective config should exist");
            assert_eq!(effective.base_url, "https://runtime.example/v1");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn llm_api_create_uses_effective_default_llm_config_when_fields_are_missing() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(
        store.clone(),
        registry_with_ids(llm),
        Some(default_api_ids()),
        Some(store::DefaultLlmConfigRecord {
            provider: LlmProvider::OpenAi,
            base_url: "https://runtime.example/v1".to_owned(),
            api_key: "sk-runtime".to_owned(),
            model: "runtime-model".to_owned(),
            temperature: Some(0.7),
            max_tokens: Some(2_048),
        }),
    )
    .await
    .expect("handler should build");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "llm-create-defaulted",
                None::<String>,
                RequestParams::LlmApiCreate(LlmApiCreateParams {
                    api_id: "managed-defaulted".to_owned(),
                    provider: None,
                    base_url: None,
                    api_key: None,
                    model: None,
                    temperature: None,
                    max_tokens: None,
                }),
            ))
            .await,
    );

    match created {
        ResponseResult::LlmApi(payload) => {
            assert_eq!(payload.api_id, "managed-defaulted");
            assert_eq!(payload.base_url, "https://runtime.example/v1");
            assert_eq!(payload.model, "runtime-model");
            assert_eq!(payload.temperature, Some(0.7));
            assert_eq!(payload.max_tokens, Some(2_048));
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn llm_api_create_fails_when_required_fields_are_missing_and_no_default_exists() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store, registry_with_ids(llm), Some(default_api_ids()), None)
        .await
        .expect("handler should build");

    match handler
        .handle(JsonRpcRequestMessage::new(
            "llm-create-incomplete",
            None::<String>,
            RequestParams::LlmApiCreate(LlmApiCreateParams {
                api_id: "incomplete".to_owned(),
                provider: None,
                base_url: None,
                api_key: None,
                model: None,
                temperature: None,
                max_tokens: None,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => match response.outcome {
            JsonRpcOutcome::Err(error) => {
                assert_eq!(error.code, ErrorCode::InvalidRequest.rpc_code())
            }
            JsonRpcOutcome::Ok(result) => panic!("unexpected success response: {result:?}"),
        },
        HandlerReply::Stream { .. } => panic!("expected unary error reply"),
    }
}

#[tokio::test]
async fn llm_api_delete_conflicts_when_referenced_by_global_or_session_config() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    store
        .save_llm_api(sample_llm_api_record("planner-default", "planner-model"))
        .await
        .expect("seed global llm api");
    store
        .save_llm_api(sample_llm_api_record("session-only", "session-model"))
        .await
        .expect("seed session llm api");

    let registry = {
        let extra_llm: Arc<dyn llm::LlmApi> = llm.clone();
        registry_with_ids(llm).register("session-only", extra_llm, "session-model")
    };
    let handler = Handler::new(store.clone(), registry, Some(default_api_ids()), None)
        .await
        .expect("handler should build");

    let global_conflict = handler
        .handle(JsonRpcRequestMessage::new(
            "llm-delete-global",
            None::<String>,
            RequestParams::LlmApiDelete(LlmApiDeleteParams {
                api_id: "planner-default".to_owned(),
            }),
        ))
        .await;
    match global_conflict {
        HandlerReply::Unary(response) => match response.outcome {
            JsonRpcOutcome::Err(error) => {
                assert_eq!(error.code, ErrorCode::Conflict.rpc_code())
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        HandlerReply::Stream { .. } => panic!("unexpected stream reply"),
    }

    let mut session_api_ids = default_api_ids();
    session_api_ids.actor_api_id = "session-only".to_owned();
    let session_start = handler
        .handle(JsonRpcRequestMessage::new(
            "session-start",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: Some("Config Test".to_owned()),
                player_profile_id: Some("profile-courier-a".to_owned()),
                config_mode: SessionConfigMode::UseSession,
                session_api_ids: Some(session_api_ids),
            }),
        ))
        .await;
    let session_id = match session_start {
        HandlerReply::Unary(response) => response
            .session_id
            .expect("session id should be present on session start"),
        HandlerReply::Stream { .. } => panic!("unexpected stream reply"),
    };
    assert!(!session_id.is_empty());

    let session_conflict = handler
        .handle(JsonRpcRequestMessage::new(
            "llm-delete-session",
            None::<String>,
            RequestParams::LlmApiDelete(LlmApiDeleteParams {
                api_id: "session-only".to_owned(),
            }),
        ))
        .await;
    match session_conflict {
        HandlerReply::Unary(response) => match response.outcome {
            JsonRpcOutcome::Err(error) => {
                assert_eq!(error.code, ErrorCode::Conflict.rpc_code())
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        HandlerReply::Stream { .. } => panic!("unexpected stream reply"),
    }
}
