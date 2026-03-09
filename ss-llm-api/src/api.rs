use std::pin::Pin;

use async_trait::async_trait;
use futures_core::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::LlmError;

pub type ChatStream = Pin<Box<dyn Stream<Item = Result<ChatChunk, LlmError>> + Send>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn new(role: Role, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<Message>,
    pub model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub response_format: Option<ResponseFormat>,
}

impl ChatRequest {
    pub fn builder() -> ChatRequestBuilder {
        ChatRequestBuilder::default()
    }

    pub fn validate(&self) -> Result<(), LlmError> {
        if self.messages.is_empty() {
            return Err(LlmError::InvalidRequest(
                "chat request requires at least one message".to_owned(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChatRequestBuilder {
    messages: Vec<Message>,
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    response_format: Option<ResponseFormat>,
}

impl ChatRequestBuilder {
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }

    pub fn messages<I>(mut self, messages: I) -> Self
    where
        I: IntoIterator<Item = Message>,
    {
        self.messages.extend(messages);
        self
    }

    pub fn system_message(self, content: impl Into<String>) -> Self {
        self.message(Message::new(Role::System, content))
    }

    pub fn user_message(self, content: impl Into<String>) -> Self {
        self.message(Message::new(Role::User, content))
    }

    pub fn assistant_message(self, content: impl Into<String>) -> Self {
        self.message(Message::new(Role::Assistant, content))
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    pub fn build(self) -> Result<ChatRequest, LlmError> {
        let request = ChatRequest {
            messages: self.messages,
            model: self.model,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            response_format: self.response_format,
        };

        request.validate()?;
        Ok(request)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    JsonObject,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Message,
    pub model: String,
    pub finish_reason: Option<String>,
    pub usage: Option<Usage>,
    pub structured_output: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatChunk {
    pub delta: String,
    pub model: Option<String>,
    pub finish_reason: Option<String>,
    pub done: bool,
    pub usage: Option<Usage>,
}

#[async_trait]
pub trait LlmApi: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError>;

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_valid_request() {
        let request = ChatRequest::builder()
            .system_message("You are helpful")
            .user_message("hello")
            .model("gpt-4.1-mini")
            .temperature(0.2)
            .max_tokens(256)
            .response_format(ResponseFormat::JsonObject)
            .build()
            .expect("request should build");

        assert_eq!(request.messages.len(), 2);
        assert_eq!(
            request.messages[0],
            Message::new(Role::System, "You are helpful")
        );
        assert_eq!(request.messages[1], Message::new(Role::User, "hello"));
        assert_eq!(request.model.as_deref(), Some("gpt-4.1-mini"));
        assert_eq!(request.temperature, Some(0.2));
        assert_eq!(request.max_tokens, Some(256));
        assert_eq!(request.response_format, Some(ResponseFormat::JsonObject));
    }

    #[test]
    fn builder_rejects_missing_messages() {
        let error = ChatRequest::builder()
            .build()
            .expect_err("request without messages should fail");

        assert!(matches!(error, LlmError::InvalidRequest(_)));
    }
}
