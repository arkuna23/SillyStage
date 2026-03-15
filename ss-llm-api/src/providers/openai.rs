use std::time::Duration;

use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error, info, warn};

use crate::api::{
    ChatChunk, ChatRequest, ChatResponse, ChatStream, LlmApi, Message, ResponseFormat, Role, Usage,
};
use crate::error::LlmError;

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 180;
const MAX_LOGGED_PROMPT_CHARS: usize = 1_200;
const MAX_LOGGED_RESPONSE_CHARS: usize = 2_000;

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
        let http = reqwest::Client::builder()
            .tls_backend_rustls()
            .timeout(config.timeout)
            .build()?;
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
        let request_body = self.build_request_body(request, stream);

        log_llm_payload(
            stream,
            self.completions_url(),
            &request_body,
            "sending llm request",
        );

        let response = self
            .http
            .post(self.completions_url())
            .header(AUTHORIZATION, format!("Bearer {}", self.config.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|error| {
                error!(
                    provider = "openai",
                    url = %self.completions_url(),
                    error = %error,
                    error_chain = %error_chain_for_log(&error),
                    "failed to send llm request"
                );
                LlmError::from(error)
            })?;

        Self::handle_error_status(response).await
    }

    async fn handle_error_status(
        response: reqwest::Response,
    ) -> Result<reqwest::Response, LlmError> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }

        let body = read_response_body(response, "error").await?;
        warn!(
            provider = "openai",
            status = status.as_u16(),
            body = %body,
            "llm request failed"
        );
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
        let body = read_response_body(response, "success").await?;
        info!(
            provider = "openai",
            body = %body,
            "received llm response body"
        );
        let payload: OpenAiChatResponse = serde_json::from_str(&body).map_err(|error| {
            error!(
                provider = "openai",
                error = %error,
                body = %truncate_response_for_log(&body),
                "llm provider returned an invalid json response body"
            );
            LlmError::from(error)
        })?;
        let response = payload.into_chat_response(request.response_format.as_ref())?;
        info!(
            provider = "openai",
            payload = %json_for_log(&response),
            "parsed llm response"
        );
        Ok(response)
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
        let chunk = ChatChunk {
            delta: String::new(),
            model: None,
            finish_reason: None,
            done: true,
            usage: None,
        };
        debug!(
            provider = "openai",
            payload = %json_for_log(&chunk),
            "received llm stream chunk"
        );
        return Ok(chunk);
    }

    debug!(
        provider = "openai",
        body = %data,
        "received llm stream event body"
    );
    let payload: OpenAiStreamResponse = serde_json::from_str(&data)?;
    let choice =
        payload.choices.into_iter().next().ok_or_else(|| {
            LlmError::StreamParse("stream response contained no choices".to_owned())
        })?;

    let chunk = ChatChunk {
        delta: choice.delta.content.unwrap_or_default(),
        model: payload.model,
        finish_reason: choice.finish_reason,
        done: false,
        usage: payload.usage.map(Usage::from),
    };
    debug!(
        provider = "openai",
        payload = %json_for_log(&chunk),
        "received llm stream chunk"
    );
    Ok(chunk)
}

fn json_for_log<T: Serialize>(payload: &T) -> String {
    serde_json::to_string(payload)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"))
}

fn log_llm_payload<T: Serialize>(stream: bool, url: String, payload: &T, message: &str) {
    let payload = json_for_llm_request_log(payload);
    if stream {
        debug!(
            provider = "openai",
            stream,
            url = %url,
            payload = %payload,
            "{message}"
        );
    } else {
        info!(
            provider = "openai",
            stream,
            url = %url,
            payload = %payload,
            "{message}"
        );
    }
}

fn json_for_llm_request_log<T: Serialize>(payload: &T) -> String {
    let mut value = match serde_json::to_value(payload) {
        Ok(value) => value,
        Err(error) => return format!("{{\"serialization_error\":\"{error}\"}}"),
    };

    shape_prompt_for_log_value(&mut value);

    serde_json::to_string_pretty(&value)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"))
}

fn shape_prompt_for_log_value(value: &mut Value) {
    let Some(messages) = value.get_mut("messages").and_then(Value::as_array_mut) else {
        return;
    };

    for message in messages {
        let Some(message_object) = message.as_object_mut() else {
            continue;
        };

        let Some(content) = message_object
            .get("content")
            .and_then(Value::as_str)
            .map(str::to_owned)
        else {
            continue;
        };

        let total_chars = content.chars().count();
        let truncated = total_chars > MAX_LOGGED_PROMPT_CHARS;
        let preview = if truncated {
            truncate_for_log(&content, MAX_LOGGED_PROMPT_CHARS)
        } else {
            content
        };
        let content_lines = preview
            .lines()
            .map(|line| Value::String(line.to_owned()))
            .collect::<Vec<_>>();

        message_object.remove("content");
        message_object.insert("content_lines".to_owned(), Value::Array(content_lines));
        message_object.insert(
            "content_char_count".to_owned(),
            Value::Number(serde_json::Number::from(total_chars as u64)),
        );
        message_object.insert("content_truncated".to_owned(), Value::Bool(truncated));
    }
}

fn truncate_for_log(content: &str, max_chars: usize) -> String {
    let total_chars = content.chars().count();
    let truncated: String = content.chars().take(max_chars).collect();
    format!(
        "{truncated}...[truncated {}/{} chars]",
        max_chars, total_chars
    )
}

