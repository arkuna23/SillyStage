mod common;

use std::sync::Arc;

use engine::LlmApiRegistry;
use futures_util::StreamExt;
use protocol::{
    ApiGroupCreateParams, ApiGroupDeleteParams, ApiGroupGetParams, ApiGroupListParams,
    ApiGroupUpdateParams, CharacterArchive, CharacterCardContent, CharacterCoverMimeType,
    CharacterCreateParams, CharacterGetParams, CharacterUpdateParams, CommonVariableDefinition,
    CommonVariableScope, ConfigGetGlobalParams, CreateSessionMessageParams,
    CreateStoryResourcesParams, DashboardGetParams, DataPackageArchive,
    DataPackageExportPrepareParams, DataPackageImportCommitParams, DataPackageImportPrepareParams,
    DeleteSessionCharacterParams, DeleteSessionMessageParams, DeleteSessionParams,
    EnterSessionCharacterSceneParams, ErrorCode, GenerateStoryParams, GetSessionCharacterParams,
    GetSessionMessageParams, GetSessionParams, GetSessionVariablesParams, GetStoryParams,
    GetStoryResourcesParams, JsonRpcOutcome, JsonRpcRequestMessage,
    LeaveSessionCharacterSceneParams, ListSessionCharactersParams, ListSessionMessagesParams,
    LorebookCreateParams, LorebookEntryPayload, LorebookUpdateParams, PresetCreateParams,
    PresetDeleteParams, PresetGetParams, PresetListParams, PresetUpdateParams, RequestParams,
    ResponseResult, RunTurnParams, SchemaCreateParams, SessionMessageKind,
    SessionUpdateConfigParams, StartSessionFromStoryParams, StartStoryDraftParams, StreamEventBody,
    StreamFrame, SuggestRepliesParams, UpdatePlayerDescriptionParams, UpdateSessionCharacterParams,
    UpdateSessionMessageParams, UpdateSessionParams, UpdateSessionVariablesParams,
    UpdateStoryDraftGraphParams, UpdateStoryGraphParams, UpdateStoryParams,
    UpdateStoryResourcesParams,
};
use serde_json::json;
use ss_handler::{Handler, HandlerReply};
use state::{StateFieldSchema, StateOp, StateUpdate, StateValueType};
use store::{
    InMemoryStore, LorebookEntryRecord, LorebookRecord, SessionBindingConfig, SessionRecord, Store,
    StoryDraftRecord, StoryDraftStatus, StoryRecord, StoryResourcesRecord,
};
use story::NarrativeNode;

use common::{
    QueuedMockLlm, assistant_response, sample_api_group_record, sample_api_record, sample_archive,
    sample_blob_record, sample_character_content, sample_character_record, sample_player_profile,
    sample_player_state_schema, sample_preset_record, sample_schema_record,
    sample_session_character_record, sample_story_graph, sample_story_record,
    sample_world_state_schema,
};

fn registry_with_ids(llm: Arc<QueuedMockLlm>) -> LlmApiRegistry {
    let llm: Arc<dyn llm::LlmApi> = llm;

    LlmApiRegistry::new()
        .register("default-planner", Arc::clone(&llm), "planner-default-model")
        .register(
            "default-architect",
            Arc::clone(&llm),
            "architect-default-model",
        )
        .register(
            "default-director",
            Arc::clone(&llm),
            "director-default-model",
        )
        .register("default-actor", Arc::clone(&llm), "actor-default-model")
        .register(
            "default-narrator",
            Arc::clone(&llm),
            "narrator-default-model",
        )
        .register("default-keeper", Arc::clone(&llm), "keeper-default-model")
        .register("default-replyer", Arc::clone(&llm), "replyer-default-model")
        .register("alt-planner", Arc::clone(&llm), "planner-alt-model")
        .register("alt-architect", Arc::clone(&llm), "architect-alt-model")
        .register("alt-director", Arc::clone(&llm), "director-alt-model")
        .register("alt-actor", Arc::clone(&llm), "actor-alt-model")
        .register("alt-narrator", Arc::clone(&llm), "narrator-alt-model")
        .register("alt-keeper", Arc::clone(&llm), "keeper-alt-model")
        .register("alt-replyer", llm, "replyer-alt-model")
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

fn joined_user_messages(request: &llm::ChatRequest) -> String {
    request
        .messages
        .iter()
        .filter(|message| matches!(message.role, llm::Role::User))
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn sample_common_variables() -> Vec<CommonVariableDefinition> {
    vec![
        CommonVariableDefinition {
            scope: CommonVariableScope::World,
            key: "gate_open".to_owned(),
            display_name: "Gate Open".to_owned(),
            character_id: None,
            pinned: true,
        },
        CommonVariableDefinition {
            scope: CommonVariableScope::Player,
            key: "coins".to_owned(),
            display_name: "Coins".to_owned(),
            character_id: None,
            pinned: true,
        },
        CommonVariableDefinition {
            scope: CommonVariableScope::Character,
            key: "trust".to_owned(),
            display_name: "Merchant Trust".to_owned(),
            character_id: Some("merchant".to_owned()),
            pinned: false,
        },
    ]
}

async fn build_handler(llm: Arc<QueuedMockLlm>) -> (Arc<InMemoryStore>, Handler) {
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");
    (store, handler)
}

async fn seed_api_groups_and_presets(store: &InMemoryStore) {
    for role in [
        ("default-planner", "planner-default-model"),
        ("default-architect", "architect-default-model"),
        ("default-director", "director-default-model"),
        ("default-actor", "actor-default-model"),
        ("default-narrator", "narrator-default-model"),
        ("default-keeper", "keeper-default-model"),
        ("default-replyer", "replyer-default-model"),
    ] {
        store
            .save_api(sample_api_record(role.0, "default"))
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

async fn seed_alternate_api_groups_and_presets(store: &InMemoryStore) {
    for role in [
        ("alt-planner", "planner-alt-model"),
        ("alt-architect", "architect-alt-model"),
        ("alt-director", "director-alt-model"),
        ("alt-actor", "actor-alt-model"),
        ("alt-narrator", "narrator-alt-model"),
        ("alt-keeper", "keeper-alt-model"),
        ("alt-replyer", "replyer-alt-model"),
    ] {
        store
            .save_api(sample_api_record(role.0, "alt"))
            .await
            .expect("save alternate api");
    }
    store
        .save_api_group(sample_api_group_record("group-alt", "alt"))
        .await
        .expect("save alternate api group");
    store
        .save_preset(sample_preset_record("preset-alt", 1024))
        .await
        .expect("save alternate preset");
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
    seed_api_groups_and_presets(store).await;
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
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
            lorebook_ids: vec![],
            planned_story: None,
        })
        .await
        .expect("save resources");
    store
        .save_story(sample_story_record("resource-1", "story-1"))
        .await
        .expect("save story");
}

async fn seed_lorebook(store: &InMemoryStore) {
    store
        .save_lorebook(LorebookRecord {
            lorebook_id: "lorebook-harbor".to_owned(),
            display_name: "Harbor Notes".to_owned(),
            entries: vec![LorebookEntryRecord {
                entry_id: "entry-tide".to_owned(),
                title: "Tide".to_owned(),
                content: "The tide is rising around dusk.".to_owned(),
                keywords: vec!["tide".to_owned(), "harbor".to_owned()],
                enabled: true,
                always_include: false,
            }],
        })
        .await
        .expect("save lorebook");
}

async fn export_sample_data_package_bytes(handler: &Handler) -> Vec<u8> {
    let prepared = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-data-package-export",
                None::<String>,
                RequestParams::DataPackageExportPrepare(DataPackageExportPrepareParams {
                    preset_ids: vec!["preset-default".to_owned()],
                    schema_ids: vec![],
                    lorebook_ids: vec![],
                    player_profile_ids: vec!["profile-courier-a".to_owned()],
                    character_ids: vec![],
                    story_resource_ids: vec![],
                    story_ids: vec!["story-1".to_owned()],
                    include_dependencies: true,
                }),
            ))
            .await,
    );

    let archive = match prepared {
        ResponseResult::DataPackageExportPrepared(payload) => {
            assert_eq!(payload.contents.presets.count, 1);
            assert_eq!(payload.contents.player_profiles.count, 1);
            assert_eq!(payload.contents.characters.count, 1);
            assert_eq!(payload.contents.story_resources.count, 1);
            assert_eq!(payload.contents.stories.count, 1);
            assert_eq!(payload.contents.schemas.count, 5);
            assert_eq!(payload.contents.lorebooks.count, 1);
            payload.archive
        }
        other => panic!("unexpected response: {other:?}"),
    };

    handler
        .download_resource_file(&archive.resource_id, &archive.file_id)
        .await
        .expect("package export should download")
        .bytes
}

