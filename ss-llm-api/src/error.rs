use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("transport error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("authentication failed")]
    Authentication,
    #[error("rate limited by provider")]
    RateLimited,
    #[error("provider error ({status}): {message}")]
    Provider { status: u16, message: String },
    #[error("stream parsing error: {0}")]
    StreamParse(String),
    #[error("structured output parse error: {0}")]
    StructuredOutputParse(String),
}