fn truncate_response_for_log(content: &str) -> String {
    if content.chars().count() <= MAX_LOGGED_RESPONSE_CHARS {
        return content.to_owned();
    }

    truncate_for_log(content, MAX_LOGGED_RESPONSE_CHARS)
}

async fn read_response_body(
    response: reqwest::Response,
    kind: &'static str,
) -> Result<String, LlmError> {
    let response_context = response_context(&response);
    let mut stream = response.bytes_stream();
    let mut body = Vec::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => body.extend_from_slice(&chunk),
            Err(error) => {
                let partial_body = response_body_preview(&body);
                error!(
                    provider = "openai",
                    status = response_context.status,
                    url = %response_context.url,
                    content_type = ?response_context.content_type,
                    content_encoding = ?response_context.content_encoding,
                    content_length = ?response_context.content_length,
                    received_body_bytes = body.len(),
                    error = %error,
                    error_chain = %error_chain_for_log(&error),
                    partial_body = %partial_body,
                    "failed to decode llm {kind} response body"
                );
                return Err(LlmError::from(error));
            }
        }
    }

    Ok(String::from_utf8_lossy(&body).into_owned())
}

fn response_body_preview(body: &[u8]) -> String {
    if body.is_empty() {
        return "<no body bytes received before decode failure>".to_owned();
    }

    truncate_response_for_log(&String::from_utf8_lossy(body))
}

fn error_chain_for_log(error: &(dyn std::error::Error + 'static)) -> String {
    let mut chain = Vec::new();
    let mut current = error.source();

    while let Some(source) = current {
        chain.push(source.to_string());
        current = source.source();
    }

    if chain.is_empty() {
        "<none>".to_owned()
    } else {
        chain.join(": ")
    }
}

#[derive(Debug)]
struct ResponseContext {
    status: u16,
    url: String,
    content_type: Option<String>,
    content_encoding: Option<String>,
    content_length: Option<String>,
}

fn response_context(response: &reqwest::Response) -> ResponseContext {
    ResponseContext {
        status: response.status().as_u16(),
        url: response.url().to_string(),
        content_type: header_value_to_owned(response, CONTENT_TYPE),
        content_encoding: header_value_to_owned(response, CONTENT_ENCODING),
        content_length: header_value_to_owned(response, CONTENT_LENGTH),
    }
}

fn header_value_to_owned(
    response: &reqwest::Response,
    header_name: reqwest::header::HeaderName,
) -> Option<String> {
    response
        .headers()
        .get(header_name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
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
        Some(ResponseFormat::JsonObject) => {
            serde_json::from_str(content).map(Some).map_err(|error| {
                error!(
                    provider = "openai",
                    error = %error,
                    content = %truncate_response_for_log(content),
                    "llm returned invalid structured json content"
                );
                LlmError::StructuredOutputParse {
                    message: error.to_string(),
                    raw_content: content.to_owned(),
                }
            })
        }
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
    use std::error::Error;
    use std::fmt::{self, Display, Formatter};

    use super::*;

    #[derive(Debug)]
    struct RootError;

    impl Display for RootError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str("socket closed")
        }
    }

    impl Error for RootError {}

    #[derive(Debug)]
    struct WrappedError {
        source: RootError,
    }

    impl Display for WrappedError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            f.write_str("body decode failed")
        }
    }

    impl Error for WrappedError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.source)
        }
    }

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
    fn builder_uses_180_second_default_timeout() {
        let config = OpenAiConfig::builder()
            .api_key("test-key")
            .default_model("gpt-4o-mini")
            .build()
            .expect("config should build");

        assert_eq!(config.timeout, Duration::from_secs(180));
    }

    #[test]
    fn response_body_preview_uses_placeholder_when_no_bytes_were_received() {
        assert_eq!(
            response_body_preview(&[]),
            "<no body bytes received before decode failure>"
        );
    }

    #[test]
    fn error_chain_for_log_collects_nested_sources() {
        let top = WrappedError { source: RootError };
        let chain = error_chain_for_log(&top);
        assert!(chain.contains("socket closed"));
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

        assert!(matches!(
            error,
            LlmError::StructuredOutputParse {
                message: _,
                raw_content: _
            }
        ));
    }

    #[test]
    fn llm_request_log_truncates_long_prompt_content() {
        let original = format!("line1\n{}", "a".repeat(MAX_LOGGED_PROMPT_CHARS + 25));
        let payload = OpenAiChatRequest {
            model: "gpt-4.1".to_owned(),
            messages: vec![OpenAiMessage {
                role: "user",
                content: original.clone(),
            }],
            temperature: Some(0.2),
            max_tokens: Some(128),
            response_format: None,
            stream: false,
        };

        let json = json_for_llm_request_log(&payload);

        assert!(json.contains("\"content_lines\""));
        assert!(json.contains("\"content_truncated\": true"));
        assert!(json.contains(&format!(
            "\"content_char_count\": {}",
            original.chars().count()
        )));
        assert!(json.contains("[truncated"));
        assert!(!json.contains("\\n"));
    }

    #[test]
    fn llm_response_log_truncates_long_content() {
        let original = "b".repeat(MAX_LOGGED_RESPONSE_CHARS + 40);

        let truncated = truncate_response_for_log(&original);

        assert!(truncated.contains(&format!(
            "[truncated {}/{} chars]",
            MAX_LOGGED_RESPONSE_CHARS,
            original.chars().count()
        )));
        assert!(!truncated.contains(&original));
    }
}
