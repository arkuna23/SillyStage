use std::time::Duration;

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api::{
    ChatChunk, ChatRequest, ChatResponse, ChatStream, LlmApi, Message, ResponseFormat, Role, Usage,
};
use crate::error::LlmError;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
    pub default_model: String,
    pub timeout: Duration,
}

impl OpenAiConfig {
    pub fn builder() -> OpenAiConfigBuilder {
        OpenAiConfigBuilder::default()
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiConfigBuilder {
    api_key: Option<String>,
    base_url: String,
    default_model: Option<String>,
    timeout: Duration,
}

impl Default for OpenAiConfigBuilder {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: DEFAULT_BASE_URL.to_owned(),
            default_model: None,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }
}

impl OpenAiConfigBuilder {
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = Some(model.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<OpenAiConfig, LlmError> {
        let api_key = self
            .api_key
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| LlmError::InvalidConfig("OpenAI API key is required".to_owned()))?;

        let default_model = self
            .default_model
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| LlmError::InvalidConfig("default model is required".to_owned()))?;

        let base_url = self.base_url.trim().trim_end_matches('/').to_owned();
        if base_url.is_empty() {
            return Err(LlmError::InvalidConfig(
                "base URL cannot be empty".to_owned(),
            ));
        }

        Ok(OpenAiConfig {
            api_key,
            base_url,
            default_model,
            timeout: self.timeout,
        })
    }
}

#[derive(Debug, Clone)]
pub struct OpenAiClient {
    http: reqwest::Client,
    config: OpenAiConfig,
}

impl OpenAiClient {
    pub fn new(config: OpenAiConfig) -> Result<Self, LlmError> {
        let http = reqwest::Client::builder().timeout(config.timeout).build()?;
        Ok(Self { http, config })
    }

    fn resolved_model<'a>(&'a self, request: &'a ChatRequest) -> &'a str {
        request
            .model
            .as_deref()
            .unwrap_or(self.config.default_model.as_str())
    }

    fn completions_url(&self) -> String {
        format!("{}/chat/completions", self.config.base_url)
    }

    fn build_request_body(&self, request: &ChatRequest, stream: bool) -> OpenAiChatRequest {
        OpenAiChatRequest {
            model: self.resolved_model(request).to_owned(),
            messages: request.messages.iter().map(OpenAiMessage::from).collect(),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            response_format: request
                .response_format
                .clone()
                .map(OpenAiResponseFormat::from),
            stream,
        }
    }

    async fn send_request(
        &self,
        request: &ChatRequest,
        stream: bool,
    ) -> Result<reqwest::Response, LlmError> {
        request.validate()?;

        let response = self
            .http
            .post(self.completions_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.config.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&self.build_request_body(request, stream))
            .send()
            .await?;

        Self::handle_error_status(response).await
    }

    async fn handle_error_status(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, LlmError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = response.text().await?;
        let message = serde_json::from_str::<OpenAiErrorEnvelope>(&body)
            .ok()
            .map(|payload| payload.error.message)
            .unwrap_or(body);

        match status.as_u16() {
            401 => Err(LlmError::Authentication),
            429 => Err(LlmError::RateLimited),
            code => Err(LlmError::Provider {
                status: code,
                message,
            }),
        }
    }
}