#[tokio::test]
async fn import_character_card_and_create_resources_via_character_id() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm.clone()).await;
    seed_schema_records(&store).await;
    let archive_bytes = sample_archive()
        .to_chr_bytes()
        .expect("archive should serialize");
    let archive_size = archive_bytes.len() as u64;

    let uploaded = handler
        .upload_resource_file(
            "character:merchant",
            "archive",
            Some("merchant.chr".to_owned()),
            protocol::CHARACTER_ARCHIVE_CONTENT_TYPE.to_owned(),
            archive_bytes,
        )
        .await
        .expect("character should import");
    let character_id = "merchant".to_owned();
    assert_eq!(uploaded.resource_id, "character:merchant");
    assert_eq!(uploaded.file_id, "archive");
    assert_eq!(uploaded.file_name.as_deref(), Some("merchant.chr"));
    assert_eq!(
        uploaded.content_type,
        protocol::CHARACTER_ARCHIVE_CONTENT_TYPE
    );
    assert_eq!(uploaded.size_bytes, archive_size);

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

    let got_cover = handler
        .download_resource_file(&format!("character:{character_id}"), "cover")
        .await
        .expect("cover should be available");
    assert_eq!(got_cover.file_name.as_deref(), Some("cover.png"));
    assert_eq!(got_cover.content_type, "image/png");
    assert_eq!(got_cover.bytes, b"cover-bytes");

    let exported_chr = handler
        .download_resource_file(&format!("character:{character_id}"), "archive")
        .await
        .expect("chr export should succeed");
    assert_eq!(exported_chr.file_name.as_deref(), Some("merchant.chr"));
    assert_eq!(
        exported_chr.content_type,
        "application/x-sillystage-character-card"
    );
    let archive =
        CharacterArchive::from_chr_bytes(&exported_chr.bytes).expect("chr archive should decode");
    assert_eq!(archive.content.id, "merchant");
    assert_eq!(archive.manifest.cover_path, "cover.png");
    assert_eq!(archive.cover_bytes, b"cover-bytes");

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
                    lorebook_ids: vec![],
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

    assert!(matches!(
        handler
            .download_resource_file("character:merchant", "cover")
            .await,
        Err(ss_handler::HandlerError::MissingCharacterCover(_))
    ));
    assert!(matches!(
        handler
            .download_resource_file("character:merchant", "archive")
            .await,
        Err(ss_handler::HandlerError::MissingCharacterCover(_))
    ));

    let updated = handler
        .upload_resource_file(
            "character:merchant",
            "cover",
            None,
            CharacterCoverMimeType::Png.as_content_type().to_owned(),
            b"cover-bytes".to_vec(),
        )
        .await
        .expect("cover should update");
    assert_eq!(updated.resource_id, "character:merchant");
    assert_eq!(updated.file_id, "cover");
    assert_eq!(updated.file_name.as_deref(), Some("cover.png"));
    assert_eq!(
        updated.content_type,
        CharacterCoverMimeType::Png.as_content_type()
    );
    assert_eq!(updated.size_bytes, b"cover-bytes".len() as u64);

    let got_cover = handler
        .download_resource_file("character:merchant", "cover")
        .await
        .expect("cover should be readable");
    assert_eq!(got_cover.bytes, b"cover-bytes");

    let exported_chr = handler
        .download_resource_file("character:merchant", "archive")
        .await
        .expect("chr export should succeed");
    let archive =
        CharacterArchive::from_chr_bytes(&exported_chr.bytes).expect("chr archive should decode");
    assert_eq!(archive.content.id, "merchant");
    assert_eq!(archive.manifest.cover_path, "cover.png");
    assert_eq!(archive.cover_bytes, b"cover-bytes");
}

#[tokio::test]
async fn story_resources_blank_planned_story_is_normalized_to_none() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm).await;
    seed_schema_records(&store).await;
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-blank-create",
                None::<String>,
                RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
                    story_concept: "A flooded harbor story.".to_owned(),
                    character_ids: vec!["merchant".to_owned()],
                    player_schema_id_seed: Some("schema-player-default".to_owned()),
                    world_schema_id_seed: Some("schema-world-default".to_owned()),
                    lorebook_ids: vec![],
                    planned_story: Some("   \n\t".to_owned()),
                }),
            ))
            .await,
    );

    let resource_id = match created {
        ResponseResult::StoryResourcesCreated(payload) => {
            assert_eq!(payload.planned_story, None);
            payload.resource_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let stored = store
        .get_story_resources(&resource_id)
        .await
        .expect("store lookup should succeed")
        .expect("resource should exist");
    assert_eq!(stored.planned_story, None);

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-blank-update",
                None::<String>,
                RequestParams::StoryResourcesUpdate(UpdateStoryResourcesParams {
                    resource_id,
                    story_concept: None,
                    character_ids: None,
                    player_schema_id_seed: None,
                    world_schema_id_seed: None,
                    lorebook_ids: None,
                    planned_story: Some(" \n ".to_owned()),
                }),
            ))
            .await,
    );

    match updated {
        ResponseResult::StoryResourcesUpdated(payload) => {
            assert_eq!(payload.planned_story, None);
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
    handler
        .upload_resource_file(
            "character:merchant",
            "cover",
            None,
            CharacterCoverMimeType::Png.as_content_type().to_owned(),
            b"cover-bytes".to_vec(),
        )
        .await
        .expect("cover should update");

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
                        schema_id: "schema-character-merchant".to_owned(),
                        system_prompt: "Stay in character and watch the waterline.".to_owned(),
                        tags: vec!["merchant".to_owned(), "storm".to_owned()],
                        folder: "harbor".to_owned(),
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

    let got_cover = handler
        .download_resource_file("character:merchant", "cover")
        .await
        .expect("cover should be readable");
    assert_eq!(got_cover.bytes, b"cover-bytes");
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
                    schema_id: "schema-character-merchant".to_owned(),
                    system_prompt: "Stay in character.".to_owned(),
                    tags: vec!["merchant".to_owned()],
                    folder: "harbor".to_owned(),
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
async fn data_package_export_prepare_downloads_zip_with_auto_dependencies() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm).await;
    seed_story_records(&store).await;
    seed_lorebook(&store).await;

    let mut resources = store
        .get_story_resources("resource-1")
        .await
        .expect("store lookup should succeed")
        .expect("story resources should exist");
    resources.lorebook_ids = vec!["lorebook-harbor".to_owned()];
    store
        .save_story_resources(resources)
        .await
        .expect("story resources should update");

    let bytes = export_sample_data_package_bytes(&handler).await;
    let archive = DataPackageArchive::from_zip_bytes(&bytes).expect("archive should decode");

    assert_eq!(archive.presets.len(), 1);
    assert_eq!(archive.presets[0].preset_id, "preset-default");
    assert_eq!(archive.player_profiles.len(), 1);
    assert_eq!(
        archive.player_profiles[0].player_profile_id,
        "profile-courier-a"
    );
    assert_eq!(archive.schemas.len(), 5);
    assert_eq!(archive.lorebooks.len(), 1);
    assert_eq!(archive.characters.len(), 1);
    assert_eq!(
        archive.characters[0].cover_bytes.as_deref(),
        Some(&b"cover-bytes"[..])
    );
    assert_eq!(archive.story_resources.len(), 1);
    assert_eq!(
        archive.story_resources[0].lorebook_ids,
        vec!["lorebook-harbor".to_owned()]
    );
    assert_eq!(archive.stories.len(), 1);
    assert_eq!(archive.stories[0].story_id, "story-1");
}

