use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use ss_store::{
    AgentPresetConfig, ApiGroupAgentBindings, ApiGroupRecord, ApiRecord, CharacterCardDefinition,
    CharacterCardRecord, FileSystemStore, LlmProvider, PlayerProfileRecord, PresetAgentConfigs,
    PresetRecord, RuntimeSnapshot, SchemaRecord, SessionBindingConfig, SessionMessageKind,
    SessionMessageRecord, SessionRecord, Store, StoryRecord, StoryResourcesRecord,
};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldState, WorldStateSchema};
use story::{NarrativeNode, StoryGraph};

static NEXT_TEMP_DIR_ID: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let id = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("ss-store-fs-test-{}-{}", std::process::id(), id));
        std::fs::create_dir_all(&path).expect("temp test dir should be created");
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn sample_character_record() -> CharacterCardRecord {
    CharacterCardRecord {
        character_id: "merchant".to_owned(),
        content: CharacterCardDefinition {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual".to_owned(),
            tendencies: vec!["likes profitable deals".to_owned()],
            schema_id: "schema-character-merchant".to_owned(),
            system_prompt: "Stay in character.".to_owned(),
        },
        cover_file_name: Some("cover.png".to_owned()),
        cover_mime_type: Some("image/png".to_owned()),
        cover_bytes: Some(b"cover".to_vec()),
    }
}

fn sample_api_record(api_id: &str, model: &str) -> ApiRecord {
    ApiRecord {
        api_id: api_id.to_owned(),
        display_name: format!("API {api_id}"),
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: "sk-secret".to_owned(),
        model: model.to_owned(),
    }
}

fn sample_agent_preset_config(max_tokens: u32) -> AgentPresetConfig {
    AgentPresetConfig {
        temperature: Some(0.1),
        max_tokens: Some(max_tokens),
        extra: None,
    }
}

fn sample_api_group_record() -> ApiGroupRecord {
    ApiGroupRecord {
        api_group_id: "group-default".to_owned(),
        display_name: "Default Group".to_owned(),
        agents: ApiGroupAgentBindings {
            planner_api_id: "api-planner".to_owned(),
            architect_api_id: "api-architect".to_owned(),
            director_api_id: "api-director".to_owned(),
            actor_api_id: "api-actor".to_owned(),
            narrator_api_id: "api-narrator".to_owned(),
            keeper_api_id: "api-keeper".to_owned(),
            replyer_api_id: "api-replyer".to_owned(),
        },
    }
}

fn sample_preset_record() -> PresetRecord {
    PresetRecord {
        preset_id: "preset-default".to_owned(),
        display_name: "Default Preset".to_owned(),
        agents: PresetAgentConfigs {
            planner: sample_agent_preset_config(512),
            architect: sample_agent_preset_config(8192),
            director: sample_agent_preset_config(512),
            actor: sample_agent_preset_config(512),
            narrator: sample_agent_preset_config(512),
            keeper: sample_agent_preset_config(512),
            replyer: sample_agent_preset_config(256),
        },
    }
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
        "gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    schema
}

fn sample_story_graph() -> StoryGraph {
    StoryGraph::new(
        "dock",
        vec![NarrativeNode::new(
            "dock",
            "Flooded Dock",
            "A flooded dock at dusk.",
            "Decide whether to trust the merchant.",
            vec!["merchant".to_owned()],
            vec![],
            vec![],
        )],
    )
}

