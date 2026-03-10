#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures_util::stream;
use llm::{ChatChunk, ChatRequest, ChatResponse, ChatStream, LlmApi, LlmError, Message, Role};

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