#[tokio::test]
async fn data_package_import_prepare_upload_and_commit_persist_records() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (source_store, source_handler) = build_handler(llm.clone()).await;
    seed_story_records(&source_store).await;
    seed_lorebook(&source_store).await;
    let mut resources = source_store
        .get_story_resources("resource-1")
        .await
        .expect("store lookup should succeed")
        .expect("story resources should exist");
    resources.lorebook_ids = vec!["lorebook-harbor".to_owned()];
    source_store
        .save_story_resources(resources)
        .await
        .expect("story resources should update");

    let archive_bytes = export_sample_data_package_bytes(&source_handler).await;

    let (target_store, target_handler) = build_handler(llm).await;
    let prepared = unary_result(
        target_handler
            .handle(JsonRpcRequestMessage::new(
                "req-data-package-import-prepare",
                None::<String>,
                RequestParams::DataPackageImportPrepare(DataPackageImportPrepareParams::default()),
            ))
            .await,
    );
    let (import_id, archive_ref) = match prepared {
        ResponseResult::DataPackageImportPrepared(payload) => (payload.import_id, payload.archive),
        other => panic!("unexpected response: {other:?}"),
    };

    let uploaded = target_handler
        .upload_resource_file(
            &archive_ref.resource_id,
            &archive_ref.file_id,
            Some("harbor-package.zip".to_owned()),
            protocol::DATA_PACKAGE_ARCHIVE_CONTENT_TYPE.to_owned(),
            archive_bytes,
        )
        .await
        .expect("package upload should succeed");
    assert_eq!(uploaded.file_name.as_deref(), Some("harbor-package.zip"));
    assert_eq!(
        uploaded.content_type,
        protocol::DATA_PACKAGE_ARCHIVE_CONTENT_TYPE
    );

    let committed = unary_result(
        target_handler
            .handle(JsonRpcRequestMessage::new(
                "req-data-package-import-commit",
                None::<String>,
                RequestParams::DataPackageImportCommit(DataPackageImportCommitParams {
                    import_id: import_id.clone(),
                }),
            ))
            .await,
    );

    match committed {
        ResponseResult::DataPackageImportCommitted(payload) => {
            assert_eq!(payload.import_id, import_id);
            assert_eq!(payload.contents.stories.count, 1);
            assert_eq!(payload.contents.story_resources.count, 1);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    assert!(
        target_store
            .get_preset("preset-default")
            .await
            .expect("store lookup should succeed")
            .is_some()
    );
    assert!(
        target_store
            .get_lorebook("lorebook-harbor")
            .await
            .expect("store lookup should succeed")
            .is_some()
    );
    assert!(
        target_store
            .get_player_profile("profile-courier-a")
            .await
            .expect("store lookup should succeed")
            .is_some()
    );
    assert!(
        target_store
            .get_story_resources("resource-1")
            .await
            .expect("store lookup should succeed")
            .is_some()
    );
    assert!(
        target_store
            .get_story("story-1")
            .await
            .expect("store lookup should succeed")
            .is_some()
    );
    assert!(
        target_store
            .get_schema("schema-character-merchant")
            .await
            .expect("store lookup should succeed")
            .is_some()
    );

    let imported_character = unary_result(
        target_handler
            .handle(JsonRpcRequestMessage::new(
                "req-character-get-imported",
                None::<String>,
                RequestParams::CharacterGet(CharacterGetParams {
                    character_id: "merchant".to_owned(),
                }),
            ))
            .await,
    );
    match imported_character {
        ResponseResult::Character(payload) => {
            assert_eq!(payload.content.name, "Haru");
            assert_eq!(payload.cover_file_name.as_deref(), Some("cover.png"));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let imported_story = unary_result(
        target_handler
            .handle(JsonRpcRequestMessage::new(
                "req-story-get-imported",
                None::<String>,
                RequestParams::StoryGet(GetStoryParams {
                    story_id: "story-1".to_owned(),
                }),
            ))
            .await,
    );
    assert!(matches!(imported_story, ResponseResult::Story(_)));

    let cover = target_handler
        .download_resource_file("character:merchant", "cover")
        .await
        .expect("cover should be downloadable");
    assert_eq!(cover.bytes, b"cover-bytes");
}

#[tokio::test]
async fn data_package_import_conflict_returns_conflict_without_partial_import() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (source_store, source_handler) = build_handler(llm.clone()).await;
    seed_story_records(&source_store).await;
    seed_lorebook(&source_store).await;
    let mut resources = source_store
        .get_story_resources("resource-1")
        .await
        .expect("store lookup should succeed")
        .expect("story resources should exist");
    resources.lorebook_ids = vec!["lorebook-harbor".to_owned()];
    source_store
        .save_story_resources(resources)
        .await
        .expect("story resources should update");
    let archive_bytes = export_sample_data_package_bytes(&source_handler).await;

    let (target_store, target_handler) = build_handler(llm).await;
    target_store
        .save_story(sample_story_record("resource-existing", "story-1"))
        .await
        .expect("duplicate story should seed");

    let prepared = unary_result(
        target_handler
            .handle(JsonRpcRequestMessage::new(
                "req-data-package-import-prepare-conflict",
                None::<String>,
                RequestParams::DataPackageImportPrepare(DataPackageImportPrepareParams::default()),
            ))
            .await,
    );
    let import_id = match prepared {
        ResponseResult::DataPackageImportPrepared(payload) => {
            target_handler
                .upload_resource_file(
                    &payload.archive.resource_id,
                    &payload.archive.file_id,
                    Some("harbor-package.zip".to_owned()),
                    protocol::DATA_PACKAGE_ARCHIVE_CONTENT_TYPE.to_owned(),
                    archive_bytes,
                )
                .await
                .expect("package upload should succeed");
            payload.import_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = match target_handler
        .handle(JsonRpcRequestMessage::new(
            "req-data-package-import-commit-conflict",
            None::<String>,
            RequestParams::DataPackageImportCommit(DataPackageImportCommitParams { import_id }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error) if error.code == ErrorCode::Conflict.rpc_code()
    ));
    assert!(
        target_store
            .get_preset("preset-default")
            .await
            .expect("store lookup should succeed")
            .is_none()
    );
    assert!(
        target_store
            .get_character("merchant")
            .await
            .expect("store lookup should succeed")
            .is_none()
    );
    assert!(
        target_store
            .get_story_resources("resource-1")
            .await
            .expect("store lookup should succeed")
            .is_none()
    );
}

#[tokio::test]
async fn data_package_import_commit_rejects_invalid_archive_bytes() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (_store, handler) = build_handler(llm).await;

    let prepared = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-data-package-import-prepare-invalid",
                None::<String>,
                RequestParams::DataPackageImportPrepare(DataPackageImportPrepareParams::default()),
            ))
            .await,
    );
    let import_id = match prepared {
        ResponseResult::DataPackageImportPrepared(payload) => {
            handler
                .upload_resource_file(
                    &payload.archive.resource_id,
                    &payload.archive.file_id,
                    Some("broken.zip".to_owned()),
                    protocol::DATA_PACKAGE_ARCHIVE_CONTENT_TYPE.to_owned(),
                    b"not-a-zip".to_vec(),
                )
                .await
                .expect("upload should succeed");
            payload.import_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-data-package-import-commit-invalid",
            None::<String>,
            RequestParams::DataPackageImportCommit(DataPackageImportCommitParams { import_id }),
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
    seed_api_groups_and_presets(&store).await;
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");

    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()))
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
                    lorebook_ids: vec![],
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
                    api_group_id: None,
                    preset_id: None,
                    common_variables: None,
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
                api_group_id: None,
                preset_id: None,
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
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-update",
                None::<String>,
                RequestParams::StoryUpdate(UpdateStoryParams {
                    story_id: "story-1".to_owned(),
                    display_name: Some("Updated Flooded Harbor".to_owned()),
                    common_variables: None,
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
async fn story_generate_persists_common_variables() {
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
    seed_api_groups_and_presets(&store).await;
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
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
            lorebook_ids: vec![],
            planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
        })
        .await
        .expect("save resources");
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");
    let common_variables = sample_common_variables();

    let generated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-generate-common-variables",
                None::<String>,
                RequestParams::StoryGenerate(GenerateStoryParams {
                    resource_id: "resource-1".to_owned(),
                    display_name: Some("Flooded Harbor".to_owned()),
                    api_group_id: None,
                    preset_id: None,
                    common_variables: Some(common_variables.clone()),
                }),
            ))
            .await,
    );

    let story_id = match generated {
        ResponseResult::StoryGenerated(payload) => {
            assert_eq!(payload.common_variables, common_variables);
            payload.story_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let stored = store
        .get_story(&story_id)
        .await
        .expect("story lookup should succeed")
        .expect("story should exist");
    assert_eq!(stored.common_variables, common_variables);
}

#[tokio::test]
async fn story_generate_rejects_invalid_common_variables() {
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
    seed_api_groups_and_presets(&store).await;
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
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
            lorebook_ids: vec![],
            planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
        })
        .await
        .expect("save resources");
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "story-generate-invalid-common-variables",
            None::<String>,
            RequestParams::StoryGenerate(GenerateStoryParams {
                resource_id: "resource-1".to_owned(),
                display_name: Some("Flooded Harbor".to_owned()),
                api_group_id: None,
                preset_id: None,
                common_variables: Some(vec![CommonVariableDefinition {
                    scope: CommonVariableScope::World,
                    key: "storm_level".to_owned(),
                    display_name: "Storm Level".to_owned(),
                    character_id: None,
                    pinned: true,
                }]),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("does not exist in the bound schema")
    ));
    assert!(
        store
            .list_stories()
            .await
            .expect("stories should list")
            .is_empty(),
        "invalid common variables should not create a story"
    );
    assert!(
        store
            .list_story_drafts()
            .await
            .expect("drafts should list")
            .is_empty(),
        "invalid common variables should not create a draft"
    );
}

