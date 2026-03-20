use serde_json::json;
use ss_protocol::{
    AgentPresetConfigPayload, ApiCreateParams, ApiDeleteParams, ApiGetParams, ApiListParams,
    ApiGroupBindingsInput, ApiGroupCreateParams, ApiGroupDeleteParams, ApiGroupGetParams,
    ApiGroupListParams, ApiUpdateParams, ArchitectPromptModePayload, CharacterCardContent,
    CharacterCreateParams, CharacterDeleteParams, CharacterGetParams, CharacterListParams,
    CharacterUpdateParams, CommonVariableDefinition, CommonVariableScope, ConfigGetGlobalParams,
    ContinueStoryDraftParams, CreateSessionMessageParams, CreateStoryParams,
    CreateStoryResourcesParams, DashboardGetParams, DataPackageExportPrepareParams,
    DataPackageImportCommitParams, DataPackageImportPrepareParams, DeleteSessionCharacterParams,
    DeleteSessionMessageParams, DeleteSessionParams, DeleteStoryDraftParams, DeleteStoryParams,
    DeleteStoryResourcesParams, EnterSessionCharacterSceneParams, FinalizeStoryDraftParams,
    GenerateStoryParams, GetRuntimeSnapshotParams, GetSessionCharacterParams,
    GetSessionMessageParams, GetSessionParams, GetSessionVariablesParams, GetStoryDraftParams,
    GetStoryParams, GetStoryResourcesParams, JsonRpcRequestMessage,
    LeaveSessionCharacterSceneParams, ListSessionCharactersParams, ListSessionMessagesParams,
    ListSessionsParams, ListStoriesParams, ListStoryDraftsParams, ListStoryResourcesParams,
    LorebookCreateParams, LorebookDeleteParams, LorebookGetParams, LorebookListParams,
    LorebookUpdateParams, PlayerProfileCreateParams, PlayerProfileDeleteParams,
    PlayerProfileGetParams, PlayerProfileListParams, PlayerProfileUpdateParams, PresetCreateParams,
    PresetDeleteParams, PresetGetParams, PresetListParams, PresetModuleEntryPayload,
    PresetPreviewRuntimeParams, PresetPreviewTemplateParams, PresetPromptModulePayload,
    PromptEntryKindPayload, PromptMessageRolePayload, PromptModuleIdPayload,
    PromptPreviewActorPurposePayload, RequestParams, RunTurnParams, SchemaCreateParams,
    SchemaDeleteParams, SchemaGetParams, SchemaListParams, SchemaUpdateParams,
    SessionGetConfigParams, SessionMessageKind, SessionUpdateConfigParams,
    SetPlayerProfileParams, StartSessionFromStoryParams, StartStoryDraftParams,
    SuggestRepliesParams, UpdateSessionCharacterParams, UpdateSessionMessageParams,
    UpdateSessionParams, UpdateSessionVariablesParams, UpdateStoryDraftGraphParams,
    UpdateStoryGraphParams, UpdateStoryParams, UpdateStoryResourcesParams,
};
use state::{StateFieldSchema, StateOp, StateUpdate, StateValueType};
use store::LlmProvider;
use story::StoryGraph;

fn sample_character_content() -> CharacterCardContent {
    CharacterCardContent {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative, casual".to_owned(),
        schema_id: "schema-character-merchant".to_owned(),
        system_prompt: "Stay in character.".to_owned(),
        tags: vec!["merchant".to_owned()],
        folder: "harbor".to_owned(),
    }
}

fn sample_api_group_bindings() -> ApiGroupBindingsInput {
    ApiGroupBindingsInput {
        planner_api_id: "api-planner".to_owned(),
        architect_api_id: "api-architect".to_owned(),
        director_api_id: "api-director".to_owned(),
        actor_api_id: "api-actor".to_owned(),
        narrator_api_id: "api-narrator".to_owned(),
        keeper_api_id: "api-keeper".to_owned(),
        replyer_api_id: "api-replyer".to_owned(),
    }
}

