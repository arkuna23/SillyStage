#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures_util::stream;
use llm::{ChatChunk, ChatRequest, ChatResponse, ChatStream, LlmApi, LlmError, Message, Role};
use protocol::{
    CharacterArchive, CharacterCardContent, CharacterCoverMimeType, StoryGeneratedPayload,
    StoryResourcesPayload,
};
use serde_json::json;
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use store::{
    AgentPresetConfig, ApiGroupAgentBindings, ApiGroupRecord, ApiRecord, BlobRecord,
    CharacterCardDefinition, CharacterCardRecord, LlmProvider, PlayerProfileRecord,
    PresetAgentConfigs, PresetRecord, SchemaRecord, SessionCharacterRecord, StoryRecord,
};
use story::{NarrativeNode, StoryGraph};

use agents::actor::CharacterCard;

type RecordedRequests = Arc<Mutex<Vec<ChatRequest>>>;
type ChatQueue = Arc<Mutex<VecDeque<Result<ChatResponse, LlmError>>>>;
type StreamQueue = Arc<Mutex<VecDeque<Result<Vec<Result<ChatChunk, LlmError>>, LlmError>>>>;

use std::collections::VecDeque;

#[derive(Clone)]
pub struct QueuedMockLlm {
    requests: RecordedRequests,
    chat_queue: ChatQueue,
    stream_queue: StreamQueue,
}

impl QueuedMockLlm {
    pub fn new(
        chat_results: Vec<Result<ChatResponse, LlmError>>,
        stream_results: Vec<Result<Vec<Result<ChatChunk, LlmError>>, LlmError>>,
    ) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            chat_queue: Arc::new(Mutex::new(VecDeque::from(chat_results))),
            stream_queue: Arc::new(Mutex::new(VecDeque::from(stream_results))),
        }
    }

    pub fn recorded_requests(&self) -> Vec<ChatRequest> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .clone()
    }
}

#[async_trait]
impl LlmApi for QueuedMockLlm {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .push(request);

        self.chat_queue
            .lock()
            .expect("chat queue lock poisoned")
            .pop_front()
            .expect("missing queued chat response")
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmError> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .push(request);

        let chunks = self
            .stream_queue
            .lock()
            .expect("stream queue lock poisoned")
            .pop_front()
            .expect("missing queued stream response")?;

        Ok(Box::pin(stream::iter(chunks)))
    }
}

pub fn assistant_response(
    content: impl Into<String>,
    structured_output: Option<serde_json::Value>,
) -> ChatResponse {
    ChatResponse {
        message: Message::new(Role::Assistant, content),
        model: "test-model".to_owned(),
        finish_reason: Some("stop".to_owned()),
        usage: None,
        structured_output,
    }
}

pub fn sample_character_card() -> CharacterCard {
    CharacterCard {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative, casual".to_owned(),
        state_schema: HashMap::from([(
            "trust".to_owned(),
            StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
        )]),
        system_prompt: "Stay in character.".to_owned(),
    }
}