#[tokio::test]
async fn story_draft_start_persists_common_variables() {
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
    seed_api_groups_and_presets(&store).await;
    store
        .save_blob(sample_blob_record())
        .await
        .expect("save blob");
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
            lorebook_ids: vec![],
            planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
        })
        .await
        .expect("save resources");
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");
    let common_variables = sample_common_variables();

    let started = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-draft-start-common-variables",
                None::<String>,
                RequestParams::StoryDraftStart(StartStoryDraftParams {
                    resource_id: "resource-1".to_owned(),
                    display_name: Some("Flooded Harbor Draft".to_owned()),
                    api_group_id: None,
                    preset_id: None,
                    common_variables: Some(common_variables.clone()),
                }),
            ))
            .await,
    );

    let draft_id = match started {
        ResponseResult::StoryDraft(payload) => {
            assert_eq!(payload.common_variables, common_variables);
            payload.draft_id
        }
        other => panic!("unexpected response: {other:?}"),
    };

    let stored = store
        .get_story_draft(&draft_id)
        .await
        .expect("draft lookup should succeed")
        .expect("draft should exist");
    assert_eq!(stored.common_variables, common_variables);
}

#[tokio::test]
async fn story_update_persists_common_variables() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");
    let common_variables = sample_common_variables();

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-update-common-variables",
                None::<String>,
                RequestParams::StoryUpdate(UpdateStoryParams {
                    story_id: "story-1".to_owned(),
                    display_name: None,
                    common_variables: Some(common_variables.clone()),
                }),
            ))
            .await,
    );

    match updated {
        ResponseResult::Story(payload) => {
            assert_eq!(payload.story_id, "story-1");
            assert_eq!(payload.display_name, "Flooded Harbor");
            assert_eq!(payload.common_variables, common_variables);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let stored = store
        .get_story("story-1")
        .await
        .expect("story lookup should succeed")
        .expect("story should exist");
    assert_eq!(stored.common_variables, common_variables);
}

#[tokio::test]
async fn story_update_rejects_character_common_variable_without_character_id() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store, registry_with_ids(llm))
        .await
        .expect("handler should build");

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "story-update-invalid-common-variable-character",
            None::<String>,
            RequestParams::StoryUpdate(UpdateStoryParams {
                story_id: "story-1".to_owned(),
                display_name: None,
                common_variables: Some(vec![CommonVariableDefinition {
                    scope: CommonVariableScope::Character,
                    key: "trust".to_owned(),
                    display_name: "Merchant Trust".to_owned(),
                    character_id: None,
                    pinned: true,
                }]),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("must set character_id")
    ));
}

#[tokio::test]
async fn story_update_rejects_common_variable_for_missing_schema_key() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store, registry_with_ids(llm))
        .await
        .expect("handler should build");

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "story-update-invalid-common-variable-key",
            None::<String>,
            RequestParams::StoryUpdate(UpdateStoryParams {
                story_id: "story-1".to_owned(),
                display_name: None,
                common_variables: Some(vec![CommonVariableDefinition {
                    scope: CommonVariableScope::World,
                    key: "storm_level".to_owned(),
                    display_name: "Storm Level".to_owned(),
                    character_id: None,
                    pinned: true,
                }]),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("does not exist in the bound schema")
    ));
}

#[tokio::test]
async fn schema_create_rejects_invalid_enum_values_definition() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let (store, handler) = build_handler(llm).await;
    seed_story_records(&store).await;

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "schema-create-invalid-enum",
            None::<String>,
            RequestParams::SchemaCreate(SchemaCreateParams {
                schema_id: "schema-invalid-enum".to_owned(),
                display_name: "Invalid Enum".to_owned(),
                tags: vec!["player".to_owned()],
                fields: [(
                    "route".to_owned(),
                    StateFieldSchema::new(StateValueType::Array)
                        .with_enum_values(vec![json!(["dock"]), json!(["tower"])]),
                )]
                .into_iter()
                .collect(),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("enum_values")
                && error.message.contains("scalar")
    ));
}

#[tokio::test]
async fn story_update_graph_replaces_story_graph() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let mut graph = sample_story_graph();
    graph.nodes.push(NarrativeNode::new(
        "gate",
        "Canal Gate",
        "A narrow ledge beside the gate.",
        "Open the route.",
        vec!["merchant".to_owned()],
        vec![],
        vec![StateOp::SetPlayerState {
            key: "coins".to_owned(),
            value: json!(5),
        }],
    ));
    graph.start_node = "gate".to_owned();

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-update-graph",
                None::<String>,
                RequestParams::StoryUpdateGraph(UpdateStoryGraphParams {
                    story_id: "story-1".to_owned(),
                    graph: graph.clone(),
                }),
            ))
            .await,
    );

    match updated {
        ResponseResult::Story(payload) => {
            assert_eq!(payload.story_id, "story-1");
            assert_eq!(payload.graph.start_node, "gate");
            assert!(matches!(
                &payload.graph.nodes[1].on_enter_updates[..],
                [StateOp::SetPlayerState { key, value }]
                    if key == "coins" && value == &json!(5)
            ));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let stored = store
        .get_story("story-1")
        .await
        .expect("story lookup should succeed")
        .expect("story should exist");
    assert_eq!(stored.graph.start_node, "gate");
    assert!(matches!(
        &stored.graph.nodes[1].on_enter_updates[..],
        [StateOp::SetPlayerState { key, value }]
            if key == "coins" && value == &json!(5)
    ));
}

#[tokio::test]
async fn story_update_graph_rejects_invalid_graph() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store, registry_with_ids(llm))
        .await
        .expect("handler should build");

    let mut graph = sample_story_graph();
    graph.start_node = "missing".to_owned();

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "story-update-graph-invalid",
            None::<String>,
            RequestParams::StoryUpdateGraph(UpdateStoryGraphParams {
                story_id: "story-1".to_owned(),
                graph,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("start node")
    ));
}

#[tokio::test]
async fn story_update_graph_rejects_noncanonical_identifier_values() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store, registry_with_ids(llm))
        .await
        .expect("handler should build");

    let mut graph = sample_story_graph();
    graph.nodes[0].on_enter_updates = vec![StateOp::SetState {
        key: "current_event".to_owned(),
        value: json!("接近沼泽"),
    }];

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "story-update-graph-current-event-invalid",
            None::<String>,
            RequestParams::StoryUpdateGraph(UpdateStoryGraphParams {
                story_id: "story-1".to_owned(),
                graph,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("current_event")
                && error.message.contains("canonical snake_case identifier")
    ));
}

