use std::collections::HashMap;

use serde_json::json;
use ss_protocol::{DataPackageArchive, DataPackageCharacterEntry};
use state::{StateFieldSchema, StateValueType};
use store::{
    AgentPresetConfig, LorebookEntryRecord, LorebookRecord, PlayerProfileRecord,
    PresetAgentConfigs, PresetRecord, SchemaRecord, StoryRecord, StoryResourcesRecord,
};
use story::{NarrativeNode, StoryGraph};

fn sample_agent_config(max_tokens: u32) -> AgentPresetConfig {
    AgentPresetConfig {
        temperature: Some(0.2),
        max_tokens: Some(max_tokens),
        extra: None,
        modules: Vec::new(),
    }
}

fn sample_preset() -> PresetRecord {
    PresetRecord {
        preset_id: "preset-default".to_owned(),
        display_name: "Default".to_owned(),
        agents: PresetAgentConfigs {
            planner: sample_agent_config(512),
            architect: sample_agent_config(2048),
            director: sample_agent_config(768),
            actor: sample_agent_config(512),
            narrator: sample_agent_config(512),
            keeper: sample_agent_config(512),
            replyer: sample_agent_config(256),
        },
    }
}

fn sample_schema(schema_id: &str) -> SchemaRecord {
    SchemaRecord {
        schema_id: schema_id.to_owned(),
        display_name: schema_id.to_owned(),
        tags: vec!["test".to_owned()],
        fields: HashMap::from([(
            "flag".to_owned(),
            StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
        )]),
    }
}

fn sample_character() -> DataPackageCharacterEntry {
    DataPackageCharacterEntry {
        character_id: "merchant".to_owned(),
        content: ss_protocol::CharacterCardContent {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly".to_owned(),
            style: "casual".to_owned(),
            schema_id: "schema-character-merchant".to_owned(),
            system_prompt: "Stay in character.".to_owned(),
        },
        cover_file_name: Some("cover.png".to_owned()),
        cover_content_type: Some("image/png".to_owned()),
        cover_bytes: Some(b"cover-bytes".to_vec()),
    }
}

fn sample_story_graph() -> StoryGraph {
    StoryGraph::new(
        "dock",
        vec![NarrativeNode::new(
            "dock",
            "Flooded Dock",
            "A flooded dock at dusk.",
            "Choose what to do next.",
            vec!["merchant".to_owned()],
            vec![],
            vec![],
        )],
    )
}

#[test]
fn data_package_archive_round_trip_preserves_manifest_and_payloads() {
    let archive = DataPackageArchive::new(
        1_234,
        vec![sample_preset()],
        vec![
            sample_schema("schema-character-merchant"),
            sample_schema("schema-player-default"),
            sample_schema("schema-world-default"),
        ],
        vec![LorebookRecord {
            lorebook_id: "lorebook-harbor".to_owned(),
            display_name: "Harbor".to_owned(),
            entries: vec![LorebookEntryRecord {
                entry_id: "entry-tide".to_owned(),
                title: "Tide".to_owned(),
                content: "The tide is rising.".to_owned(),
                keywords: vec!["tide".to_owned()],
                enabled: true,
                always_include: false,
            }],
        }],
        vec![PlayerProfileRecord {
            player_profile_id: "profile-courier".to_owned(),
            display_name: "Courier".to_owned(),
            description: "A cautious courier.".to_owned(),
        }],
        vec![sample_character()],
        vec![StoryResourcesRecord {
            resource_id: "resource-1".to_owned(),
            story_concept: "A flooded harbor story.".to_owned(),
            character_ids: vec!["merchant".to_owned()],
            player_schema_id_seed: Some("schema-player-default".to_owned()),
            world_schema_id_seed: Some("schema-world-default".to_owned()),
            lorebook_ids: vec!["lorebook-harbor".to_owned()],
            planned_story: Some("Opening Situation: A courier arrives.".to_owned()),
        }],
        vec![StoryRecord {
            story_id: "story-1".to_owned(),
            display_name: "Flooded Harbor".to_owned(),
            resource_id: "resource-1".to_owned(),
            graph: sample_story_graph(),
            world_schema_id: "schema-world-default".to_owned(),
            player_schema_id: "schema-player-default".to_owned(),
            introduction: "The courier reaches the dock.".to_owned(),
            common_variables: vec![],
            created_at_ms: Some(10),
            updated_at_ms: Some(20),
        }],
    );

    let bytes = archive.to_zip_bytes().expect("archive should encode");
    let decoded = DataPackageArchive::from_zip_bytes(&bytes).expect("archive should decode");

    assert_eq!(
        decoded.manifest.format,
        ss_protocol::DATA_PACKAGE_ARCHIVE_FORMAT
    );
    assert_eq!(
        decoded.manifest.version,
        ss_protocol::DATA_PACKAGE_ARCHIVE_VERSION
    );
    assert_eq!(
        decoded.contents().characters.ids,
        vec!["merchant".to_owned()]
    );
    assert_eq!(decoded.contents().stories.ids, vec!["story-1".to_owned()]);
    assert_eq!(decoded.characters.len(), 1);
    assert_eq!(decoded.characters[0].content.name, "Haru");
    assert_eq!(
        decoded.characters[0].cover_bytes.as_deref(),
        Some(&b"cover-bytes"[..])
    );
    assert_eq!(decoded.story_resources.len(), 1);
    assert_eq!(
        decoded.story_resources[0].lorebook_ids,
        vec!["lorebook-harbor"]
    );
    assert_eq!(decoded.stories.len(), 1);
    assert_eq!(decoded.stories[0].resource_id, "resource-1");
}
