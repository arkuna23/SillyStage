use std::collections::HashMap;

use serde_json::json;
use ss_store::{
    AgentApiIds, CharacterCardDefinition, CharacterCardRecord, DefaultLlmConfigRecord,
    InMemoryStore, LlmApiRecord, LlmProvider, PlayerProfileRecord, RuntimeSnapshot, SchemaRecord,
    SessionConfigMode, SessionEngineConfig, SessionMessageKind, SessionMessageRecord,
    SessionRecord, Store, StoryDraftRecord, StoryDraftStatus, StoryRecord, StoryResourcesRecord,
};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldState, WorldStateSchema};
use story::{NarrativeNode, StoryGraph};

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner".to_owned(),
        architect_api_id: "architect".to_owned(),
        director_api_id: "director".to_owned(),
        actor_api_id: "actor".to_owned(),
        narrator_api_id: "narrator".to_owned(),
        keeper_api_id: "keeper".to_owned(),
        replyer_api_id: "replyer".to_owned(),
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

fn sample_llm_api_record() -> LlmApiRecord {
    LlmApiRecord {
        api_id: "default".to_owned(),
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: "sk-secret".to_owned(),
        model: "gpt-4.1-mini".to_owned(),
        temperature: Some(0.2),
        max_tokens: Some(512),
    }
}

