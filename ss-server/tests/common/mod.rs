#![allow(dead_code)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use engine::{AgentApiIds, LlmApiRegistry};
use llm::{ChatChunk, ChatRequest, ChatResponse, ChatStream, LlmApi, LlmError, Message, Role};
use protocol::CharacterArchive;
use serde_json::json;
use state::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use story::{NarrativeNode, StoryGraph};

use agents::actor::CharacterCard;

type RecordedRequests = Arc<Mutex<Vec<ChatRequest>>>;
type ChatQueue = Arc<Mutex<VecDeque<Result<ChatResponse, LlmError>>>>;
type StreamQueue = Arc<Mutex<VecDeque<Result<Vec<Result<ChatChunk, LlmError>>, LlmError>>>>;

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
        self.requests.lock().expect("requests lock poisoned").push(request);

        self.chat_queue
            .lock()
            .expect("chat queue lock poisoned")
            .pop_front()
            .expect("missing queued chat response")
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmError> {
        self.requests.lock().expect("requests lock poisoned").push(request);

        let chunks = self
            .stream_queue
            .lock()
            .expect("stream queue lock poisoned")
            .pop_front()
            .expect("missing queued stream response")?;

        Ok(Box::pin(futures_util::stream::iter(chunks)))
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

pub fn default_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner-default".to_owned(),
        architect_api_id: "architect-default".to_owned(),
        director_api_id: "director-default".to_owned(),
        actor_api_id: "actor-default".to_owned(),
        narrator_api_id: "narrator-default".to_owned(),
        keeper_api_id: "keeper-default".to_owned(),
    }
}

pub fn registry_with_ids(llm: Arc<QueuedMockLlm>) -> LlmApiRegistry {
    let default = default_api_ids();
    let llm: Arc<dyn LlmApi> = llm;

    LlmApiRegistry::new()
        .register(default.planner_api_id, Arc::clone(&llm), "planner-model")
        .register(default.architect_api_id, Arc::clone(&llm), "architect-model")
        .register(default.director_api_id, Arc::clone(&llm), "director-model")
        .register(default.actor_api_id, Arc::clone(&llm), "actor-model")
        .register(default.narrator_api_id, Arc::clone(&llm), "narrator-model")
        .register(default.keeper_api_id, llm, "keeper-model")
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
        protocol::CharacterCardContent::from(sample_character_card()),
        protocol::CharacterCoverMimeType::Png,
        b"cover-bytes".to_vec(),
    )
}