#[tokio::test]
async fn story_draft_update_graph_replaces_partial_graph() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    store
        .save_story_draft(StoryDraftRecord {
            draft_id: "draft-1".to_owned(),
            display_name: "Flooded Harbor Draft".to_owned(),
            resource_id: "resource-1".to_owned(),
            api_group_id: "group-default".to_owned(),
            preset_id: "preset-default".to_owned(),
            planned_story: "Opening section".to_owned(),
            outline_sections: vec!["Opening section".to_owned()],
            next_section_index: 0,
            partial_graph: sample_story_graph(),
            world_schema_id: "schema-world-default".to_owned(),
            player_schema_id: "schema-player-default".to_owned(),
            introduction: "Draft intro".to_owned(),
            common_variables: vec![],
            section_summaries: vec![],
            section_node_ids: vec![],
            status: StoryDraftStatus::Building,
            final_story_id: None,
            created_at_ms: Some(1_000),
            updated_at_ms: Some(2_000),
        })
        .await
        .expect("save draft");
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let mut partial_graph = sample_story_graph();
    partial_graph.nodes.push(NarrativeNode::new(
        "gate",
        "Canal Gate",
        "A narrow ledge beside the gate.",
        "Open the route.",
        vec!["merchant".to_owned()],
        vec![],
        vec![StateOp::SetState {
            key: "gate_open".to_owned(),
            value: json!(true),
        }],
    ));
    partial_graph.start_node = "gate".to_owned();

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "story-draft-update-graph",
                None::<String>,
                RequestParams::StoryDraftUpdateGraph(UpdateStoryDraftGraphParams {
                    draft_id: "draft-1".to_owned(),
                    partial_graph: partial_graph.clone(),
                }),
            ))
            .await,
    );

    match updated {
        ResponseResult::StoryDraft(payload) => {
            assert_eq!(payload.draft_id, "draft-1");
            assert_eq!(payload.partial_graph.start_node, "gate");
            assert!(matches!(
                &payload.partial_graph.nodes[1].on_enter_updates[..],
                [StateOp::SetState { key, value }]
                    if key == "gate_open" && value == &json!(true)
            ));
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let stored = store
        .get_story_draft("draft-1")
        .await
        .expect("draft lookup should succeed")
        .expect("draft should exist");
    assert_eq!(stored.partial_graph.start_node, "gate");
    assert!(matches!(
        &stored.partial_graph.nodes[1].on_enter_updates[..],
        [StateOp::SetState { key, value }]
            if key == "gate_open" && value == &json!(true)
    ));
}

#[tokio::test]
async fn story_draft_update_graph_rejects_finalized_draft() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    store
        .save_story_draft(StoryDraftRecord {
            draft_id: "draft-1".to_owned(),
            display_name: "Flooded Harbor Draft".to_owned(),
            resource_id: "resource-1".to_owned(),
            api_group_id: "group-default".to_owned(),
            preset_id: "preset-default".to_owned(),
            planned_story: "Opening section".to_owned(),
            outline_sections: vec!["Opening section".to_owned()],
            next_section_index: 1,
            partial_graph: sample_story_graph(),
            world_schema_id: "schema-world-default".to_owned(),
            player_schema_id: "schema-player-default".to_owned(),
            introduction: "Draft intro".to_owned(),
            common_variables: vec![],
            section_summaries: vec!["Opening done".to_owned()],
            section_node_ids: vec![vec!["dock".to_owned()]],
            status: StoryDraftStatus::Finalized,
            final_story_id: Some("story-1".to_owned()),
            created_at_ms: Some(1_000),
            updated_at_ms: Some(2_000),
        })
        .await
        .expect("save draft");
    let handler = Handler::new(store, registry_with_ids(llm))
        .await
        .expect("handler should build");

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "story-draft-update-graph-finalized",
            None::<String>,
            RequestParams::StoryDraftUpdateGraph(UpdateStoryDraftGraphParams {
                draft_id: "draft-1".to_owned(),
                partial_graph: sample_story_graph(),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("already finalized")
    ));
}