fn sample_default_llm_config_record() -> DefaultLlmConfigRecord {
    DefaultLlmConfigRecord {
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: "sk-default".to_owned(),
        model: "gpt-4.1-mini".to_owned(),
        temperature: Some(0.1),
        max_tokens: Some(1024),
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

fn sample_story_draft() -> StoryDraftRecord {
    StoryDraftRecord {
        draft_id: "draft-1".to_owned(),
        display_name: "Flooded Harbor Draft".to_owned(),
        resource_id: "resource-1".to_owned(),
        planned_story: "Opening Situation:\nA courier arrives at dusk.".to_owned(),
        outline_sections: vec![
            "A courier arrives at dusk.".to_owned(),
            "The merchant tests the courier.".to_owned(),
        ],
        next_section_index: 1,
        partial_graph: sample_story_graph(),
        world_schema_id: "schema-world-story-1".to_owned(),
        player_schema_id: "schema-player-story-1".to_owned(),
        introduction: "The courier reaches a flooded dock.".to_owned(),
        section_summaries: vec!["The courier reaches the flooded dock.".to_owned()],
        section_node_ids: vec![vec!["dock".to_owned()]],
        status: StoryDraftStatus::Building,
        final_story_id: None,
        created_at_ms: Some(1_500),
        updated_at_ms: Some(2_500),
    }
}

fn sample_session() -> SessionRecord {
    SessionRecord {
        session_id: "session-1".to_owned(),
        display_name: "Courier Run".to_owned(),
        story_id: "story-1".to_owned(),
        player_profile_id: Some("profile-courier".to_owned()),
        player_schema_id: "schema-player-story-1".to_owned(),
        snapshot: RuntimeSnapshot {
            story_id: "story-1".to_owned(),
            player_description: "A determined courier.".to_owned(),
            world_state: WorldState::new("dock")
                .with_active_characters(vec!["merchant".to_owned()]),
            turn_index: 0,
        },
        config: SessionEngineConfig {
            mode: SessionConfigMode::UseGlobal,
            session_api_ids: None,
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
async fn in_memory_store_persists_all_records() {
    let store = InMemoryStore::new();

    store
        .set_global_config(sample_api_ids())
        .await
        .expect("save global config");
    store
        .save_llm_api(sample_llm_api_record())
        .await
        .expect("save llm api");
    store
        .set_default_llm_config(sample_default_llm_config_record())
        .await
        .expect("save default llm config");
    store
        .set_default_llm_config(sample_default_llm_config_record())
        .await
        .expect("save default llm config");
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
    store
        .save_story_draft(sample_story_draft())
        .await
        .expect("save draft");
    store.save_story(sample_story()).await.expect("save story");
    store
        .save_session(sample_session())
        .await
        .expect("save session");
    store
        .save_session_message(sample_session_message())
        .await
        .expect("save session message");

    assert!(
        store
            .get_global_config()
            .await
            .expect("load global")
            .is_some()
    );
    assert!(
        store
            .get_llm_api("default")
            .await
            .expect("load llm api")
            .is_some()
    );
    assert!(
        store
            .get_default_llm_config()
            .await
            .expect("load default llm config")
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
            .get_story_draft("draft-1")
            .await
            .expect("load draft")
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
async fn in_memory_store_lists_and_deletes_records() {
    let store = InMemoryStore::new();
    store
        .save_llm_api(sample_llm_api_record())
        .await
        .expect("save llm api");
    store
        .set_default_llm_config(sample_default_llm_config_record())
        .await
        .expect("save default llm config");
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
    store
        .save_story_draft(sample_story_draft())
        .await
        .expect("save draft");
    store.save_story(sample_story()).await.expect("save story");
    store
        .save_session(sample_session())
        .await
        .expect("save session");
    store
        .save_session_message(sample_session_message())
        .await
        .expect("save session message");

    assert_eq!(store.list_llm_apis().await.expect("list").len(), 1);
    assert!(
        store
            .get_default_llm_config()
            .await
            .expect("get default llm config")
            .is_some()
    );
    assert_eq!(store.list_schemas().await.expect("list").len(), 1);
    assert_eq!(store.list_player_profiles().await.expect("list").len(), 1);
    assert_eq!(store.list_characters().await.expect("list").len(), 1);
    assert_eq!(store.list_story_resources().await.expect("list").len(), 1);
    assert_eq!(store.list_story_drafts().await.expect("list").len(), 1);
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
            .expect("delete player profile")
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
            .delete_llm_api("default")
            .await
            .expect("delete llm api")
            .is_some()
    );
    assert!(
        store
            .delete_session_message("session-message-1")
            .await
            .expect("delete session message")
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
            .delete_story_draft("draft-1")
            .await
            .expect("delete draft")
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

    assert!(store.list_sessions().await.expect("list").is_empty());
    assert!(
        store
            .list_session_messages("session-1")
            .await
            .expect("list")
            .is_empty()
    );
    assert!(store.list_stories().await.expect("list").is_empty());
    assert!(store.list_story_resources().await.expect("list").is_empty());
    assert!(store.list_story_drafts().await.expect("list").is_empty());
    assert!(store.list_characters().await.expect("list").is_empty());
    assert!(store.list_player_profiles().await.expect("list").is_empty());
    assert!(store.list_schemas().await.expect("list").is_empty());
}

#[tokio::test]
async fn in_memory_store_supports_characters_without_cover() {
    let store = InMemoryStore::new();
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

    let loaded = store
        .get_character("coverless")
        .await
        .expect("load character")
        .expect("character should exist");

    assert!(loaded.cover_file_name.is_none());
    assert!(loaded.cover_mime_type.is_none());
    assert!(loaded.cover_bytes.is_none());
}

#[test]
fn story_and_session_records_deserialize_without_timestamps() {
    let story: StoryRecord = serde_json::from_value(json!({
        "story_id": "story-legacy",
        "display_name": "Legacy Story",
        "resource_id": "resource-legacy",
        "graph": sample_story_graph(),
        "world_schema_id": "schema-world-legacy",
        "player_schema_id": "schema-player-legacy",
        "introduction": "Legacy intro"
    }))
    .expect("legacy story should deserialize");
    assert!(story.created_at_ms.is_none());
    assert!(story.updated_at_ms.is_none());

    let session: SessionRecord = serde_json::from_value(json!({
        "session_id": "session-legacy",
        "display_name": "Legacy Session",
        "story_id": "story-legacy",
        "player_profile_id": null,
        "player_schema_id": "schema-player-legacy",
        "snapshot": {
            "story_id": "story-legacy",
            "player_description": "A legacy player.",
            "world_state": WorldState::new("dock"),
            "turn_index": 0
        },
        "config": {
            "mode": "use_global",
            "session_api_ids": null
        }
    }))
    .expect("legacy session should deserialize");
    assert!(session.created_at_ms.is_none());
    assert!(session.updated_at_ms.is_none());
}

#[tokio::test]
async fn in_memory_store_returns_none_for_missing_default_llm_config() {
    let store = InMemoryStore::new();
    assert!(
        store
            .get_default_llm_config()
            .await
            .expect("default llm config lookup should succeed")
            .is_none()
    );
}
