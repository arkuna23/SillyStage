use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use llm::{ChatRequest, LlmApi, ResponseFormat};
use serde::{Deserialize, Serialize};

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use state::{PlayerStateSchema, WorldState};
use story::NarrativeNode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplyHistoryKind {
    PlayerInput,
    Narration,
    Dialogue,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplyHistoryMessage {
    pub kind: ReplyHistoryKind,
    pub turn_index: u64,
    pub speaker_id: String,
    pub speaker_name: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ReplyerRequest<'a> {
    pub current_node: &'a NarrativeNode,
    pub character_cards: &'a [CharacterCard],
    pub player_name: Option<&'a str>,
    pub player_description: &'a str,
    pub player_state_schema: &'a PlayerStateSchema,
    pub world_state: &'a WorldState,
    pub history: &'a [ReplyHistoryMessage],
    pub limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplyOption {
    pub id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyerResponse {
    pub replies: Vec<ReplyOption>,
    pub output: llm::ChatResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReplyerOutput {
    replies: Vec<ReplyOption>,
}

pub struct Replyer {
    llm: Arc<dyn LlmApi>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Replyer {
    pub fn new(llm: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, ReplyerError> {
        Self::new_with_options(llm, model, None, None)
    }

    pub fn new_with_options(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Self, ReplyerError> {
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt: include_str!("./prompts/replyer.txt").to_owned(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_prompt_file(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, ReplyerError> {
        let system_prompt = fs::read_to_string(path).map_err(ReplyerError::ReadPrompt)?;
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt,
            temperature: None,
            max_tokens: None,
        })
    }

    pub async fn suggest(
        &self,
        request: ReplyerRequest<'_>,
    ) -> Result<ReplyerResponse, ReplyerError> {
        Self::validate_request(&request)?;

        let user_prompt = self.build_user_prompt(&request)?;
        let output = self
            .llm
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(&self.model)
                    .system_message(&self.system_prompt)
                    .user_message(user_prompt)
                    .response_format(ResponseFormat::JsonObject);
                if let Some(temperature) = self.temperature {
                    builder = builder.temperature(temperature);
                }
                if let Some(max_tokens) = self.max_tokens {
                    builder = builder.max_tokens(max_tokens);
                }
                builder.build()?
            })
            .await?;

        let structured = output
            .structured_output
            .clone()
            .ok_or(ReplyerError::MissingOutput)?;
        let parsed: ReplyerOutput =
            serde_json::from_value(structured).map_err(ReplyerError::InvalidJson)?;
        let replies = sanitize_replies(parsed.replies, request.limit)?;

        Ok(ReplyerResponse { replies, output })
    }

    fn validate_request(request: &ReplyerRequest<'_>) -> Result<(), ReplyerError> {
        if request.limit == 0 {
            return Err(ReplyerError::InvalidRequest(
                "reply limit must be greater than zero".to_owned(),
            ));
        }

        let cards_by_id: HashMap<&str, &CharacterCard> = request
            .character_cards
            .iter()
            .map(|card| (card.id.as_str(), card))
            .collect();

        for character_id in &request.current_node.characters {
            if !cards_by_id.contains_key(character_id.as_str()) {
                return Err(ReplyerError::InvalidRequest(format!(
                    "missing character card for current node id '{character_id}'"
                )));
            }
        }

        Ok(())
    }

    fn build_user_prompt(&self, request: &ReplyerRequest<'_>) -> Result<String, ReplyerError> {
        let current_node_json = serde_json::to_string_pretty(request.current_node)
            .map_err(ReplyerError::SerializePromptData)?;
        let current_cast_json =
            serde_json::to_string_pretty(&self.current_cast_summaries(request)?)
                .map_err(ReplyerError::SerializePromptData)?;
        let player_name_json = serde_json::to_string_pretty(&request.player_name)
            .map_err(ReplyerError::SerializePromptData)?;
        let player_state_schema_json = serde_json::to_string_pretty(request.player_state_schema)
            .map_err(ReplyerError::SerializePromptData)?;
        let world_state_json =
            serde_json::to_string_pretty(&request.world_state.observable_prompt_view())
                .map_err(ReplyerError::SerializePromptData)?;
        let history_json = serde_json::to_string_pretty(request.history)
            .map_err(ReplyerError::SerializePromptData)?;

        Ok(format!(
            "REPLY_LIMIT:\n{}\n\nPLAYER_NAME:\n{}\n\nPLAYER_DESCRIPTION:\n{}\n\nCURRENT_CAST:\n{}\n\nCURRENT_NODE:\n{}\n\nPLAYER_STATE_SCHEMA:\n{}\n\nWORLD_STATE:\n{}\n\nSESSION_HISTORY:\n{}",
            request.limit,
            player_name_json,
            request.player_description,
            current_cast_json,
            current_node_json,
            player_state_schema_json,
            world_state_json,
            history_json
        ))
    }

    fn current_cast_summaries<'a>(
        &self,
        request: &'a ReplyerRequest<'a>,
    ) -> Result<Vec<CharacterCardSummaryRef<'a>>, ReplyerError> {
        let cast_by_id: HashMap<&str, &CharacterCard> = request
            .character_cards
            .iter()
            .map(|card| (card.id.as_str(), card))
            .collect();

        request
            .current_node
            .characters
            .iter()
            .map(|character_id| {
                cast_by_id
                    .get(character_id.as_str())
                    .map(|card| card.summary_ref())
                    .ok_or_else(|| {
                        ReplyerError::InvalidRequest(format!(
                            "missing character card for current cast id '{character_id}'"
                        ))
                    })
            })
            .collect()
    }
}

fn sanitize_replies(
    replies: Vec<ReplyOption>,
    limit: usize,
) -> Result<Vec<ReplyOption>, ReplyerError> {
    let mut normalized = Vec::new();
    let mut seen_texts = HashSet::new();
    let mut seen_ids = HashSet::new();

    for reply in replies {
        let text = reply.text.trim();
        if text.is_empty() {
            continue;
        }

        let mut id = reply.id.trim().to_owned();
        if id.is_empty() || !seen_ids.insert(id.clone()) {
            id = format!("reply-{}", normalized.len());
            seen_ids.insert(id.clone());
        }

        let text_key = text.to_owned();
        if !seen_texts.insert(text_key.clone()) {
            continue;
        }

        normalized.push(ReplyOption { id, text: text_key });
        if normalized.len() >= limit {
            break;
        }
    }

    if normalized.is_empty() {
        return Err(ReplyerError::InvalidReplies(
            "replyer output did not contain any valid replies".to_owned(),
        ));
    }

    Ok(normalized)
}

#[derive(Debug, thiserror::Error)]
pub enum ReplyerError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error(transparent)]
    ReadPrompt(std::io::Error),
    #[error(transparent)]
    SerializePromptData(serde_json::Error),
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error("replyer response did not include structured output")]
    MissingOutput,
    #[error("replyer response contained invalid json: {0}")]
    InvalidJson(serde_json::Error),
    #[error("replyer response was invalid: {0}")]
    InvalidReplies(String),
}