fn sample_preset_agents() -> ss_protocol::PresetAgentPayloads {
    let config = |max_tokens| AgentPresetConfigPayload {
        temperature: Some(0.1),
        max_tokens: Some(max_tokens),
        extra: None,
        modules: Vec::new(),
    };

    let mut planner = config(512);
    planner.modules.push(PresetPromptModulePayload {
        module_id: PromptModuleIdPayload::Task,
        display_name: "Task".to_owned(),
        message_role: PromptMessageRolePayload::System,
        order: 20,
        entries: vec![PresetModuleEntryPayload {
            entry_id: "planner-tone".to_owned(),
            display_name: "Planner Tone".to_owned(),
            kind: PromptEntryKindPayload::CustomText,
            enabled: true,
            order: 10,
            required: false,
            text: Some("Favor concise story plans.".to_owned()),
            context_key: None,
        }],
    });

    ss_protocol::PresetAgentPayloads {
        planner,
        architect: config(8192),
        director: config(512),
        actor: config(512),
        narrator: config(512),
        keeper: config(512),
        replyer: config(256),
    }
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
            scope: CommonVariableScope::Character,
            key: "trust".to_owned(),
            display_name: "Merchant Trust".to_owned(),
            character_id: Some("merchant".to_owned()),
            pinned: false,
        },
    ]
}

#[test]
fn story_requests_round_trip_with_stable_methods() {
    let generate_story = JsonRpcRequestMessage::new(
        "story-1",
        None::<String>,
        RequestParams::StoryGenerate(GenerateStoryParams {
            resource_id: "res-1".to_owned(),
            display_name: Some("Flooded Harbor".to_owned()),
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
            common_variables: Some(sample_common_variables()),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&generate_story)
            .expect("serialize")
            .contains("\"method\": \"story.generate\"")
    );

    let start_session = JsonRpcRequestMessage::new(
        "story-2",
        None::<String>,
        RequestParams::StoryStartSession(StartSessionFromStoryParams {
            story_id: "story-1".to_owned(),
            display_name: Some("Courier Run".to_owned()),
            player_profile_id: Some("profile-courier".to_owned()),
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&start_session)
            .expect("serialize")
            .contains("\"method\": \"story.start_session\"")
    );

    let draft_start = JsonRpcRequestMessage::new(
        "story-draft-start",
        None::<String>,
        RequestParams::StoryDraftStart(StartStoryDraftParams {
            resource_id: "res-1".to_owned(),
            display_name: Some("Draft Harbor".to_owned()),
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
            common_variables: Some(sample_common_variables()),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&draft_start)
            .expect("serialize")
            .contains("\"method\": \"story_draft.start\"")
    );

    let draft_continue = JsonRpcRequestMessage::new(
        "story-draft-continue",
        None::<String>,
        RequestParams::StoryDraftContinue(ContinueStoryDraftParams {
            draft_id: "draft-1".to_owned(),
        }),
    );
    let draft_continue_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&draft_continue).expect("serialize"))
            .expect("deserialize");
    assert!(matches!(
        draft_continue_round_trip.params,
        RequestParams::StoryDraftContinue(ContinueStoryDraftParams { draft_id }) if draft_id == "draft-1"
    ));

    let draft_update_graph = JsonRpcRequestMessage::new(
        "story-draft-update-graph",
        None::<String>,
        RequestParams::StoryDraftUpdateGraph(UpdateStoryDraftGraphParams {
            draft_id: "draft-1".to_owned(),
            partial_graph: StoryGraph::new("start", vec![]),
        }),
    );
    let draft_update_graph_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&draft_update_graph).expect("serialize"))
            .expect("deserialize");
    assert!(matches!(
        draft_update_graph_round_trip.params,
        RequestParams::StoryDraftUpdateGraph(UpdateStoryDraftGraphParams { draft_id, .. })
            if draft_id == "draft-1"
    ));

    let draft_finalize = JsonRpcRequestMessage::new(
        "story-draft-finalize",
        None::<String>,
        RequestParams::StoryDraftFinalize(FinalizeStoryDraftParams {
            draft_id: "draft-1".to_owned(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&draft_finalize)
            .expect("serialize")
            .contains("\"method\": \"story_draft.finalize\"")
    );
}

#[test]
fn data_package_requests_round_trip() {
    let export_prepare = JsonRpcRequestMessage::new(
        "data-package-export",
        None::<String>,
        RequestParams::DataPackageExportPrepare(DataPackageExportPrepareParams {
            preset_ids: vec!["preset-default".to_owned()],
            schema_ids: vec!["schema-player-default".to_owned()],
            lorebook_ids: vec!["lorebook-harbor".to_owned()],
            player_profile_ids: vec!["profile-courier".to_owned()],
            character_ids: vec!["merchant".to_owned()],
            story_resource_ids: vec!["resource-1".to_owned()],
            story_ids: vec!["story-1".to_owned()],
            include_dependencies: true,
        }),
    );
    assert!(
        serde_json::to_string_pretty(&export_prepare)
            .expect("serialize")
            .contains("\"method\": \"data_package.export_prepare\"")
    );

    let import_prepare = JsonRpcRequestMessage::new(
        "data-package-import-prepare",
        None::<String>,
        RequestParams::DataPackageImportPrepare(DataPackageImportPrepareParams::default()),
    );
    let import_prepare_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&import_prepare).expect("serialize"))
            .expect("deserialize");
    assert!(matches!(
        import_prepare_round_trip.params,
        RequestParams::DataPackageImportPrepare(_)
    ));

    let import_commit = JsonRpcRequestMessage::new(
        "data-package-import-commit",
        None::<String>,
        RequestParams::DataPackageImportCommit(DataPackageImportCommitParams {
            import_id: "package-import-1".to_owned(),
        }),
    );
    let import_commit_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&import_commit).expect("serialize"))
            .expect("deserialize");
    assert!(matches!(
        import_commit_round_trip.params,
        RequestParams::DataPackageImportCommit(DataPackageImportCommitParams { import_id })
            if import_id == "package-import-1"
    ));
}