#[tokio::test]
async fn session_update_changes_display_name() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
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
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
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
async fn session_character_crud_round_trips_and_tracks_scene_presence() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary session start"),
    };
    let session_id = started.session_id.expect("session id should exist");

    store
        .save_session_character(sample_session_character_record(&session_id, "dock_guard"))
        .await
        .expect("save session character");

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-character-get",
                Some(session_id.clone()),
                RequestParams::SessionCharacterGet(GetSessionCharacterParams {
                    session_character_id: "dock_guard".to_owned(),
                }),
            ))
            .await,
    );
    match fetched {
        ResponseResult::SessionCharacter(payload) => {
            assert_eq!(payload.session_character_id, "dock_guard");
            assert_eq!(payload.display_name, "Dock Guard");
            assert!(!payload.in_scene);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let listed = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-character-list",
                Some(session_id.clone()),
                RequestParams::SessionCharacterList(ListSessionCharactersParams::default()),
            ))
            .await,
    );
    match listed {
        ResponseResult::SessionCharactersListed(payload) => {
            assert_eq!(payload.session_characters.len(), 1);
            assert_eq!(
                payload.session_characters[0].session_character_id,
                "dock_guard"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-character-update",
                Some(session_id.clone()),
                RequestParams::SessionCharacterUpdate(UpdateSessionCharacterParams {
                    session_character_id: "dock_guard".to_owned(),
                    display_name: "Harbormaster".to_owned(),
                    personality: "stern but fair".to_owned(),
                    style: "measured, formal".to_owned(),
                    system_prompt: "Manage the harbor with strict discipline.".to_owned(),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::SessionCharacter(payload) => {
            assert_eq!(payload.display_name, "Harbormaster");
            assert_eq!(payload.personality, "stern but fair");
            assert!(!payload.in_scene);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let entered = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-character-enter",
                Some(session_id.clone()),
                RequestParams::SessionCharacterEnterScene(EnterSessionCharacterSceneParams {
                    session_character_id: "dock_guard".to_owned(),
                }),
            ))
            .await,
    );
    match entered {
        ResponseResult::SessionCharacter(payload) => {
            assert_eq!(payload.session_character_id, "dock_guard");
            assert!(payload.in_scene);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let session = store
        .get_session(&session_id)
        .await
        .expect("load session")
        .expect("session should exist");
    assert!(
        session
            .snapshot
            .world_state
            .active_characters()
            .iter()
            .any(|id| id == "dock_guard")
    );

    let left = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-character-leave",
                Some(session_id.clone()),
                RequestParams::SessionCharacterLeaveScene(LeaveSessionCharacterSceneParams {
                    session_character_id: "dock_guard".to_owned(),
                }),
            ))
            .await,
    );
    match left {
        ResponseResult::SessionCharacter(payload) => {
            assert_eq!(payload.session_character_id, "dock_guard");
            assert!(!payload.in_scene);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let deleted = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "session-character-delete",
                Some(session_id.clone()),
                RequestParams::SessionCharacterDelete(DeleteSessionCharacterParams {
                    session_character_id: "dock_guard".to_owned(),
                }),
            ))
            .await,
    );
    match deleted {
        ResponseResult::SessionCharacterDeleted(payload) => {
            assert_eq!(payload.session_character_id, "dock_guard");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    assert!(
        store
            .get_session_character("dock_guard")
            .await
            .expect("load session character")
            .is_none()
    );
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
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
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
                RequestParams::SessionSuggestReplies(SuggestRepliesParams { limit: Some(3) }),
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
async fn session_suggest_replies_uses_only_recent_eight_messages() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![Ok(assistant_response(
            "{}",
            Some(json!({
                "replies": [
                    { "id": "r1", "text": "Tell me what changed." }
                ]
            })),
        ))],
        vec![],
    ));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()))
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
                api_group_id: None,
                preset_id: None,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary session start"),
    };
    let session_id = started.session_id.expect("session id should exist");

    for sequence in 0..10_u64 {
        store
            .save_session_message(store::SessionMessageRecord {
                message_id: format!("message-{sequence}"),
                session_id: session_id.clone(),
                kind: store::SessionMessageKind::Dialogue,
                sequence,
                turn_index: sequence,
                recorded_at_ms: 5_000 + sequence,
                created_at_ms: 5_000 + sequence,
                updated_at_ms: 5_000 + sequence,
                speaker_id: "merchant".to_owned(),
                speaker_name: "Haru".to_owned(),
                text: format!("history line {sequence}"),
            })
            .await
            .expect("save session message");
    }

    let suggested = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "suggest-replies-recent-window",
                Some(session_id),
                RequestParams::SessionSuggestReplies(SuggestRepliesParams { limit: Some(3) }),
            ))
            .await,
    );

    match suggested {
        ResponseResult::SuggestedReplies(payload) => {
            assert_eq!(payload.replies.len(), 1);
            assert_eq!(payload.replies[0].text, "Tell me what changed.");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let requests = llm.recorded_requests();
    let user_message = joined_user_messages(requests.first().expect("request should exist"));
    assert!(!user_message.contains("history line 0"));
    assert!(!user_message.contains("history line 1"));
    assert!(user_message.contains("history line 2"));
    assert!(user_message.contains("history line 9"));
}

#[tokio::test]
async fn session_config_can_switch_between_session_and_global_modes() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    seed_alternate_api_groups_and_presets(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()))
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
                api_group_id: Some("group-alt".to_owned()),
                preset_id: Some("preset-alt".to_owned()),
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
    assert_eq!(config.api_group_id, "group-alt");
    assert_eq!(config.preset_id, "preset-alt");

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
            assert_eq!(payload.api_group_id, "group-alt");
            assert_eq!(payload.preset_id, "preset-alt");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let session_use_global = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-4",
                Some(session_id.clone()),
                RequestParams::SessionUpdateConfig(SessionUpdateConfigParams {
                    api_group_id: Some("group-default".to_owned()),
                    preset_id: Some("preset-default".to_owned()),
                }),
            ))
            .await,
    );
    match session_use_global {
        ResponseResult::SessionConfig(payload) => {
            assert_eq!(payload.api_group_id, "group-default");
            assert_eq!(payload.preset_id, "preset-default");
        }
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn dashboard_get_returns_counts_global_config_and_recent_lists() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_schema_records(&store).await;
    seed_api_groups_and_presets(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    store
        .save_blob(sample_blob_record())
        .await
        .expect("save covered blob");
    let character_with_cover = sample_character_record();
    let mut character_without_cover = sample_character_record();
    character_without_cover.character_id = "guard".to_owned();
    character_without_cover.content.id = "guard".to_owned();
    character_without_cover.content.name = "Mina".to_owned();
    character_without_cover.cover_blob_id = None;
    character_without_cover.cover_file_name = None;
    character_without_cover.cover_mime_type = None;

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
                lorebook_ids: vec![],
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
                common_variables: vec![],
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
                binding: SessionBindingConfig {
                    api_group_id: "group-default".to_owned(),
                    preset_id: "preset-default".to_owned(),
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
            assert_eq!(
                payload.global_config.api_group_id.as_deref(),
                Some("group-default")
            );
            assert_eq!(
                payload.global_config.preset_id.as_deref(),
                Some("preset-default")
            );
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
    let handler = Handler::new(store, registry_with_ids(llm))
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
            assert!(payload.global_config.api_group_id.is_none());
            assert!(payload.global_config.preset_id.is_none());
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
            assert!(payload.api_group_id.is_none());
            assert!(payload.preset_id.is_none());
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
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()))
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
                api_group_id: None,
                preset_id: None,
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
async fn run_turn_stream_creates_and_activates_session_character_before_actor_beats() {
    let llm = Arc::new(QueuedMockLlm::new(
        vec![
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({
                    "ops": []
                })),
            )),
            Ok(assistant_response(
                "{\"role_actions\":[{\"type\":\"create_and_enter\",\"session_character_id\":\"dock_guard\",\"display_name\":\"Dock Guard\",\"personality\":\"dutiful and wary\",\"style\":\"short, formal\",\"system_prompt\":\"Keep watch over the dock.\"}],\"beats\":[{\"type\":\"Actor\",\"speaker_id\":\"dock_guard\",\"purpose\":\"AdvanceGoal\"}]}",
                Some(json!({
                    "role_actions": [
                        {
                            "type": "create_and_enter",
                            "session_character_id": "dock_guard",
                            "display_name": "Dock Guard",
                            "personality": "dutiful and wary",
                            "style": "short, formal",
                            "system_prompt": "Keep watch over the dock."
                        }
                    ],
                    "beats": [
                        {
                            "type": "Actor",
                            "speaker_id": "dock_guard",
                            "purpose": "AdvanceGoal"
                        }
                    ]
                })),
            )),
            Ok(assistant_response(
                "{\"ops\":[]}",
                Some(json!({
                    "ops": []
                })),
            )),
        ],
        vec![Ok(vec![
            Ok(llm::ChatChunk {
                delta: "<dialogue>State your business.</dialogue>".to_owned(),
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
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
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
                player_input: "Who's on watch tonight?".to_owned(),
            }),
        ))
        .await;

    let (_, events) = match reply {
        HandlerReply::Stream { ack, events } => {
            assert!(matches!(
                &ack.outcome,
                JsonRpcOutcome::Ok(result) if matches!(&**result, ResponseResult::TurnStreamAccepted(_))
            ));
            (ack, events)
        }
        HandlerReply::Unary(_) => panic!("expected stream reply"),
    };
    let frames = events.collect::<Vec<_>>().await;

    let mut saw_created = false;
    let mut saw_entered = false;
    let mut saw_actor_completed = false;
    for frame in &frames {
        match &frame.frame {
            StreamFrame::Event {
                body:
                    StreamEventBody::SessionCharacterCreated {
                        session_character,
                        snapshot,
                    },
            } => {
                saw_created = true;
                assert_eq!(session_character.session_character_id, "dock_guard");
                assert_eq!(session_character.display_name, "Dock Guard");
                assert!(session_character.in_scene);
                assert!(
                    snapshot
                        .world_state
                        .active_characters()
                        .iter()
                        .any(|id| id == "dock_guard")
                );
            }
            StreamFrame::Event {
                body:
                    StreamEventBody::SessionCharacterEnteredScene {
                        session_character_id,
                        snapshot,
                    },
            } => {
                saw_entered = true;
                assert_eq!(session_character_id, "dock_guard");
                assert!(
                    snapshot
                        .world_state
                        .active_characters()
                        .iter()
                        .any(|id| id == "dock_guard")
                );
            }
            StreamFrame::Event {
                body:
                    StreamEventBody::ActorCompleted {
                        speaker_id,
                        response,
                        ..
                    },
            } => {
                if speaker_id == "dock_guard" {
                    saw_actor_completed = true;
                    assert_eq!(response.speaker_name, "Dock Guard");
                }
            }
            _ => {}
        }
    }
    assert!(saw_created);
    assert!(saw_entered);
    assert!(saw_actor_completed);

    let saved_character = store
        .get_session_character("dock_guard")
        .await
        .expect("load session character")
        .expect("session character should be saved");
    assert_eq!(saved_character.session_id, session_id);
    assert_eq!(saved_character.display_name, "Dock Guard");

    let saved_session = store
        .get_session(&session_id)
        .await
        .expect("load session")
        .expect("session should exist");
    assert!(
        saved_session
            .snapshot
            .world_state
            .active_characters()
            .iter()
            .any(|id| id == "dock_guard")
    );

    let mut messages = store
        .list_session_messages(&session_id)
        .await
        .expect("load session messages");
    messages.sort_by_key(|message| message.sequence);
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].speaker_id, "player");
    assert_eq!(messages[1].speaker_id, "dock_guard");
    assert_eq!(messages[1].speaker_name, "Dock Guard");
    assert_eq!(messages[1].text, "State your business.");
}

