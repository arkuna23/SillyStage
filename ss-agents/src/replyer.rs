use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::Arc;

use llm::{ChatRequest, LlmApi, ResponseFormat};
use serde::{Deserialize, Serialize};

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use crate::prompt::{
    SystemPromptEntry, append_system_prompt_entries, render_character_summaries, render_node,
    render_observable_world_state, render_player, render_sections, render_state_schema_fields,
};
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
    pub current_cast_ids: &'a [String],
    pub lorebook_base: Option<&'a str>,
    pub lorebook_matched: Option<&'a str>,
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

    pub fn with_system_prompt_entries(mut self, entries: &[SystemPromptEntry]) -> Self {
        self.system_prompt = append_system_prompt_entries(&self.system_prompt, entries);
        self
    }

    pub async fn suggest(
        &self,
        request: ReplyerRequest<'_>,
    ) -> Result<ReplyerResponse, ReplyerError> {
        Self::validate_request(&request)?;

        let (stable_prompt, dynamic_prompt) = self.build_user_prompts(&request)?;
        let output = self
            .llm
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(&self.model)
                    .system_message(&self.system_prompt)
                    .user_message(stable_prompt)
                    .user_message(dynamic_prompt)
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

        for character_id in request.current_cast_ids {
            if !cards_by_id.contains_key(character_id.as_str()) {
                return Err(ReplyerError::InvalidRequest(format!(
                    "missing character card for current node id '{character_id}'"
                )));
            }
        }

        Ok(())
    }

    fn build_user_prompts(
        &self,
        request: &ReplyerRequest<'_>,
    ) -> Result<(String, String), ReplyerError> {
        let mut stable_sections = Vec::new();
        if let Some(lorebook_base) = request.lorebook_base.as_deref() {
            stable_sections.push(("LOREBOOK_BASE", lorebook_base.to_owned()));
        }
        stable_sections.push((
            "PLAYER",
            render_player(request.player_name, request.player_description),
        ));
        stable_sections.push(("REPLY_LIMIT", request.limit.to_string()));
        stable_sections.push((
            "CURRENT_CAST",
            render_character_summaries(&self.current_cast_summaries(request)?, request.player_name),
        ));
        stable_sections.push(("CURRENT_NODE", render_node(request.current_node)));
        stable_sections.push((
            "PLAYER_STATE_SCHEMA",
            render_state_schema_fields(&request.player_state_schema.fields),
        ));

        let mut dynamic_sections = vec![
            (
                "WORLD_STATE",
                render_observable_world_state(request.world_state),
            ),
            ("SESSION_HISTORY", render_reply_history(request.history)),
        ];
        if let Some(lorebook_matched) = request.lorebook_matched.as_deref() {
            dynamic_sections.push(("LOREBOOK_MATCHED", lorebook_matched.to_owned()));
        }

        let stable_prompt = render_sections(&stable_sections);
        let dynamic_prompt = render_sections(&dynamic_sections);

        Ok((stable_prompt, dynamic_prompt))
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
            .current_cast_ids
            .iter()
            .map(|character_id| {
                cast_by_id
                    .get(character_id.as_str())
                    .map(|card| {
                        card.summary_ref(request.world_state.character_states(character_id))
                    })
                    .ok_or_else(|| {
                        ReplyerError::InvalidRequest(format!(
                            "missing character card for current cast id '{character_id}'"
                        ))
                    })
            })
            .collect()
    }
}

fn render_reply_history(history: &[ReplyHistoryMessage]) -> String {
    if history.is_empty() {
        return "- none".to_owned();
    }

    history
        .iter()
        .map(|message| {
            format!(
                "- [turn:{}|{}|{}|{}] {}",
                message.turn_index,
                message.speaker_id,
                message.speaker_name,
                serde_json::to_string(&message.kind).unwrap_or_default(),
                crate::prompt::normalize_inline_text(&message.text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
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
