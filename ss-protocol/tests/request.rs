use engine::{AgentApiIdOverrides, AgentApiIds, SessionConfigMode};
use serde_json::json;
use ss_protocol::{
    CharacterCardContent, CharacterCoverMimeType, CharacterCreateParams, CharacterDeleteParams,
    CharacterExportChrParams, CharacterGetCoverParams, CharacterGetParams, CharacterListParams,
    CharacterSetCoverParams, CharacterUpdateParams, ConfigGetGlobalParams,
    ConfigUpdateGlobalParams, CreateStoryResourcesParams, DashboardGetParams, DeleteSessionParams,
    DeleteStoryParams, DeleteStoryResourcesParams, GenerateStoryParams, GetRuntimeSnapshotParams,
    GetSessionParams, GetStoryParams, GetStoryResourcesParams, JsonRpcRequestMessage,
    ListSessionsParams, ListStoriesParams, ListStoryResourcesParams, LlmApiCreateParams,
    LlmApiDeleteParams, LlmApiGetParams, LlmApiListParams, LlmApiUpdateParams,
    PlayerProfileCreateParams, PlayerProfileDeleteParams, PlayerProfileGetParams,
    PlayerProfileListParams, PlayerProfileUpdateParams, RequestParams, RunTurnParams,
    SchemaCreateParams, SchemaDeleteParams, SchemaGetParams, SchemaListParams, SchemaUpdateParams,
    SessionGetConfigParams, SessionUpdateConfigParams, SetPlayerProfileParams,
    StartSessionFromStoryParams, UpdateStoryResourcesParams, UploadChunkParams,
    UploadCompleteParams, UploadInitParams, UploadTargetKind,
};
use state::{StateFieldSchema, StateValueType};
use store::LlmProvider;

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
            player_schema_id_seed: Some("schema-player-default".to_owned()),
            world_schema_id_seed: Some("schema-world-default".to_owned()),
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
            player_profile_id: Some("profile-courier".to_owned()),
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
    let dashboard_get = JsonRpcRequestMessage::new(
        "dashboard-get",
        None::<String>,
        RequestParams::DashboardGet(DashboardGetParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&dashboard_get)
            .expect("dashboard get should serialize")
            .contains("\"method\": \"dashboard.get\"")
    );

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

    let character_update = JsonRpcRequestMessage::new(
        "character-update",
        None::<String>,
        RequestParams::CharacterUpdate(CharacterUpdateParams {
            character_id: "merchant".to_owned(),
            content: CharacterCardContent {
                id: "merchant".to_owned(),
                name: "Haru".to_owned(),
                personality: "greedy but friendly trader".to_owned(),
                style: "talkative, casual".to_owned(),
                tendencies: vec!["likes profitable deals".to_owned()],
                schema_id: "schema-character-merchant".to_owned(),
                system_prompt: "Stay in character.".to_owned(),
            },
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_update)
            .expect("character update should serialize")
            .contains("\"method\": \"character.update\"")
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
                schema_id: "schema-character-merchant".to_owned(),
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

    let llm_api_create = JsonRpcRequestMessage::new(
        "llm-api-create",
        None::<String>,
        RequestParams::LlmApiCreate(LlmApiCreateParams {
            api_id: "default".to_owned(),
            provider: LlmProvider::OpenAi,
            base_url: "https://api.openai.example/v1".to_owned(),
            api_key: "sk-secret".to_owned(),
            model: "gpt-4.1-mini".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&llm_api_create)
            .expect("llm api create should serialize")
            .contains("\"method\": \"llm_api.create\"")
    );

    let llm_api_get = JsonRpcRequestMessage::new(
        "llm-api-get",
        None::<String>,
        RequestParams::LlmApiGet(LlmApiGetParams {
            api_id: "default".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&llm_api_get)
            .expect("llm api get should serialize")
            .contains("\"method\": \"llm_api.get\"")
    );

    let llm_api_list = JsonRpcRequestMessage::new(
        "llm-api-list",
        None::<String>,
        RequestParams::LlmApiList(LlmApiListParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&llm_api_list)
            .expect("llm api list should serialize")
            .contains("\"method\": \"llm_api.list\"")
    );

    let llm_api_update = JsonRpcRequestMessage::new(
        "llm-api-update",
        None::<String>,
        RequestParams::LlmApiUpdate(LlmApiUpdateParams {
            api_id: "default".to_owned(),
            provider: None,
            base_url: Some("https://api.alt.example/v1".to_owned()),
            api_key: None,
            model: Some("gpt-4.1".to_owned()),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&llm_api_update)
            .expect("llm api update should serialize")
            .contains("\"method\": \"llm_api.update\"")
    );

    let llm_api_delete = JsonRpcRequestMessage::new(
        "llm-api-delete",
        None::<String>,
        RequestParams::LlmApiDelete(LlmApiDeleteParams {
            api_id: "default".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&llm_api_delete)
            .expect("llm api delete should serialize")
            .contains("\"method\": \"llm_api.delete\"")
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

    let schema_create = JsonRpcRequestMessage::new(
        "schema-create",
        None::<String>,
        RequestParams::SchemaCreate(SchemaCreateParams {
            schema_id: "schema-player-default".to_owned(),
            display_name: "Player Schema".to_owned(),
            tags: vec!["player".to_owned()],
            fields: [(
                "coins".to_owned(),
                StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
            )]
            .into_iter()
            .collect(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&schema_create)
            .expect("schema create should serialize")
            .contains("\"method\": \"schema.create\"")
    );

    let schema_get = JsonRpcRequestMessage::new(
        "schema-get",
        None::<String>,
        RequestParams::SchemaGet(SchemaGetParams {
            schema_id: "schema-player-default".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&schema_get)
            .expect("schema get should serialize")
            .contains("\"method\": \"schema.get\"")
    );

    let schema_list = JsonRpcRequestMessage::new(
        "schema-list",
        None::<String>,
        RequestParams::SchemaList(SchemaListParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&schema_list)
            .expect("schema list should serialize")
            .contains("\"method\": \"schema.list\"")
    );

    let schema_update = JsonRpcRequestMessage::new(
        "schema-update",
        None::<String>,
        RequestParams::SchemaUpdate(SchemaUpdateParams {
            schema_id: "schema-player-default".to_owned(),
            display_name: Some("Player Schema V2".to_owned()),
            tags: None,
            fields: None,
        }),
    );
    assert!(
        serde_json::to_string_pretty(&schema_update)
            .expect("schema update should serialize")
            .contains("\"method\": \"schema.update\"")
    );

    let schema_delete = JsonRpcRequestMessage::new(
        "schema-delete",
        None::<String>,
        RequestParams::SchemaDelete(SchemaDeleteParams {
            schema_id: "schema-player-default".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&schema_delete)
            .expect("schema delete should serialize")
            .contains("\"method\": \"schema.delete\"")
    );

    let player_profile_create = JsonRpcRequestMessage::new(
        "player-profile-create",
        None::<String>,
        RequestParams::PlayerProfileCreate(PlayerProfileCreateParams {
            player_profile_id: "profile-courier".to_owned(),
            display_name: "Courier".to_owned(),
            description: "A determined courier.".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&player_profile_create)
            .expect("player profile create should serialize")
            .contains("\"method\": \"player_profile.create\"")
    );

    let player_profile_get = JsonRpcRequestMessage::new(
        "player-profile-get",
        None::<String>,
        RequestParams::PlayerProfileGet(PlayerProfileGetParams {
            player_profile_id: "profile-courier".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&player_profile_get)
            .expect("player profile get should serialize")
            .contains("\"method\": \"player_profile.get\"")
    );

    let player_profile_list = JsonRpcRequestMessage::new(
        "player-profile-list",
        None::<String>,
        RequestParams::PlayerProfileList(PlayerProfileListParams::default()),
    );
    assert!(
        serde_json::to_string_pretty(&player_profile_list)
            .expect("player profile list should serialize")
            .contains("\"method\": \"player_profile.list\"")
    );

    let player_profile_update = JsonRpcRequestMessage::new(
        "player-profile-update",
        None::<String>,
        RequestParams::PlayerProfileUpdate(PlayerProfileUpdateParams {
            player_profile_id: "profile-courier".to_owned(),
            display_name: Some("Courier V2".to_owned()),
            description: None,
        }),
    );
    assert!(
        serde_json::to_string_pretty(&player_profile_update)
            .expect("player profile update should serialize")
            .contains("\"method\": \"player_profile.update\"")
    );

    let player_profile_delete = JsonRpcRequestMessage::new(
        "player-profile-delete",
        None::<String>,
        RequestParams::PlayerProfileDelete(PlayerProfileDeleteParams {
            player_profile_id: "profile-courier".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&player_profile_delete)
            .expect("player profile delete should serialize")
            .contains("\"method\": \"player_profile.delete\"")
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
            player_schema_id_seed: None,
            world_schema_id_seed: None,
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

    let set_player_profile = JsonRpcRequestMessage::new(
        "req-set-profile",
        Some("session-1"),
        RequestParams::SessionSetPlayerProfile(SetPlayerProfileParams {
            player_profile_id: Some("profile-courier".to_owned()),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&set_player_profile)
            .expect("set player profile should serialize")
            .contains("\"method\": \"session.set_player_profile\"")
    );
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
