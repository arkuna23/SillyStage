use std::collections::HashMap;

use agents::actor::CharacterCard;
use serde_json::json;
use ss_store::{
    AgentApiIds, CharacterCardRecord, InMemoryStore, RuntimeSnapshot, SessionConfigMode,
    SessionEngineConfig, SessionRecord, Store, StoryRecord, StoryResourcesRecord,
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
    }
}

fn sample_character_record() -> CharacterCardRecord {
    CharacterCardRecord {
        character_id: "merchant".to_owned(),
        content: CharacterCard {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual".to_owned(),
            tendencies: vec!["likes profitable deals".to_owned()],
            state_schema: HashMap::from([(
                "trust".to_owned(),
                StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
            )]),
            system_prompt: "Stay in character.".to_owned(),
        },
        cover_file_name: Some("cover.png".to_owned()),
        cover_mime_type: Some("image/png".to_owned()),
        cover_bytes: Some(b"cover".to_vec()),
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
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
        planned_story: Some("Opening Situation:\nA courier arrives at dusk.".to_owned()),
    }
}

fn sample_story() -> StoryRecord {
    StoryRecord {
        story_id: "story-1".to_owned(),
        display_name: "Flooded Harbor".to_owned(),
        resource_id: "resource-1".to_owned(),
        graph: sample_story_graph(),
        world_state_schema: sample_world_state_schema(),
        player_state_schema: sample_player_state_schema(),
        introduction: "The courier reaches a flooded dock.".to_owned(),
    }
}

fn sample_session() -> SessionRecord {
    SessionRecord {
        session_id: "session-1".to_owned(),
        display_name: "Courier Run".to_owned(),
        story_id: "story-1".to_owned(),
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

    assert!(
        store
            .get_global_config()
            .await
            .expect("load global")
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
}

#[tokio::test]
async fn in_memory_store_lists_and_deletes_records() {
    let store = InMemoryStore::new();
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

    assert_eq!(store.list_characters().await.expect("list").len(), 1);
    assert_eq!(store.list_story_resources().await.expect("list").len(), 1);
    assert_eq!(store.list_stories().await.expect("list").len(), 1);
    assert_eq!(store.list_sessions().await.expect("list").len(), 1);

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

    assert!(store.list_sessions().await.expect("list").is_empty());
    assert!(store.list_stories().await.expect("list").is_empty());
    assert!(store.list_story_resources().await.expect("list").is_empty());
    assert!(store.list_characters().await.expect("list").is_empty());
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