#[tokio::test]
async fn update_player_description_persists_to_session_snapshot() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    seed_alternate_api_groups_and_presets(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm.clone()))
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
                api_group_id: None,
                preset_id: None,
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
async fn session_variables_get_and_update_round_trip() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary session start"),
    };
    let session_id = started.session_id.expect("session id should exist");

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-get-variables",
                Some(session_id.clone()),
                RequestParams::SessionGetVariables(GetSessionVariablesParams::default()),
            ))
            .await,
    );
    match fetched {
        ResponseResult::SessionVariables(payload) => {
            assert!(payload.custom.is_empty());
            assert!(payload.player_state.is_empty());
            assert!(payload.character_state.is_empty());
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "req-update-variables",
                Some(session_id.clone()),
                RequestParams::SessionUpdateVariables(UpdateSessionVariablesParams {
                    update: StateUpdate::new()
                        .push(StateOp::SetState {
                            key: "gate_open".to_owned(),
                            value: json!(true),
                        })
                        .push(StateOp::SetPlayerState {
                            key: "coins".to_owned(),
                            value: json!(11),
                        })
                        .push(StateOp::SetCharacterState {
                            character: "merchant".to_owned(),
                            key: "trust".to_owned(),
                            value: json!(4),
                        }),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::SessionVariables(payload) => {
            assert_eq!(payload.custom.get("gate_open"), Some(&json!(true)));
            assert_eq!(payload.player_state.get("coins"), Some(&json!(11)));
            assert_eq!(
                payload
                    .character_state
                    .get("merchant")
                    .and_then(|state| state.get("trust")),
                Some(&json!(4))
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let session = store
        .get_session(&session_id)
        .await
        .expect("load session")
        .expect("session exists");
    assert_eq!(
        session.snapshot.world_state.state("gate_open"),
        Some(&json!(true))
    );
    assert_eq!(
        session.snapshot.world_state.player_state("coins"),
        Some(&json!(11))
    );
    assert_eq!(
        session
            .snapshot
            .world_state
            .character_state("merchant", "trust"),
        Some(&json!(4))
    );
}

#[tokio::test]
async fn session_variable_update_rejects_non_variable_ops() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
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
                api_group_id: None,
                preset_id: None,
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary session start"),
    };
    let session_id = started.session_id.expect("session id should exist");

    let response = match handler
        .handle(JsonRpcRequestMessage::new(
            "req-update-variables",
            Some(session_id),
            RequestParams::SessionUpdateVariables(UpdateSessionVariablesParams {
                update: StateUpdate::new().push(StateOp::SetCurrentNode {
                    node_id: "gate".to_owned(),
                }),
            }),
        ))
        .await
    {
        HandlerReply::Unary(response) => response,
        HandlerReply::Stream { .. } => panic!("expected unary response"),
    };

    assert!(matches!(
        response.outcome,
        JsonRpcOutcome::Err(error)
            if error.code == ErrorCode::InvalidRequest.rpc_code()
                && error.message.contains("SetCurrentNode")
    ));
}

#[tokio::test]
async fn lorebook_update_changes_base_info_and_preserves_entries() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "lorebook-create",
                None::<String>,
                RequestParams::LorebookCreate(LorebookCreateParams {
                    lorebook_id: "harbor".to_owned(),
                    display_name: "Harbor Notes".to_owned(),
                    entries: vec![LorebookEntryPayload {
                        entry_id: "fog".to_owned(),
                        title: "Fog".to_owned(),
                        content: "The harbor is covered by cold fog.".to_owned(),
                        keywords: vec!["fog".to_owned(), "harbor".to_owned()],
                        enabled: true,
                        always_include: false,
                    }],
                }),
            ))
            .await,
    );
    match created {
        ResponseResult::Lorebook(payload) => {
            assert_eq!(payload.lorebook_id, "harbor");
            assert_eq!(payload.display_name, "Harbor Notes");
            assert_eq!(payload.entries.len(), 1);
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "lorebook-update",
                None::<String>,
                RequestParams::LorebookUpdate(LorebookUpdateParams {
                    lorebook_id: "harbor".to_owned(),
                    display_name: Some("Updated Harbor Notes".to_owned()),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::Lorebook(payload) => {
            assert_eq!(payload.lorebook_id, "harbor");
            assert_eq!(payload.display_name, "Updated Harbor Notes");
            assert_eq!(payload.entries.len(), 1);
            assert_eq!(payload.entries[0].entry_id, "fog");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let stored = store
        .get_lorebook("harbor")
        .await
        .expect("lorebook should load")
        .expect("lorebook should exist");
    assert_eq!(stored.display_name, "Updated Harbor Notes");
    assert_eq!(stored.entries.len(), 1);
    assert_eq!(stored.entries[0].entry_id, "fog");
}

#[tokio::test]
async fn api_group_crud_masks_keys_and_round_trips() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");
    for (api_id, suffix) in [
        ("managed-planner", "managed"),
        ("managed-architect", "managed"),
        ("managed-director", "managed"),
        ("managed-actor", "managed"),
        ("managed-narrator", "managed"),
        ("managed-keeper", "managed"),
        ("managed-replyer", "managed"),
        ("updated-planner", "updated"),
        ("updated-architect", "updated"),
        ("updated-director", "updated"),
        ("updated-actor", "updated"),
        ("updated-narrator", "updated"),
        ("updated-keeper", "updated"),
        ("updated-replyer", "updated"),
    ] {
        store
            .save_api(sample_api_record(api_id, suffix))
            .await
            .expect("save api");
    }

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "api-group-create",
                None::<String>,
                RequestParams::ApiGroupCreate(ApiGroupCreateParams {
                    api_group_id: "managed".to_owned(),
                    display_name: "Managed Group".to_owned(),
                    bindings: protocol::ApiGroupBindingsInput {
                        planner_api_id: "managed-planner".to_owned(),
                        architect_api_id: "managed-architect".to_owned(),
                        director_api_id: "managed-director".to_owned(),
                        actor_api_id: "managed-actor".to_owned(),
                        narrator_api_id: "managed-narrator".to_owned(),
                        keeper_api_id: "managed-keeper".to_owned(),
                        replyer_api_id: "managed-replyer".to_owned(),
                    },
                }),
            ))
            .await,
    );
    match created {
        ResponseResult::ApiGroup(payload) => {
            assert_eq!(payload.api_group_id, "managed");
            assert_eq!(payload.display_name, "Managed Group");
            assert_eq!(payload.bindings.actor_api_id, "managed-actor");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "api-group-get",
                None::<String>,
                RequestParams::ApiGroupGet(ApiGroupGetParams {
                    api_group_id: "managed".to_owned(),
                }),
            ))
            .await,
    );
    match fetched {
        ResponseResult::ApiGroup(payload) => {
            assert_eq!(payload.bindings.actor_api_id, "managed-actor");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let listed = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "api-group-list",
                None::<String>,
                RequestParams::ApiGroupList(ApiGroupListParams::default()),
            ))
            .await,
    );
    match listed {
        ResponseResult::ApiGroupsListed(payload) => {
            assert!(
                payload
                    .api_groups
                    .iter()
                    .any(|group| group.api_group_id == "managed")
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "api-group-update",
                None::<String>,
                RequestParams::ApiGroupUpdate(ApiGroupUpdateParams {
                    api_group_id: "managed".to_owned(),
                    display_name: Some("Updated Group".to_owned()),
                    bindings: Some(protocol::ApiGroupBindingsInput {
                        planner_api_id: "updated-planner".to_owned(),
                        architect_api_id: "updated-architect".to_owned(),
                        director_api_id: "updated-director".to_owned(),
                        actor_api_id: "updated-actor".to_owned(),
                        narrator_api_id: "updated-narrator".to_owned(),
                        keeper_api_id: "updated-keeper".to_owned(),
                        replyer_api_id: "updated-replyer".to_owned(),
                    }),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::ApiGroup(payload) => {
            assert_eq!(payload.display_name, "Updated Group");
            assert_eq!(payload.bindings.actor_api_id, "updated-actor");
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let stored = store
        .get_api_group("managed")
        .await
        .expect("api group should load")
        .expect("api group should exist");
    assert_eq!(stored.agents.actor_api_id, "updated-actor");

    let deleted = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "api-group-delete",
                None::<String>,
                RequestParams::ApiGroupDelete(ApiGroupDeleteParams {
                    api_group_id: "managed".to_owned(),
                }),
            ))
            .await,
    );
    match deleted {
        ResponseResult::ApiGroupDeleted(payload) => assert_eq!(payload.api_group_id, "managed"),
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn preset_crud_round_trips_and_preserves_values() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let created = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "preset-create",
                None::<String>,
                RequestParams::PresetCreate(PresetCreateParams {
                    preset_id: "managed".to_owned(),
                    display_name: "Managed Preset".to_owned(),
                    agents: protocol::PresetAgentPayloads {
                        planner: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.1),
                            max_tokens: Some(256),
                            extra: None,
                            prompt_entries: vec![protocol::PresetPromptEntryPayload {
                                entry_id: "planner-tone".to_owned(),
                                title: "Planner Tone".to_owned(),
                                content: "Favor concise story plans.".to_owned(),
                                enabled: true,
                            }],
                        },
                        architect: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.2),
                            max_tokens: Some(1024),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        director: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.3),
                            max_tokens: Some(384),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        actor: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.4),
                            max_tokens: Some(512),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        narrator: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.5),
                            max_tokens: Some(640),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        keeper: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.6),
                            max_tokens: Some(768),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        replyer: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.7),
                            max_tokens: Some(128),
                            extra: None,
                            prompt_entries: vec![protocol::PresetPromptEntryPayload {
                                entry_id: "replyer-style".to_owned(),
                                title: "Replyer Style".to_owned(),
                                content: "Prefer short candidate replies.".to_owned(),
                                enabled: true,
                            }],
                        },
                    },
                }),
            ))
            .await,
    );
    match created {
        ResponseResult::Preset(payload) => {
            assert_eq!(payload.preset_id, "managed");
            assert_eq!(payload.agents.actor.max_tokens, Some(512));
            assert_eq!(payload.agents.planner.prompt_entries.len(), 1);
            assert_eq!(
                payload.agents.planner.prompt_entries[0].entry_id,
                "planner-tone"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let fetched = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "preset-get",
                None::<String>,
                RequestParams::PresetGet(PresetGetParams {
                    preset_id: "managed".to_owned(),
                }),
            ))
            .await,
    );
    match fetched {
        ResponseResult::Preset(payload) => {
            assert_eq!(payload.display_name, "Managed Preset");
            assert_eq!(payload.agents.architect.max_tokens, Some(1024));
            assert_eq!(payload.agents.replyer.prompt_entries.len(), 1);
            assert_eq!(
                payload.agents.replyer.prompt_entries[0].content,
                "Prefer short candidate replies."
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let updated = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "preset-update",
                None::<String>,
                RequestParams::PresetUpdate(PresetUpdateParams {
                    preset_id: "managed".to_owned(),
                    display_name: Some("Updated Preset".to_owned()),
                    agents: Some(protocol::PresetAgentPayloads {
                        planner: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.2),
                            max_tokens: Some(300),
                            extra: None,
                            prompt_entries: vec![
                                protocol::PresetPromptEntryPayload {
                                    entry_id: "planner-voice".to_owned(),
                                    title: "Planner Voice".to_owned(),
                                    content: "Keep outline sections punchy.".to_owned(),
                                    enabled: true,
                                },
                                protocol::PresetPromptEntryPayload {
                                    entry_id: "planner-disabled".to_owned(),
                                    title: "Disabled".to_owned(),
                                    content: "Do not use.".to_owned(),
                                    enabled: false,
                                },
                            ],
                        },
                        architect: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.25),
                            max_tokens: Some(2048),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        director: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.3),
                            max_tokens: Some(400),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        actor: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.35),
                            max_tokens: Some(500),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        narrator: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.4),
                            max_tokens: Some(600),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        keeper: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.45),
                            max_tokens: Some(700),
                            extra: None,
                            prompt_entries: Vec::new(),
                        },
                        replyer: protocol::AgentPresetConfigPayload {
                            temperature: Some(0.5),
                            max_tokens: Some(200),
                            extra: Some(json!({"style":"short"})),
                            prompt_entries: vec![protocol::PresetPromptEntryPayload {
                                entry_id: "replyer-style".to_owned(),
                                title: "Replyer Style".to_owned(),
                                content: "Offer direct but varied replies.".to_owned(),
                                enabled: true,
                            }],
                        },
                    }),
                }),
            ))
            .await,
    );
    match updated {
        ResponseResult::Preset(payload) => {
            assert_eq!(payload.display_name, "Updated Preset");
            assert_eq!(payload.agents.architect.max_tokens, Some(2048));
            assert_eq!(payload.agents.replyer.extra, Some(json!({"style":"short"})));
            assert_eq!(payload.agents.planner.prompt_entries.len(), 2);
            assert_eq!(
                payload.agents.planner.prompt_entries[0].entry_id,
                "planner-voice"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }
    let listed = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "preset-list",
                None::<String>,
                RequestParams::PresetList(PresetListParams::default()),
            ))
            .await,
    );
    match listed {
        ResponseResult::PresetsListed(payload) => {
            let preset = payload
                .presets
                .iter()
                .find(|preset| preset.preset_id == "managed")
                .expect("managed preset should be listed");
            assert_eq!(preset.agents.planner.prompt_entry_count, 2);
            assert_eq!(preset.agents.planner.prompt_entries.len(), 2);
            assert_eq!(
                preset.agents.planner.prompt_entries[0].entry_id,
                "planner-voice"
            );
        }
        other => panic!("unexpected response: {other:?}"),
    }

    let deleted = unary_result(
        handler
            .handle(JsonRpcRequestMessage::new(
                "preset-delete",
                None::<String>,
                RequestParams::PresetDelete(PresetDeleteParams {
                    preset_id: "managed".to_owned(),
                }),
            ))
            .await,
    );
    match deleted {
        ResponseResult::PresetDeleted(payload) => assert_eq!(payload.preset_id, "managed"),
        other => panic!("unexpected response: {other:?}"),
    }
}

