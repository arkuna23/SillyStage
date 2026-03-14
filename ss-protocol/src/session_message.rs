use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionMessageKind {
    PlayerInput,
    Narration,
    Dialogue,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreateSessionMessageParams {
    pub kind: SessionMessageKind,
    pub speaker_id: String,
    pub speaker_name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetSessionMessageParams {
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ListSessionMessagesParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpdateSessionMessageParams {
    pub message_id: String,
    pub kind: SessionMessageKind,
    pub speaker_id: String,
    pub speaker_name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeleteSessionMessageParams {
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagePayload {
    pub message_id: String,
    pub kind: SessionMessageKind,
    pub sequence: u64,
    pub turn_index: u64,
    pub recorded_at_ms: u64,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub speaker_id: String,
    pub speaker_name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessagesListedPayload {
    pub messages: Vec<SessionMessagePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionMessageDeletedPayload {
    pub message_id: String,
}