#[test]
fn preset_preview_requests_round_trip() {
    let template_preview = JsonRpcRequestMessage::new(
        "preset-preview-template",
        None::<String>,
        RequestParams::PresetPreviewTemplate(PresetPreviewTemplateParams {
            preset_id: "preset-default".to_owned(),
            agent: ss_protocol::PresetAgentIdPayload::Planner,
            module_id: Some(PromptModuleIdPayload::StaticContext),
            architect_mode: None,
        }),
    );
    assert!(
        serde_json::to_string_pretty(&template_preview)
            .expect("serialize")
            .contains("\"method\": \"preset_preview.template\"")
    );

    let runtime_preview = JsonRpcRequestMessage::new(
        "preset-preview-runtime",
        Some("session-1".to_owned()),
        RequestParams::PresetPreviewRuntime(PresetPreviewRuntimeParams {
            preset_id: "preset-default".to_owned(),
            agent: ss_protocol::PresetAgentIdPayload::Actor,
            module_id: None,
            architect_mode: None,
            resource_id: None,
            draft_id: None,
            character_id: Some("merchant".to_owned()),
            actor_purpose: Some(PromptPreviewActorPurposePayload::ReactToPlayer),
            narrator_purpose: None,
            keeper_phase: None,
            previous_node_id: None,
            player_input: Some("Can you help me?".to_owned()),
            reply_limit: None,
        }),
    );
    let runtime_preview_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&runtime_preview).expect("serialize"))
            .expect("deserialize");
    assert!(matches!(
        runtime_preview_round_trip.params,
        RequestParams::PresetPreviewRuntime(PresetPreviewRuntimeParams {
            preset_id,
            character_id,
            actor_purpose: Some(PromptPreviewActorPurposePayload::ReactToPlayer),
            ..
        }) if preset_id == "preset-default" && character_id.as_deref() == Some("merchant")
    ));

    let architect_preview = JsonRpcRequestMessage::new(
        "preset-preview-architect",
        None::<String>,
        RequestParams::PresetPreviewRuntime(PresetPreviewRuntimeParams {
            preset_id: "preset-default".to_owned(),
            agent: ss_protocol::PresetAgentIdPayload::Architect,
            module_id: None,
            architect_mode: Some(ArchitectPromptModePayload::DraftContinue),
            resource_id: None,
            draft_id: Some("draft-1".to_owned()),
            character_id: None,
            actor_purpose: None,
            narrator_purpose: None,
            keeper_phase: None,
            previous_node_id: None,
            player_input: None,
            reply_limit: None,
        }),
    );
    assert!(
        serde_json::to_string_pretty(&architect_preview)
            .expect("serialize")
            .contains("\"method\": \"preset_preview.runtime\"")
    );
}

