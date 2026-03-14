#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures_util::stream;
use llm::{ChatChunk, ChatRequest, ChatResponse, ChatStream, LlmApi, LlmError, Message, Role};

type RecordedRequests = Arc<Mutex<Vec<ChatRequest>>>;
type ChatResults = Arc<Mutex<Vec<Result<ChatResponse, LlmError>>>>;
type StreamChunks = Vec<Result<ChatChunk, LlmError>>;
type StreamResultSlot = Arc<Mutex<Option<Result<StreamChunks, LlmError>>>>;

#[derive(Clone)]
pub struct MockLlm {
    requests: RecordedRequests,
    chat_results: ChatResults,
    stream_result: StreamResultSlot,
}

impl MockLlm {
    pub fn with_chat_response(response: ChatResponse) -> Self {
        Self::with_chat_responses(vec![Ok(response)])
    }

    pub fn with_chat_responses(responses: Vec<Result<ChatResponse, LlmError>>) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            chat_results: Arc::new(Mutex::new(responses)),
            stream_result: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with_stream_chunks(chunks: Vec<Result<ChatChunk, LlmError>>) -> Self {
        Self {
            requests: Arc::new(Mutex::new(Vec::new())),
            chat_results: Arc::new(Mutex::new(Vec::new())),
            stream_result: Arc::new(Mutex::new(Some(Ok(chunks)))),
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
impl LlmApi for MockLlm {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .push(request);

        let mut results = self
            .chat_results
            .lock()
            .expect("chat_results lock poisoned");
        assert!(!results.is_empty(), "missing configured chat result");
        results.remove(0)
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmError> {
        self.requests
            .lock()
            .expect("requests lock poisoned")
            .push(request);

        let chunks = self
            .stream_result
            .lock()
            .expect("stream_result lock poisoned")
            .take()
            .expect("missing configured stream result")?;

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
