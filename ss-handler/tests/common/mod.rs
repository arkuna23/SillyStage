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
use ss_handler::store::{CharacterCardRecord, StoryRecord};
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
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
        tendencies: vec!["likes profitable deals".to_owned()],
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
        CharacterCardContent::from(sample_character_card()),
        CharacterCoverMimeType::Png,
        b"cover-bytes".to_vec(),
    )
}

pub fn sample_character_record() -> CharacterCardRecord {
    let archive = sample_archive();
    let summary = archive.summary();

    CharacterCardRecord {
        character_id: summary.character_id.clone(),
        archive,
        summary,
    }
}

pub fn sample_resources_payload(resource_id: impl Into<String>) -> StoryResourcesPayload {
    StoryResourcesPayload {
        resource_id: resource_id.into(),
        story_concept: "A flooded harbor story.".to_owned(),
        character_ids: vec!["merchant".to_owned()],
        player_state_schema_seed: sample_player_state_schema(),
        world_state_schema_seed: Some(sample_world_state_schema()),
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
        graph: sample_story_graph(),
        world_state_schema: sample_world_state_schema(),
        player_state_schema: sample_player_state_schema(),
        introduction: "The courier reaches a flooded dock at dusk.".to_owned(),
    }
}

pub fn sample_story_record(
    resource_id: impl Into<String>,
    story_id: impl Into<String>,
) -> StoryRecord {
    let story_id = story_id.into();

    StoryRecord {
        story_id: story_id.clone(),
        resource_id: resource_id.into(),
        generated: sample_story_payload("resource-1", story_id),
    }
}
