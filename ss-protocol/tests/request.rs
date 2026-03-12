use engine::{AgentApiIdOverrides, AgentApiIds, SessionConfigMode};
use serde_json::json;
use ss_protocol::{
    CharacterCardContent, CharacterCoverMimeType, CharacterCreateParams, CharacterDeleteParams,
    CharacterExportChrParams, CharacterGetCoverParams, CharacterGetParams, CharacterListParams,
    CharacterSetCoverParams, ConfigGetGlobalParams, ConfigUpdateGlobalParams,
    CreateStoryResourcesParams, DeleteSessionParams, DeleteStoryParams,
    DeleteStoryResourcesParams, GenerateStoryParams, GetRuntimeSnapshotParams,
    GetSessionParams, GetStoryParams, GetStoryResourcesParams, JsonRpcRequestMessage,
    ListSessionsParams, ListStoriesParams, ListStoryResourcesParams, RequestParams,
    RunTurnParams, SessionGetConfigParams, SessionUpdateConfigParams,
    StartSessionFromStoryParams, UpdateStoryResourcesParams, UploadChunkParams,
    UploadCompleteParams, UploadInitParams, UploadTargetKind,
};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};

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

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner-default".to_owned(),
        architect_api_id: "architect-default".to_owned(),
        director_api_id: "director-default".to_owned(),
        actor_api_id: "actor-default".to_owned(),
        narrator_api_id: "narrator-default".to_owned(),
        keeper_api_id: "keeper-default".to_owned(),
    }
}

#[test]
fn create_story_resources_request_uses_character_ids_only() {
    let request = JsonRpcRequestMessage::new(
        "req-1",
        None::<String>,
        RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
            story_concept: "A flooded harbor story.".to_owned(),
            character_ids: vec!["merchant".to_owned(), "guard".to_owned()],
            player_state_schema_seed: sample_player_state_schema(),
            world_state_schema_seed: Some(sample_world_state_schema()),
            planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
        }),
    );

    let json = serde_json::to_string_pretty(&request).expect("request should serialize");
    assert!(json.contains("\"method\": \"story_resources.create\""));
    assert!(json.contains("\"character_ids\""));
    assert!(!json.contains("\"character_cards\""));
}

#[test]
fn upload_and_story_requests_round_trip_with_stable_methods() {
    let upload_init = JsonRpcRequestMessage::new(
        "upload-1",
        None::<String>,
        RequestParams::UploadInit(UploadInitParams {
            target_kind: UploadTargetKind::CharacterCard,
            file_name: "merchant.chr".to_owned(),
            content_type: "application/x-sillystage-character-card".to_owned(),
            total_size: 4096,
            sha256: "abcd1234".to_owned(),
        }),
    );
    let upload_init_json =
        serde_json::to_string_pretty(&upload_init).expect("upload init should serialize");
    assert!(upload_init_json.contains("\"method\": \"upload.init\""));

    let upload_chunk = JsonRpcRequestMessage::new(
        "upload-2",
        None::<String>,
        RequestParams::UploadChunk(UploadChunkParams {
            upload_id: "up-1".to_owned(),
            chunk_index: 0,
            offset: 0,
            payload_base64: "aGVsbG8=".to_owned(),
            is_last: false,
        }),
    );
    let upload_chunk_round_trip: JsonRpcRequestMessage = serde_json::from_str(
        &serde_json::to_string(&upload_chunk).expect("upload chunk should serialize"),
    )
    .expect("upload chunk should deserialize");
    assert!(matches!(
        upload_chunk_round_trip.params,
        RequestParams::UploadChunk(UploadChunkParams { upload_id, .. }) if upload_id == "up-1"
    ));

    let upload_complete = JsonRpcRequestMessage::new(
        "upload-3",
        None::<String>,
        RequestParams::UploadComplete(UploadCompleteParams {
            upload_id: "up-1".to_owned(),
        }),
    );
    let upload_complete_json =
        serde_json::to_string_pretty(&upload_complete).expect("upload complete should serialize");
    assert!(upload_complete_json.contains("\"method\": \"upload.complete\""));

    let generate_story = JsonRpcRequestMessage::new(
        "story-1",
        None::<String>,
        RequestParams::StoryGenerate(GenerateStoryParams {
            resource_id: "res-1".to_owned(),
            display_name: Some("Flooded Harbor".to_owned()),
            architect_api_id: Some("architect-fast".to_owned()),
        }),
    );
    let generate_story_json =
        serde_json::to_string_pretty(&generate_story).expect("generate story should serialize");
    assert!(generate_story_json.contains("\"method\": \"story.generate\""));

    let start_session = JsonRpcRequestMessage::new(
        "story-2",
        None::<String>,
        RequestParams::StoryStartSession(StartSessionFromStoryParams {
            story_id: "story-1".to_owned(),
            display_name: Some("Courier Run".to_owned()),
            player_description: "A disguised courier posing as a dock clerk.".to_owned(),
            config_mode: SessionConfigMode::UseSession,
            session_api_ids: Some(sample_api_ids()),
        }),
    );
    let start_session_json =
        serde_json::to_string_pretty(&start_session).expect("start session should serialize");
    assert!(start_session_json.contains("\"method\": \"story.start_session\""));
}