fn sample_story_resources() -> StoryResourcesRecord {
    StoryResourcesRecord {
        resource_id: "resource-1".to_owned(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec!["merchant".to_owned()],
        player_schema_id_seed: Some("schema-player-default".to_owned()),
        world_schema_id_seed: Some("schema-world-default".to_owned()),
        planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
    }
}

fn sample_story() -> StoryRecord {
    StoryRecord {
        story_id: "story-1".to_owned(),
        display_name: "Flooded Harbor".to_owned(),
        resource_id: "resource-1".to_owned(),
        graph: sample_story_graph(),
        world_schema_id: "schema-world-story-1".to_owned(),
        player_schema_id: "schema-player-story-1".to_owned(),
        introduction: "The courier reaches a flooded dock.".to_owned(),
        created_at_ms: Some(1_000),
        updated_at_ms: Some(2_000),
    }
}

fn sample_session() -> SessionRecord {
    SessionRecord {
        session_id: "session-1".to_owned(),
        display_name: "Courier Run".to_owned(),
        story_id: "story-1".to_owned(),
        player_profile_id: Some("profile-courier".to_owned()),
        player_schema_id: "schema-player-story-1".to_owned(),
        binding: SessionBindingConfig {
            api_group_id: "group-default".to_owned(),
            preset_id: "preset-default".to_owned(),
        },
        snapshot: RuntimeSnapshot {
            story_id: "story-1".to_owned(),
            player_description: "A determined courier.".to_owned(),
            world_state: WorldState::new("dock")
                .with_active_characters(vec!["merchant".to_owned()]),
            turn_index: 0,
        },
        created_at_ms: Some(3_000),
        updated_at_ms: Some(4_000),
    }
}

fn sample_session_message() -> SessionMessageRecord {
    SessionMessageRecord {
        message_id: "session-message-1".to_owned(),
        session_id: "session-1".to_owned(),
        kind: SessionMessageKind::Narration,
        sequence: 0,
        turn_index: 0,
        recorded_at_ms: 3_500,
        created_at_ms: 3_500,
        updated_at_ms: 3_500,
        speaker_id: "narrator".to_owned(),
        speaker_name: "Narrator".to_owned(),
        text: "A courier steps onto the dock.".to_owned(),
    }
}

fn sample_schema_record(schema_id: &str, display_name: &str) -> SchemaRecord {
    let fields = if schema_id.contains("world") {
        sample_world_state_schema().fields
    } else if schema_id.contains("player") {
        sample_player_state_schema().fields
    } else {
        HashMap::from([(
            "trust".to_owned(),
            StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
        )])
    };

    SchemaRecord {
        schema_id: schema_id.to_owned(),
        display_name: display_name.to_owned(),
        tags: vec!["test".to_owned()],
        fields,
    }
}

fn sample_player_profile() -> PlayerProfileRecord {
    PlayerProfileRecord {
        player_profile_id: "profile-courier".to_owned(),
        display_name: "Courier".to_owned(),
        description: "A determined courier.".to_owned(),
    }
}

#[tokio::test]
async fn filesystem_store_round_trips_all_records() {
    let temp_dir = TestDir::new();
    let store = FileSystemStore::new(temp_dir.path.clone())
        .await
        .expect("filesystem store should build");

    for (api_id, model) in [
        ("api-planner", "planner-model"),
        ("api-architect", "architect-model"),
        ("api-director", "director-model"),
        ("api-actor", "actor-model"),
        ("api-narrator", "narrator-model"),
        ("api-keeper", "keeper-model"),
        ("api-replyer", "replyer-model"),
    ] {
        store
            .save_api(sample_api_record(api_id, model))
            .await
            .expect("save api");
    }
    store
        .save_api_group(sample_api_group_record())
        .await
        .expect("save api group");
    store
        .save_preset(sample_preset_record())
        .await
        .expect("save preset");
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
    store
        .save_player_profile(sample_player_profile())
        .await
        .expect("save player profile");
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");
    store
        .save_story_resources(sample_story_resources())
        .await
        .expect("save resources");
    store.save_story(sample_story()).await.expect("save story");
    store
        .save_session(sample_session())
        .await
        .expect("save session");
    store
        .save_session_message(sample_session_message())
        .await
        .expect("save session message");

    assert!(store.root().join("apis/api-planner.json").exists());
    assert!(store.root().join("api_groups/group-default.json").exists());
    assert!(store.root().join("presets/preset-default.json").exists());
    assert!(
        store
            .root()
            .join("schemas/schema-character-merchant.json")
            .exists()
    );
    assert!(
        store
            .root()
            .join("player_profiles/profile-courier.json")
            .exists()
    );
    assert!(store.root().join("characters/merchant/cover.bin").exists());
    assert!(
        store
            .root()
            .join("characters/merchant/record.json")
            .exists()
    );

    assert!(
        store
            .get_api("api-planner")
            .await
            .expect("load api")
            .is_some()
    );
    assert!(
        store
            .get_api_group("group-default")
            .await
            .expect("load api group")
            .is_some()
    );
    assert!(
        store
            .get_preset("preset-default")
            .await
            .expect("load preset")
            .is_some()
    );
    assert!(
        store
            .get_schema("schema-character-merchant")
            .await
            .expect("load schema")
            .is_some()
    );
    assert!(
        store
            .get_player_profile("profile-courier")
            .await
            .expect("load player profile")
            .is_some()
    );
    assert!(
        store
            .get_character("merchant")
            .await
            .expect("load character")
            .is_some()
    );
    assert!(
        store
            .get_story_resources("resource-1")
            .await
            .expect("load resources")
            .is_some()
    );
    assert!(
        store
            .get_story("story-1")
            .await
            .expect("load story")
            .is_some()
    );
    assert!(
        store
            .get_session("session-1")
            .await
            .expect("load session")
            .is_some()
    );
    assert!(
        store
            .get_session_message("session-message-1")
            .await
            .expect("load session message")
            .is_some()
    );
}

#[tokio::test]
async fn filesystem_store_lists_and_deletes_records() {
    let temp_dir = TestDir::new();
    let store = FileSystemStore::new(temp_dir.path.clone())
        .await
        .expect("filesystem store should build");

    for (api_id, model) in [
        ("api-planner", "planner-model"),
        ("api-architect", "architect-model"),
        ("api-director", "director-model"),
        ("api-actor", "actor-model"),
        ("api-narrator", "narrator-model"),
        ("api-keeper", "keeper-model"),
        ("api-replyer", "replyer-model"),
    ] {
        store
            .save_api(sample_api_record(api_id, model))
            .await
            .expect("save api");
    }
    store
        .save_api_group(sample_api_group_record())
        .await
        .expect("save api group");
    store
        .save_preset(sample_preset_record())
        .await
        .expect("save preset");
    store
        .save_schema(sample_schema_record(
            "schema-character-merchant",
            "Merchant Schema",
        ))
        .await
        .expect("save character schema");
    store
        .save_player_profile(sample_player_profile())
        .await
        .expect("save player profile");
    store
        .save_character(sample_character_record())
        .await
        .expect("save character");
    store
        .save_story_resources(sample_story_resources())
        .await
        .expect("save resources");
    store.save_story(sample_story()).await.expect("save story");
    store
        .save_session(sample_session())
        .await
        .expect("save session");
    store
        .save_session_message(sample_session_message())
        .await
        .expect("save session message");

    assert_eq!(store.list_apis().await.expect("list").len(), 7);
    assert_eq!(store.list_api_groups().await.expect("list").len(), 1);
    assert_eq!(store.list_presets().await.expect("list").len(), 1);
    assert_eq!(store.list_schemas().await.expect("list").len(), 1);
    assert_eq!(store.list_player_profiles().await.expect("list").len(), 1);
    assert_eq!(store.list_characters().await.expect("list").len(), 1);
    assert_eq!(store.list_story_resources().await.expect("list").len(), 1);
    assert_eq!(store.list_stories().await.expect("list").len(), 1);
    assert_eq!(store.list_sessions().await.expect("list").len(), 1);
    assert_eq!(
        store
            .list_session_messages("session-1")
            .await
            .expect("list")
            .len(),
        1
    );

    assert!(
        store
            .delete_player_profile("profile-courier")
            .await
            .expect("delete profile")
            .is_some()
    );
    assert!(
        store
            .delete_schema("schema-character-merchant")
            .await
            .expect("delete schema")
            .is_some()
    );
    assert!(
        store
            .delete_session_message("session-message-1")
            .await
            .expect("delete message")
            .is_some()
    );
    assert!(
        store
            .delete_session("session-1")
            .await
            .expect("delete session")
            .is_some()
    );
    assert!(
        store
            .delete_story("story-1")
            .await
            .expect("delete story")
            .is_some()
    );
    assert!(
        store
            .delete_story_resources("resource-1")
            .await
            .expect("delete resources")
            .is_some()
    );
    assert!(
        store
            .delete_character("merchant")
            .await
            .expect("delete character")
            .is_some()
    );
    assert!(
        store
            .delete_api_group("group-default")
            .await
            .expect("delete api group")
            .is_some()
    );
    for api_id in [
        "api-planner",
        "api-architect",
        "api-director",
        "api-actor",
        "api-narrator",
        "api-keeper",
        "api-replyer",
    ] {
        assert!(
            store
                .delete_api(api_id)
                .await
                .expect("delete api")
                .is_some()
        );
    }
    assert!(
        store
            .delete_preset("preset-default")
            .await
            .expect("delete preset")
            .is_some()
    );

    assert!(store.list_sessions().await.expect("list").is_empty());
    assert!(store.list_stories().await.expect("list").is_empty());
    assert!(store.list_story_resources().await.expect("list").is_empty());
    assert!(store.list_characters().await.expect("list").is_empty());
    assert!(store.list_player_profiles().await.expect("list").is_empty());
    assert!(store.list_schemas().await.expect("list").is_empty());
    assert!(
        store
            .list_session_messages("session-1")
            .await
            .expect("list")
            .is_empty()
    );
    assert!(store.list_apis().await.expect("list").is_empty());
    assert!(store.list_api_groups().await.expect("list").is_empty());
    assert!(store.list_presets().await.expect("list").is_empty());
}

#[tokio::test]
async fn filesystem_store_rejects_invalid_path_ids() {
    let temp_dir = TestDir::new();
    let store = FileSystemStore::new(temp_dir.path.clone())
        .await
        .expect("filesystem store should build");

    let error = store
        .get_character("../merchant")
        .await
        .expect_err("invalid character id should fail");
    assert!(error.to_string().contains("invalid store id"));
}

#[tokio::test]
async fn filesystem_store_supports_characters_without_cover_file() {
    let temp_dir = TestDir::new();
    let store = FileSystemStore::new(temp_dir.path.clone())
        .await
        .expect("filesystem store should build");
    let mut character = sample_character_record();
    character.character_id = "coverless".to_owned();
    character.content.id = "coverless".to_owned();
    character.cover_file_name = None;
    character.cover_mime_type = None;
    character.cover_bytes = None;

    store
        .save_character(character)
        .await
        .expect("save character without cover");

    assert!(!store.root().join("characters/coverless/cover.bin").exists());

    let loaded = store
        .get_character("coverless")
        .await
        .expect("load character")
        .expect("character should exist");
    assert!(loaded.cover_file_name.is_none());
    assert!(loaded.cover_mime_type.is_none());
    assert!(loaded.cover_bytes.is_none());
}

#[tokio::test]
async fn filesystem_store_persists_story_and_session_timestamps() {
    let temp_dir = TestDir::new();
    let store = FileSystemStore::new(temp_dir.path.clone())
        .await
        .expect("filesystem store should build");

    let story = sample_story();
    let session = sample_session();
    let message = sample_session_message();
    store.save_story(story.clone()).await.expect("save story");
    store
        .save_session(session.clone())
        .await
        .expect("save session");
    store
        .save_session_message(message.clone())
        .await
        .expect("save session message");

    let loaded_story = store
        .get_story(&story.story_id)
        .await
        .expect("load story")
        .expect("story should exist");
    let loaded_session = store
        .get_session(&session.session_id)
        .await
        .expect("load session")
        .expect("session should exist");
    let loaded_message = store
        .get_session_message(&message.message_id)
        .await
        .expect("load session message")
        .expect("session message should exist");

    assert_eq!(loaded_story.created_at_ms, story.created_at_ms);
    assert_eq!(loaded_story.updated_at_ms, story.updated_at_ms);
    assert_eq!(loaded_session.created_at_ms, session.created_at_ms);
    assert_eq!(loaded_session.updated_at_ms, session.updated_at_ms);
    assert_eq!(loaded_message.updated_at_ms, message.updated_at_ms);
}
