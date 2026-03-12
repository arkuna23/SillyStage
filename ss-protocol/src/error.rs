use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    ParseError,
    InvalidRequest,
    MethodNotFound,
    InvalidParams,
    InternalError,
    NotFound,
    Conflict,
    BackendError,
    StreamError,
}

impl ErrorCode {
    pub const fn rpc_code(self) -> i32 {
        match self {
            Self::ParseError => -32_700,
            Self::InvalidRequest => -32_600,
            Self::MethodNotFound => -32_601,
            Self::InvalidParams => -32_602,
            Self::InternalError => -32_603,
            Self::NotFound => 40_404,
            Self::Conflict => 40_909,
            Self::BackendError => 50_001,
            Self::StreamError => 50_002,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorPayload {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl ErrorPayload {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code: code.rpc_code(),
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}