#[async_trait]
impl LlmApi for OpenAiClient {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LlmError> {
        let response = self.send_request(&request, false).await?;
        let payload: OpenAiChatResponse = response.json().await?;
        payload.into_chat_response(request.response_format.as_ref())
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<ChatStream, LlmError> {
        if matches!(request.response_format, Some(ResponseFormat::JsonObject)) {
            return Err(LlmError::InvalidRequest(
                "json_object response format is not supported for chat_stream".to_owned(),
            ));
        }

        let response = self.send_request(&request, true).await?;
        let stream = response
            .bytes_stream()
            .eventsource()
            .map(|event| match event {
                Ok(event) => map_stream_event(event.data),
                Err(error) => Err(LlmError::StreamParse(error.to_string())),
            });

        Ok(Box::pin(stream))
    }
}

fn map_stream_event(data: String) -> Result<ChatChunk, LlmError> {
    if data == "[DONE]" {
        return Ok(ChatChunk {
            delta: String::new(),
            model: None,
            finish_reason: None,
            done: true,
            usage: None,
        });
    }

    let payload: OpenAiStreamResponse = serde_json::from_str(&data)?;
    let choice =
        payload.choices.into_iter().next().ok_or_else(|| {
            LlmError::StreamParse("stream response contained no choices".to_owned())
        })?;

    Ok(ChatChunk {
        delta: choice.delta.content.unwrap_or_default(),
        model: payload.model,
        finish_reason: choice.finish_reason,
        done: false,
        usage: payload.usage.map(Usage::from),
    })
}

#[derive(Debug, Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<OpenAiResponseFormat>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Debug, Serialize)]
struct OpenAiResponseFormat {
    #[serde(rename = "type")]
    kind: ResponseFormat,
}

impl From<ResponseFormat> for OpenAiResponseFormat {
    fn from(format: ResponseFormat) -> Self {
        Self { kind: format }
    }
}

#[derive(Debug, Serialize)]
struct OpenAiMessage {
    role: &'static str,
    content: String,
}

impl From<&Message> for OpenAiMessage {
    fn from(message: &Message) -> Self {
        Self {
            role: match message.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
            },
            content: message.content.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

impl OpenAiChatResponse {
    fn into_chat_response(
        self,
        response_format: Option<&ResponseFormat>,
    ) -> Result<ChatResponse, LlmError> {
        let choice = self
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| LlmError::Provider {
                status: 200,
                message: "OpenAI response contained no choices".to_owned(),
            })?;

        let content = choice.message.content.unwrap_or_default();
        let structured_output = parse_structured_output(response_format, &content)?;

        Ok(ChatResponse {
            message: Message::new(role_from_openai(&choice.message.role), content),
            model: self.model,
            finish_reason: choice.finish_reason,
            usage: self.usage.map(Usage::from),
            structured_output,
        })
    }
}

fn parse_structured_output(
    response_format: Option<&ResponseFormat>,
    content: &str,
) -> Result<Option<Value>, LlmError> {
    match response_format {
        Some(ResponseFormat::JsonObject) => serde_json::from_str(content)
            .map(Some)
            .map_err(|error| LlmError::StructuredOutputParse(error.to_string())),
        None => Ok(None),
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    role: String,
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamResponse {
    model: Option<String>,
    choices: Vec<OpenAiStreamChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiStreamChoice {
    delta: OpenAiDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct OpenAiDelta {
    content: Option<String>,
}

#[derive(Debug, Copy, Clone, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl From<OpenAiUsage> for Usage {
    fn from(usage: OpenAiUsage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorEnvelope {
    error: OpenAiErrorBody,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorBody {
    message: String,
}

fn role_from_openai(role: &str) -> Role {
    match role {
        "system" => Role::System,
        "assistant" => Role::Assistant,
        _ => Role::User,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_rejects_missing_api_key() {
        let error = OpenAiConfig::builder()
            .default_model("gpt-4o-mini")
            .build()
            .expect_err("builder should require api key");

        assert!(matches!(error, LlmError::InvalidConfig(_)));
    }

    #[test]
    fn builder_trims_and_normalizes_base_url() {
        let config = OpenAiConfig::builder()
            .api_key("test-key")
            .default_model("gpt-4o-mini")
            .base_url(" http://localhost:8080/ ")
            .build()
            .expect("config should build");

        assert_eq!(config.base_url, "http://localhost:8080");
    }

    #[test]
    fn stream_done_event_maps_to_terminal_chunk() {
        let chunk = map_stream_event("[DONE]".to_owned()).expect("done chunk should parse");

        assert!(chunk.done);
        assert!(chunk.delta.is_empty());
    }

    #[test]
    fn structured_output_parse_rejects_invalid_json() {
        let error = parse_structured_output(Some(&ResponseFormat::JsonObject), "not json")
            .expect_err("invalid json should fail");

        assert!(matches!(error, LlmError::StructuredOutputParse(_)));
    }
}