pub fn sample_story_graph() -> StoryGraph {
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

pub fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

pub fn sample_world_state_schema() -> WorldStateSchema {
    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    schema
}

pub fn sample_archive() -> CharacterArchive {
    CharacterArchive::new(
        sample_character_content(),
        CharacterCoverMimeType::Png,
        b"cover-bytes".to_vec(),
    )
}

pub fn sample_character_content() -> CharacterCardContent {
    CharacterCardContent {
        id: "merchant".to_owned(),
        name: "Haru".to_owned(),
        personality: "greedy but friendly trader".to_owned(),
        style: "talkative, casual".to_owned(),
        schema_id: "schema-character-merchant".to_owned(),
        system_prompt: "Stay in character.".to_owned(),
    }
}

pub fn sample_character_record() -> CharacterCardRecord {
    let archive = sample_archive();

    CharacterCardRecord {
        character_id: archive.content.id.clone(),
        content: CharacterCardDefinition {
            id: archive.content.id.clone(),
            name: archive.content.name.clone(),
            personality: archive.content.personality.clone(),
            style: archive.content.style.clone(),
            schema_id: archive.content.schema_id.clone(),
            system_prompt: archive.content.system_prompt.clone(),
        },
        cover_blob_id: Some("blob-cover-merchant".to_owned()),
        cover_file_name: Some(archive.manifest.cover_path.clone()),
        cover_mime_type: Some(
            serde_json::to_string(&archive.manifest.cover_mime_type)
                .expect("cover mime type should serialize")
                .trim_matches('"')
                .to_owned(),
        ),
    }
}

pub fn sample_blob_record() -> BlobRecord {
    let archive = sample_archive();

    BlobRecord {
        blob_id: "blob-cover-merchant".to_owned(),
        file_name: Some(archive.manifest.cover_path.clone()),
        content_type: archive
            .manifest
            .cover_mime_type
            .as_content_type()
            .to_owned(),
        bytes: archive.cover_bytes.clone(),
    }
}

pub fn sample_resources_payload(resource_id: impl Into<String>) -> StoryResourcesPayload {
    StoryResourcesPayload {
        resource_id: resource_id.into(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec!["merchant".to_owned()],
        player_schema_id_seed: Some("schema-player-default".to_owned()),
        world_schema_id_seed: Some("schema-world-default".to_owned()),
        lorebook_ids: vec![],
        planned_story: None,
    }
}

pub fn sample_story_payload(
    resource_id: impl Into<String>,
    story_id: impl Into<String>,
) -> StoryGeneratedPayload {
    StoryGeneratedPayload {
        resource_id: resource_id.into(),
        story_id: story_id.into(),
        display_name: "Flooded Harbor".to_owned(),
        graph: sample_story_graph(),
        world_schema_id: "schema-world-story-1".to_owned(),
        player_schema_id: "schema-player-story-1".to_owned(),
        introduction: "The courier reaches a flooded dock at dusk.".to_owned(),
        common_variables: vec![],
    }
}

pub fn sample_story_record(
    resource_id: impl Into<String>,
    story_id: impl Into<String>,
) -> StoryRecord {
    let story_id = story_id.into();

    StoryRecord {
        story_id: story_id.clone(),
        display_name: "Flooded Harbor".to_owned(),
        resource_id: resource_id.into(),
        graph: sample_story_graph(),
        world_schema_id: "schema-world-story-1".to_owned(),
        player_schema_id: "schema-player-story-1".to_owned(),
        introduction: sample_story_payload("resource-1", story_id).introduction,
        common_variables: vec![],
        created_at_ms: Some(1_000),
        updated_at_ms: Some(2_000),
    }
}

pub fn sample_api_record(api_id: &str, model_suffix: &str) -> ApiRecord {
    ApiRecord {
        api_id: api_id.to_owned(),
        display_name: format!("API {api_id}"),
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: format!("sk-{model_suffix}-{api_id}"),
        model: format!("{api_id}-{model_suffix}-model"),
    }
}

pub fn sample_api_group_record(api_group_id: &str, model_suffix: &str) -> ApiGroupRecord {
    ApiGroupRecord {
        api_group_id: api_group_id.to_owned(),
        display_name: format!("Group {api_group_id}"),
        agents: ApiGroupAgentBindings {
            planner_api_id: format!("{model_suffix}-planner"),
            architect_api_id: format!("{model_suffix}-architect"),
            director_api_id: format!("{model_suffix}-director"),
            actor_api_id: format!("{model_suffix}-actor"),
            narrator_api_id: format!("{model_suffix}-narrator"),
            keeper_api_id: format!("{model_suffix}-keeper"),
            replyer_api_id: format!("{model_suffix}-replyer"),
        },
    }
}

pub fn sample_preset_record(preset_id: &str, token_base: u32) -> PresetRecord {
    let config = |offset: u32| AgentPresetConfig {
        temperature: Some(0.1 + (offset as f32 * 0.05)),
        max_tokens: Some(token_base + offset * 64),
        extra: None,
        modules: Vec::new(),
    };

    PresetRecord {
        preset_id: preset_id.to_owned(),
        display_name: format!("Preset {preset_id}"),
        agents: PresetAgentConfigs {
            planner: config(0),
            architect: config(1),
            director: config(2),
            actor: config(3),
            narrator: config(4),
            keeper: config(5),
            replyer: config(6),
        },
    }
}

pub fn sample_schema_record(schema_id: &str, display_name: &str) -> SchemaRecord {
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

pub fn sample_player_profile(id: &str, description: &str) -> PlayerProfileRecord {
    PlayerProfileRecord {
        player_profile_id: id.to_owned(),
        display_name: id.to_owned(),
        description: description.to_owned(),
    }
}

pub fn sample_session_character_record(
    session_id: &str,
    session_character_id: &str,
) -> SessionCharacterRecord {
    SessionCharacterRecord {
        session_character_id: session_character_id.to_owned(),
        session_id: session_id.to_owned(),
        display_name: "Dock Guard".to_owned(),
        personality: "dutiful and wary".to_owned(),
        style: "short, formal".to_owned(),
        system_prompt: "Keep watch over the dock.".to_owned(),
        created_at_ms: 3_600,
        updated_at_ms: 3_600,
    }
}