#[test]
fn object_crud_requests_keep_stable_method_names() {
    let character_get = JsonRpcRequestMessage::new(
        "character-get",
        None::<String>,
        RequestParams::CharacterGet(CharacterGetParams {
            character_id: "merchant".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_get)
            .expect("character get should serialize")
            .contains("\"method\": \"character.get\"")
    );

    let character_create = JsonRpcRequestMessage::new(
        "character-create",
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
    );
    assert!(
        serde_json::to_string_pretty(&character_create)
            .expect("character create should serialize")
            .contains("\"method\": \"character.create\"")
    );

    let character_list = JsonRpcRequestMessage::new(
        "character-list",
        None::<String>,
        RequestParams::CharacterList(CharacterListParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&character_list)
            .expect("character list should serialize")
            .contains("\"method\": \"character.list\"")
    );

    let character_get_cover = JsonRpcRequestMessage::new(
        "character-get-cover",
        None::<String>,
        RequestParams::CharacterGetCover(CharacterGetCoverParams {
            character_id: "merchant".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_get_cover)
            .expect("character get cover should serialize")
            .contains("\"method\": \"character.get_cover\"")
    );

    let character_export_chr = JsonRpcRequestMessage::new(
        "character-export-chr",
        None::<String>,
        RequestParams::CharacterExportChr(CharacterExportChrParams {
            character_id: "merchant".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_export_chr)
            .expect("character export chr should serialize")
            .contains("\"method\": \"character.export_chr\"")
    );

    let character_set_cover = JsonRpcRequestMessage::new(
        "character-set-cover",
        None::<String>,
        RequestParams::CharacterSetCover(CharacterSetCoverParams {
            character_id: "merchant".to_owned(),
            cover_mime_type: CharacterCoverMimeType::Png,
            cover_base64: "ZmFrZS1jb3Zlcg==".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_set_cover)
            .expect("character set cover should serialize")
            .contains("\"method\": \"character.set_cover\"")
    );

    let character_delete = JsonRpcRequestMessage::new(
        "character-delete",
        None::<String>,
        RequestParams::CharacterDelete(CharacterDeleteParams {
            character_id: "merchant".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_delete)
            .expect("character delete should serialize")
            .contains("\"method\": \"character.delete\"")
    );

    let resources_get = JsonRpcRequestMessage::new(
        "resources-get",
        None::<String>,
        RequestParams::StoryResourcesGet(GetStoryResourcesParams {
            resource_id: "resource-1".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&resources_get)
            .expect("resources get should serialize")
            .contains("\"method\": \"story_resources.get\"")
    );

    let resources_list = JsonRpcRequestMessage::new(
        "resources-list",
        None::<String>,
        RequestParams::StoryResourcesList(ListStoryResourcesParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&resources_list)
            .expect("resources list should serialize")
            .contains("\"method\": \"story_resources.list\"")
    );

    let resources_delete = JsonRpcRequestMessage::new(
        "resources-delete",
        None::<String>,
        RequestParams::StoryResourcesDelete(DeleteStoryResourcesParams {
            resource_id: "resource-1".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&resources_delete)
            .expect("resources delete should serialize")
            .contains("\"method\": \"story_resources.delete\"")
    );

    let story_get = JsonRpcRequestMessage::new(
        "story-get",
        None::<String>,
        RequestParams::StoryGet(GetStoryParams {
            story_id: "story-1".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&story_get)
            .expect("story get should serialize")
            .contains("\"method\": \"story.get\"")
    );

    let story_list = JsonRpcRequestMessage::new(
        "story-list",
        None::<String>,
        RequestParams::StoryList(ListStoriesParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&story_list)
            .expect("story list should serialize")
            .contains("\"method\": \"story.list\"")
    );

    let story_delete = JsonRpcRequestMessage::new(
        "story-delete",
        None::<String>,
        RequestParams::StoryDelete(DeleteStoryParams {
            story_id: "story-1".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&story_delete)
            .expect("story delete should serialize")
            .contains("\"method\": \"story.delete\"")
    );

    let session_get = JsonRpcRequestMessage::new(
        "session-get",
        Some("session-1"),
        RequestParams::SessionGet(GetSessionParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&session_get)
            .expect("session get should serialize")
            .contains("\"method\": \"session.get\"")
    );

    let session_list = JsonRpcRequestMessage::new(
        "session-list",
        None::<String>,
        RequestParams::SessionList(ListSessionsParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&session_list)
            .expect("session list should serialize")
            .contains("\"method\": \"session.list\"")
    );

    let session_delete = JsonRpcRequestMessage::new(
        "session-delete",
        Some("session-1"),
        RequestParams::SessionDelete(DeleteSessionParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&session_delete)
            .expect("session delete should serialize")
            .contains("\"method\": \"session.delete\"")
    );
}

#[test]
fn session_requests_keep_stable_method_names() {
    let run_turn = JsonRpcRequestMessage::new(
        "req-turn",
        Some("session-1"),
        RequestParams::SessionRunTurn(RunTurnParams {
            player_input: "Open the gate.".to_owned(),
            api_overrides: Some(AgentApiIdOverrides {
                actor_api_id: Some("actor-creative".to_owned()),
                ..AgentApiIdOverrides::default()
            }),
        }),
    );
    let run_turn_json =
        serde_json::to_string_pretty(&run_turn).expect("run_turn request should serialize");
    assert!(run_turn_json.contains("\"method\": \"session.run_turn\""));

    let update_resources = JsonRpcRequestMessage::new(
        "req-update",
        None::<String>,
        RequestParams::StoryResourcesUpdate(UpdateStoryResourcesParams {
            resource_id: "res-1".to_owned(),
            story_concept: None,
            character_ids: Some(vec!["merchant".to_owned()]),
            player_state_schema_seed: None,
            world_state_schema_seed: None,
            planned_story: Some("Core Conflict:\nThe flood gate is jammed.".to_owned()),
        }),
    );
    let update_resources_json = serde_json::to_string_pretty(&update_resources)
        .expect("update resources request should serialize");
    assert!(update_resources_json.contains("\"method\": \"story_resources.update\""));

    let get_snapshot = JsonRpcRequestMessage::new(
        "req-snapshot",
        Some("session-1"),
        RequestParams::SessionGetRuntimeSnapshot(GetRuntimeSnapshotParams::default()),
    );
    let get_snapshot_json =
        serde_json::to_string_pretty(&get_snapshot).expect("snapshot request should serialize");
    assert!(get_snapshot_json.contains("\"method\": \"session.get_runtime_snapshot\""));
}

#[test]
fn config_requests_round_trip() {
    let get_global = JsonRpcRequestMessage::new(
        "cfg-1",
        None::<String>,
        RequestParams::ConfigGetGlobal(ConfigGetGlobalParams::default()),
    );
    let get_global_json =
        serde_json::to_string_pretty(&get_global).expect("global config request should serialize");
    assert!(get_global_json.contains("\"method\": \"config.get_global\""));

    let update_global = JsonRpcRequestMessage::new(
        "cfg-2",
        None::<String>,
        RequestParams::ConfigUpdateGlobal(ConfigUpdateGlobalParams {
            api_overrides: AgentApiIdOverrides {
                director_api_id: Some("director-alt".to_owned()),
                ..AgentApiIdOverrides::default()
            },
        }),
    );
    let update_global_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&update_global).expect("serialize update"))
            .expect("deserialize update");
    assert!(matches!(
        update_global_round_trip.params,
        RequestParams::ConfigUpdateGlobal(ConfigUpdateGlobalParams { api_overrides })
            if api_overrides.director_api_id.as_deref() == Some("director-alt")
    ));

    let get_session = JsonRpcRequestMessage::new(
        "cfg-3",
        Some("session-1"),
        RequestParams::SessionGetConfig(SessionGetConfigParams::default()),
    );
    let get_session_json = serde_json::to_string_pretty(&get_session)
        .expect("session config request should serialize");
    assert!(get_session_json.contains("\"method\": \"session.get_config\""));

    let update_session = JsonRpcRequestMessage::new(
        "cfg-4",
        Some("session-1"),
        RequestParams::SessionUpdateConfig(SessionUpdateConfigParams {
            mode: SessionConfigMode::UseSession,
            session_api_ids: Some(sample_api_ids()),
            api_overrides: Some(AgentApiIdOverrides {
                keeper_api_id: Some("keeper-alt".to_owned()),
                ..AgentApiIdOverrides::default()
            }),
        }),
    );
    let update_session_json =
        serde_json::to_string_pretty(&update_session).expect("session update should serialize");
    assert!(update_session_json.contains("\"method\": \"session.update_config\""));
}