#[tokio::test]
async fn api_group_and_preset_delete_conflict_when_referenced_by_session() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let store = Arc::new(InMemoryStore::new());
    seed_story_records(&store).await;
    seed_alternate_api_groups_and_presets(&store).await;
    let handler = Handler::new(store.clone(), registry_with_ids(llm))
        .await
        .expect("handler should build");

    let session_start = handler
        .handle(JsonRpcRequestMessage::new(
            "session-start",
            None::<String>,
            RequestParams::StoryStartSession(StartSessionFromStoryParams {
                story_id: "story-1".to_owned(),
                display_name: Some("Config Test".to_owned()),
                player_profile_id: Some("profile-courier-a".to_owned()),
                api_group_id: Some("group-alt".to_owned()),
                preset_id: Some("preset-alt".to_owned()),
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

    let api_group_conflict = handler
        .handle(JsonRpcRequestMessage::new(
            "api-group-delete-session",
            None::<String>,
            RequestParams::ApiGroupDelete(ApiGroupDeleteParams {
                api_group_id: "group-alt".to_owned(),
            }),
        ))
        .await;
    match api_group_conflict {
        HandlerReply::Unary(response) => match response.outcome {
            JsonRpcOutcome::Err(error) => {
                assert_eq!(error.code, ErrorCode::Conflict.rpc_code())
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        HandlerReply::Stream { .. } => panic!("unexpected stream reply"),
    }

    let preset_conflict = handler
        .handle(JsonRpcRequestMessage::new(
            "preset-delete-session",
            None::<String>,
            RequestParams::PresetDelete(PresetDeleteParams {
                preset_id: "preset-alt".to_owned(),
            }),
        ))
        .await;
    match preset_conflict {
        HandlerReply::Unary(response) => match response.outcome {
            JsonRpcOutcome::Err(error) => {
                assert_eq!(error.code, ErrorCode::Conflict.rpc_code())
            }
            other => panic!("unexpected outcome: {other:?}"),
        },
        HandlerReply::Stream { .. } => panic!("unexpected stream reply"),
    }
}