#[test]
fn api_group_and_preset_requests_round_trip() {
    let api_create = JsonRpcRequestMessage::new(
        "api-create",
        None::<String>,
        RequestParams::ApiCreate(ApiCreateParams {
            api_id: "api-planner".to_owned(),
            display_name: "Planner API".to_owned(),
            provider: LlmProvider::OpenAi,
            base_url: "https://api.openai.example/v1".to_owned(),
            api_key: "sk-secret".to_owned(),
            model: "planner-model".to_owned(),
        }),
    );
    let api_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&api_create).expect("serialize"))
            .expect("deserialize");
    assert!(matches!(
        api_round_trip.params,
        RequestParams::ApiCreate(ApiCreateParams { api_id, .. }) if api_id == "api-planner"
    ));

    let api_group_create = JsonRpcRequestMessage::new(
        "api-group-create",
        None::<String>,
        RequestParams::ApiGroupCreate(ApiGroupCreateParams {
            api_group_id: "group-default".to_owned(),
            display_name: "Default Group".to_owned(),
            bindings: sample_api_group_bindings(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&api_group_create)
            .expect("serialize")
            .contains("\"method\": \"api_group.create\"")
    );

    for request in [
        JsonRpcRequestMessage::new(
            "api-get",
            None::<String>,
            RequestParams::ApiGet(ApiGetParams {
                api_id: "api-planner".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "api-list",
            None::<String>,
            RequestParams::ApiList(ApiListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "api-update",
            None::<String>,
            RequestParams::ApiUpdate(ApiUpdateParams {
                api_id: "api-planner".to_owned(),
                display_name: Some("Planner API 2".to_owned()),
                provider: None,
                base_url: None,
                api_key: None,
                model: Some("planner-model-2".to_owned()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "api-delete",
            None::<String>,
            RequestParams::ApiDelete(ApiDeleteParams {
                api_id: "api-planner".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "api-group-get",
            None::<String>,
            RequestParams::ApiGroupGet(ApiGroupGetParams {
                api_group_id: "group-default".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "api-group-list",
            None::<String>,
            RequestParams::ApiGroupList(ApiGroupListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "api-group-delete",
            None::<String>,
            RequestParams::ApiGroupDelete(ApiGroupDeleteParams {
                api_group_id: "group-default".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "preset-create",
            None::<String>,
            RequestParams::PresetCreate(PresetCreateParams {
                preset_id: "preset-default".to_owned(),
                display_name: "Default Preset".to_owned(),
                agents: sample_preset_agents(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "preset-get",
            None::<String>,
            RequestParams::PresetGet(PresetGetParams {
                preset_id: "preset-default".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "preset-list",
            None::<String>,
            RequestParams::PresetList(PresetListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "preset-delete",
            None::<String>,
            RequestParams::PresetDelete(PresetDeleteParams {
                preset_id: "preset-default".to_owned(),
            }),
        ),
    ] {
        let json = serde_json::to_string_pretty(&request).expect("serialize");
        let round_trip: JsonRpcRequestMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(request.method(), round_trip.method());
    }
}

#[test]
fn character_schema_and_session_requests_round_trip() {
    let character_create = JsonRpcRequestMessage::new(
        "character-create",
        None::<String>,
        RequestParams::CharacterCreate(CharacterCreateParams {
            content: sample_character_content(),
        }),
    );
    assert!(
        serde_json::to_string_pretty(&character_create)
            .expect("serialize")
            .contains("\"method\": \"character.create\"")
    );

    let requests = vec![
        JsonRpcRequestMessage::new(
            "character-get",
            None::<String>,
            RequestParams::CharacterGet(CharacterGetParams {
                character_id: "merchant".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "character-update",
            None::<String>,
            RequestParams::CharacterUpdate(CharacterUpdateParams {
                character_id: "merchant".to_owned(),
                content: sample_character_content(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "character-list",
            None::<String>,
            RequestParams::CharacterList(CharacterListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "character-delete",
            None::<String>,
            RequestParams::CharacterDelete(CharacterDeleteParams {
                character_id: "merchant".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-run-turn",
            Some("session-1"),
            RequestParams::SessionRunTurn(RunTurnParams {
                player_input: "Open the gate.".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-suggest",
            Some("session-1"),
            RequestParams::SessionSuggestReplies(SuggestRepliesParams { limit: Some(3) }),
        ),
        JsonRpcRequestMessage::new(
            "session-set-profile",
            Some("session-1"),
            RequestParams::SessionSetPlayerProfile(SetPlayerProfileParams {
                player_profile_id: Some("profile-courier".to_owned()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-update-config",
            Some("session-1"),
            RequestParams::SessionUpdateConfig(SessionUpdateConfigParams {
                api_group_id: Some("group-default".to_owned()),
                preset_id: Some("preset-default".to_owned()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-message-create",
            Some("session-1"),
            RequestParams::SessionMessageCreate(CreateSessionMessageParams {
                kind: SessionMessageKind::Dialogue,
                speaker_id: "merchant".to_owned(),
                speaker_name: "Haru".to_owned(),
                text: "Take the lantern.".to_owned(),
            }),
        ),
    ];

    for request in requests {
        let json = serde_json::to_string_pretty(&request).expect("serialize");
        let round_trip: JsonRpcRequestMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(request.method(), round_trip.method());
    }
}

#[test]
fn resource_story_schema_profile_and_dashboard_requests_round_trip() {
    let requests = vec![
        JsonRpcRequestMessage::new(
            "resources-create",
            None::<String>,
            RequestParams::StoryResourcesCreate(CreateStoryResourcesParams {
                story_concept: "A flooded harbor story.".to_owned(),
                character_ids: vec!["merchant".to_owned(), "guard".to_owned()],
                player_schema_id_seed: Some("schema-player-default".to_owned()),
                world_schema_id_seed: Some("schema-world-default".to_owned()),
                lorebook_ids: vec![],
                planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "resources-get",
            None::<String>,
            RequestParams::StoryResourcesGet(GetStoryResourcesParams {
                resource_id: "resource-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "resources-list",
            None::<String>,
            RequestParams::StoryResourcesList(ListStoryResourcesParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "resources-update",
            None::<String>,
            RequestParams::StoryResourcesUpdate(UpdateStoryResourcesParams {
                resource_id: "resource-1".to_owned(),
                story_concept: Some("Updated story".to_owned()),
                character_ids: None,
                player_schema_id_seed: None,
                world_schema_id_seed: None,
                lorebook_ids: None,
                planned_story: None,
            }),
        ),
        JsonRpcRequestMessage::new(
            "resources-delete",
            None::<String>,
            RequestParams::StoryResourcesDelete(DeleteStoryResourcesParams {
                resource_id: "resource-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "lorebook-create",
            None::<String>,
            RequestParams::LorebookCreate(LorebookCreateParams {
                lorebook_id: "lorebook-1".to_owned(),
                display_name: "Harbor Lore".to_owned(),
                entries: vec![],
            }),
        ),
        JsonRpcRequestMessage::new(
            "lorebook-get",
            None::<String>,
            RequestParams::LorebookGet(LorebookGetParams {
                lorebook_id: "lorebook-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "lorebook-list",
            None::<String>,
            RequestParams::LorebookList(LorebookListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "lorebook-update",
            None::<String>,
            RequestParams::LorebookUpdate(LorebookUpdateParams {
                lorebook_id: "lorebook-1".to_owned(),
                display_name: Some("Updated Harbor Lore".to_owned()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "lorebook-delete",
            None::<String>,
            RequestParams::LorebookDelete(LorebookDeleteParams {
                lorebook_id: "lorebook-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-create",
            None::<String>,
            RequestParams::StoryCreate(CreateStoryParams {
                resource_id: "resource-1".to_owned(),
                display_name: Some("Manual Harbor".to_owned()),
                graph: StoryGraph::new("start", vec![]),
                world_schema_id: "schema-world-story-1".to_owned(),
                player_schema_id: "schema-player-story-1".to_owned(),
                introduction: "A courier arrives at the harbor.".to_owned(),
                common_variables: Some(sample_common_variables()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-get",
            None::<String>,
            RequestParams::StoryGet(GetStoryParams {
                story_id: "story-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-update",
            None::<String>,
            RequestParams::StoryUpdate(UpdateStoryParams {
                story_id: "story-1".to_owned(),
                display_name: Some("Renamed Story".to_owned()),
                common_variables: None,
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-update-graph",
            None::<String>,
            RequestParams::StoryUpdateGraph(UpdateStoryGraphParams {
                story_id: "story-1".to_owned(),
                graph: StoryGraph::new("start", vec![]),
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-list",
            None::<String>,
            RequestParams::StoryList(ListStoriesParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "story-delete",
            None::<String>,
            RequestParams::StoryDelete(DeleteStoryParams {
                story_id: "story-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
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
        ),
        JsonRpcRequestMessage::new(
            "schema-get",
            None::<String>,
            RequestParams::SchemaGet(SchemaGetParams {
                schema_id: "schema-player-default".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "schema-list",
            None::<String>,
            RequestParams::SchemaList(SchemaListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "schema-update",
            None::<String>,
            RequestParams::SchemaUpdate(SchemaUpdateParams {
                schema_id: "schema-player-default".to_owned(),
                display_name: Some("Updated Schema".to_owned()),
                tags: Some(vec!["player".to_owned(), "updated".to_owned()]),
                fields: Some(
                    [(
                        "coins".to_owned(),
                        StateFieldSchema::new(StateValueType::Int).with_default(json!(1)),
                    )]
                    .into_iter()
                    .collect(),
                ),
            }),
        ),
        JsonRpcRequestMessage::new(
            "schema-delete",
            None::<String>,
            RequestParams::SchemaDelete(SchemaDeleteParams {
                schema_id: "schema-player-default".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "profile-create",
            None::<String>,
            RequestParams::PlayerProfileCreate(PlayerProfileCreateParams {
                player_profile_id: "profile-courier".to_owned(),
                display_name: "Courier".to_owned(),
                description: "A determined courier.".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "profile-get",
            None::<String>,
            RequestParams::PlayerProfileGet(PlayerProfileGetParams {
                player_profile_id: "profile-courier".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "profile-list",
            None::<String>,
            RequestParams::PlayerProfileList(PlayerProfileListParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "profile-update",
            None::<String>,
            RequestParams::PlayerProfileUpdate(PlayerProfileUpdateParams {
                player_profile_id: "profile-courier".to_owned(),
                display_name: Some("Updated Courier".to_owned()),
                description: Some("An experienced courier.".to_owned()),
            }),
        ),
        JsonRpcRequestMessage::new(
            "profile-delete",
            None::<String>,
            RequestParams::PlayerProfileDelete(PlayerProfileDeleteParams {
                player_profile_id: "profile-courier".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-get",
            Some("session-1"),
            RequestParams::SessionGet(GetSessionParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "session-update",
            Some("session-1"),
            RequestParams::SessionUpdate(UpdateSessionParams {
                display_name: "Updated Session".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-list",
            None::<String>,
            RequestParams::SessionList(ListSessionsParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "session-delete",
            Some("session-1"),
            RequestParams::SessionDelete(DeleteSessionParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "session-message-get",
            Some("session-1"),
            RequestParams::SessionMessageGet(GetSessionMessageParams {
                message_id: "message-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-message-list",
            Some("session-1"),
            RequestParams::SessionMessageList(ListSessionMessagesParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "session-message-update",
            Some("session-1"),
            RequestParams::SessionMessageUpdate(UpdateSessionMessageParams {
                message_id: "message-1".to_owned(),
                kind: SessionMessageKind::Action,
                speaker_id: "merchant".to_owned(),
                speaker_name: "Haru".to_owned(),
                text: "Take the lantern.".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-message-delete",
            Some("session-1"),
            RequestParams::SessionMessageDelete(DeleteSessionMessageParams {
                message_id: "message-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-character-get",
            Some("session-1"),
            RequestParams::SessionCharacterGet(GetSessionCharacterParams {
                session_character_id: "dock_guard".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-character-list",
            Some("session-1"),
            RequestParams::SessionCharacterList(ListSessionCharactersParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "session-character-update",
            Some("session-1"),
            RequestParams::SessionCharacterUpdate(UpdateSessionCharacterParams {
                session_character_id: "dock_guard".to_owned(),
                display_name: "Dock Guard".to_owned(),
                personality: "stern".to_owned(),
                style: "brief".to_owned(),
                system_prompt: "Stay on duty.".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-character-enter",
            Some("session-1"),
            RequestParams::SessionCharacterEnterScene(EnterSessionCharacterSceneParams {
                session_character_id: "dock_guard".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-character-leave",
            Some("session-1"),
            RequestParams::SessionCharacterLeaveScene(LeaveSessionCharacterSceneParams {
                session_character_id: "dock_guard".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-character-delete",
            Some("session-1"),
            RequestParams::SessionCharacterDelete(DeleteSessionCharacterParams {
                session_character_id: "dock_guard".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "session-update-description",
            Some("session-1"),
            RequestParams::SessionUpdatePlayerDescription(
                ss_protocol::UpdatePlayerDescriptionParams {
                    player_description: "A determined courier.".to_owned(),
                },
            ),
        ),
        JsonRpcRequestMessage::new(
            "snapshot-get",
            Some("session-1"),
            RequestParams::SessionGetRuntimeSnapshot(GetRuntimeSnapshotParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "variables-get",
            Some("session-1"),
            RequestParams::SessionGetVariables(GetSessionVariablesParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "variables-update",
            Some("session-1"),
            RequestParams::SessionUpdateVariables(UpdateSessionVariablesParams {
                update: StateUpdate::new().push(StateOp::SetPlayerState {
                    key: "coins".to_owned(),
                    value: json!(7),
                }),
            }),
        ),
        JsonRpcRequestMessage::new(
            "config-get-global",
            None::<String>,
            RequestParams::ConfigGetGlobal(ConfigGetGlobalParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "session-config-get",
            Some("session-1"),
            RequestParams::SessionGetConfig(SessionGetConfigParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "dashboard-get",
            None::<String>,
            RequestParams::DashboardGet(DashboardGetParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "story-draft-get",
            None::<String>,
            RequestParams::StoryDraftGet(GetStoryDraftParams {
                draft_id: "draft-1".to_owned(),
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-draft-list",
            None::<String>,
            RequestParams::StoryDraftList(ListStoryDraftsParams::default()),
        ),
        JsonRpcRequestMessage::new(
            "story-draft-update-graph",
            None::<String>,
            RequestParams::StoryDraftUpdateGraph(UpdateStoryDraftGraphParams {
                draft_id: "draft-1".to_owned(),
                partial_graph: StoryGraph::new("start", vec![]),
            }),
        ),
        JsonRpcRequestMessage::new(
            "story-draft-delete",
            None::<String>,
            RequestParams::StoryDraftDelete(DeleteStoryDraftParams {
                draft_id: "draft-1".to_owned(),
            }),
        ),
    ];

    for request in requests {
        let json = serde_json::to_string_pretty(&request).expect("serialize");
        let round_trip: JsonRpcRequestMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(request.method(), round_trip.method());
    }
}

#[test]
fn story_update_and_schema_enum_requests_preserve_new_fields() {
    let common_variables = sample_common_variables();
    let story_generate = JsonRpcRequestMessage::new(
        "story-generate-common-variables",
        None::<String>,
        RequestParams::StoryGenerate(GenerateStoryParams {
            resource_id: "resource-1".to_owned(),
            display_name: Some("Flooded Harbor".to_owned()),
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
            common_variables: Some(common_variables.clone()),
        }),
    );
    let story_generate_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&story_generate).expect("serialize"))
            .expect("deserialize");
    match story_generate_round_trip.params {
        RequestParams::StoryGenerate(GenerateStoryParams {
            resource_id,
            common_variables: Some(round_tripped),
            ..
        }) => {
            assert_eq!(resource_id, "resource-1");
            assert_eq!(round_tripped, common_variables);
        }
        other => panic!("unexpected params: {other:?}"),
    }

    let story_create = JsonRpcRequestMessage::new(
        "story-create-common-variables",
        None::<String>,
        RequestParams::StoryCreate(CreateStoryParams {
            resource_id: "resource-1".to_owned(),
            display_name: Some("Manual Harbor".to_owned()),
            graph: StoryGraph::new("dock", vec![]),
            world_schema_id: "schema-world-story-1".to_owned(),
            player_schema_id: "schema-player-story-1".to_owned(),
            introduction: "At the dock.".to_owned(),
            common_variables: Some(common_variables.clone()),
        }),
    );
    let story_create_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&story_create).expect("serialize"))
            .expect("deserialize");
    match story_create_round_trip.params {
        RequestParams::StoryCreate(CreateStoryParams {
            resource_id,
            display_name,
            graph,
            world_schema_id,
            player_schema_id,
            introduction,
            common_variables: Some(round_tripped),
        }) => {
            assert_eq!(resource_id, "resource-1");
            assert_eq!(display_name.as_deref(), Some("Manual Harbor"));
            assert_eq!(graph.start_node, "dock");
            assert_eq!(world_schema_id, "schema-world-story-1");
            assert_eq!(player_schema_id, "schema-player-story-1");
            assert_eq!(introduction, "At the dock.");
            assert_eq!(round_tripped, common_variables);
        }
        other => panic!("unexpected params: {other:?}"),
    }

    let draft_start = JsonRpcRequestMessage::new(
        "story-draft-start-common-variables",
        None::<String>,
        RequestParams::StoryDraftStart(StartStoryDraftParams {
            resource_id: "resource-1".to_owned(),
            display_name: Some("Draft Harbor".to_owned()),
            api_group_id: Some("group-default".to_owned()),
            preset_id: Some("preset-default".to_owned()),
            common_variables: Some(common_variables.clone()),
        }),
    );
    let draft_start_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&draft_start).expect("serialize"))
            .expect("deserialize");
    match draft_start_round_trip.params {
        RequestParams::StoryDraftStart(StartStoryDraftParams {
            resource_id,
            common_variables: Some(round_tripped),
            ..
        }) => {
            assert_eq!(resource_id, "resource-1");
            assert_eq!(round_tripped, common_variables);
        }
        other => panic!("unexpected params: {other:?}"),
    }

    let story_update = JsonRpcRequestMessage::new(
        "story-update-common-variables",
        None::<String>,
        RequestParams::StoryUpdate(UpdateStoryParams {
            story_id: "story-1".to_owned(),
            display_name: None,
            common_variables: Some(common_variables.clone()),
        }),
    );
    let story_update_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&story_update).expect("serialize"))
            .expect("deserialize");
    match story_update_round_trip.params {
        RequestParams::StoryUpdate(UpdateStoryParams {
            story_id,
            display_name,
            common_variables: Some(round_tripped),
        }) => {
            assert_eq!(story_id, "story-1");
            assert_eq!(display_name, None);
            assert_eq!(round_tripped, common_variables);
        }
        other => panic!("unexpected params: {other:?}"),
    }

    let schema_create = JsonRpcRequestMessage::new(
        "schema-create-enum",
        None::<String>,
        RequestParams::SchemaCreate(SchemaCreateParams {
            schema_id: "schema-zone".to_owned(),
            display_name: "Zone Schema".to_owned(),
            tags: vec!["world".to_owned()],
            fields: [(
                "zone".to_owned(),
                StateFieldSchema::new(StateValueType::String)
                    .with_default(json!("dock"))
                    .with_enum_values(vec![json!("dock"), json!("tower")]),
            )]
            .into_iter()
            .collect(),
        }),
    );
    let schema_create_round_trip: JsonRpcRequestMessage =
        serde_json::from_str(&serde_json::to_string(&schema_create).expect("serialize"))
            .expect("deserialize");
    match schema_create_round_trip.params {
        RequestParams::SchemaCreate(SchemaCreateParams { fields, .. }) => {
            let field = fields.get("zone").expect("field should exist");
            assert_eq!(
                field.enum_values.as_ref(),
                Some(&vec![json!("dock"), json!("tower")])
            );
            assert_eq!(field.default.as_ref(), Some(&json!("dock")));
        }
        other => panic!("unexpected params: {other:?}"),
    }
}
