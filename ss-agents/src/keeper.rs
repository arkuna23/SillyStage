use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use crate::actor::{ActorResponse, ActorSegmentKind, CharacterCard, CharacterCardSummaryRef};
use crate::director::{ActorPurpose, NarratorPurpose};
use crate::narrator::NarratorResponse;
use crate::prompt::{
    compact_json, normalize_inline_text, render_character_summaries, render_node,
    render_observable_world_state, render_optional_node, render_sections,
    render_state_schema_fields,
};
use state::{PlayerStateSchema, StateOp, StateUpdate, WorldState};
use story::NarrativeNode;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeeperPhase {
    AfterPlayerInput,
    AfterTurnOutputs,
}

#[derive(Debug, Clone, Copy)]
pub struct KeeperRequest<'a> {
    pub phase: KeeperPhase,
    pub player_input: &'a str,
    pub previous_node: Option<&'a NarrativeNode>,
    pub current_node: &'a NarrativeNode,
    pub character_cards: &'a [CharacterCard],
    pub current_cast_ids: &'a [String],
    pub player_name: Option<&'a str>,
    pub player_description: &'a str,
    pub player_state_schema: &'a PlayerStateSchema,
    pub world_state: &'a WorldState,
    pub completed_beats: &'a [KeeperBeat],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KeeperBeat {
    Narrator {
        purpose: NarratorPurpose,
        text: String,
    },
    Actor {
        speaker_id: String,
        purpose: ActorPurpose,
        visible_segments: Vec<KeeperActorSegment>,
    },
}

impl KeeperBeat {
    pub fn from_narrator_response(purpose: NarratorPurpose, response: &NarratorResponse) -> Self {
        Self::Narrator {
            purpose,
            text: response.text.clone(),
        }
    }

    pub fn from_actor_response(purpose: ActorPurpose, response: &ActorResponse) -> Self {
        let visible_segments = response
            .segments
            .iter()
            .filter_map(|segment| match segment.kind {
                ActorSegmentKind::Dialogue => Some(KeeperActorSegment {
                    kind: KeeperActorSegmentKind::Dialogue,
                    text: segment.text.clone(),
                }),
                ActorSegmentKind::Action => Some(KeeperActorSegment {
                    kind: KeeperActorSegmentKind::Action,
                    text: segment.text.clone(),
                }),
                ActorSegmentKind::Thought => None,
            })
            .collect();

        Self::Actor {
            speaker_id: response.speaker_id.clone(),
            purpose,
            visible_segments,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeeperActorSegmentKind {
    Dialogue,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeeperActorSegment {
    pub kind: KeeperActorSegmentKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperResponse {
    pub update: StateUpdate,
    pub output: llm::ChatResponse,
}

pub struct Keeper {
    llm: Arc<dyn LlmApi>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Keeper {
    pub fn new(llm: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, KeeperError> {
        Self::new_with_options(llm, model, None, None)
    }

    pub fn new_with_options(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Self, KeeperError> {
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt: include_str!("./prompts/keeper.txt").to_owned(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_prompt_file(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, KeeperError> {
        let system_prompt = fs::read_to_string(path).map_err(KeeperError::ReadPrompt)?;
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt,
            temperature: None,
            max_tokens: None,
        })
    }

    pub async fn keep(&self, request: KeeperRequest<'_>) -> Result<KeeperResponse, KeeperError> {
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
                    .response_format(llm::ResponseFormat::JsonObject);
                if let Some(temperature) = self.temperature {
                    builder = builder.temperature(temperature);
                }
                if let Some(max_tokens) = self.max_tokens {
                    builder = builder.max_tokens(max_tokens);
                }
                builder.build()?
            })
            .await?;

        let update: StateUpdate = output
            .structured_output
            .as_ref()
            .ok_or(KeeperError::MissingOutput)
            .and_then(|value| {
                serde_json::from_value(value.clone()).map_err(|error| {
                    error!(
                        error = %error,
                        structured_output = %structured_output_for_log(value),
                        "keeper returned invalid structured output"
                    );
                    KeeperError::InvalidJson(error)
                })
            })?;
        Self::validate_update(&update)?;

        Ok(KeeperResponse { update, output })
    }

    fn validate_request(request: &KeeperRequest<'_>) -> Result<(), KeeperError> {
        let cards_by_id: HashMap<&str, &CharacterCard> = request
            .character_cards
            .iter()
            .map(|card| (card.id.as_str(), card))
            .collect();

        for character_id in request.current_cast_ids {
            if !cards_by_id.contains_key(character_id.as_str()) {
                return Err(KeeperError::InvalidRequest(format!(
                    "missing character card for current node id '{character_id}'"
                )));
            }
        }

        if let Some(previous_node) = &request.previous_node {
            for character_id in &previous_node.characters {
                if !cards_by_id.contains_key(character_id.as_str()) {
                    return Err(KeeperError::InvalidRequest(format!(
                        "missing character card for previous node id '{character_id}'"
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_update(update: &StateUpdate) -> Result<(), KeeperError> {
        for op in &update.ops {
            match op {
                StateOp::SetState { .. }
                | StateOp::RemoveState { .. }
                | StateOp::SetPlayerState { .. }
                | StateOp::RemovePlayerState { .. }
                | StateOp::SetActiveCharacters { .. }
                | StateOp::AddActiveCharacter { .. }
                | StateOp::RemoveActiveCharacter { .. }
                | StateOp::SetCharacterState { .. }
                | StateOp::RemoveCharacterState { .. } => {}
                StateOp::SetCurrentNode { .. } => {
                    return Err(KeeperError::DisallowedOp(
                        "SetCurrentNode is reserved for Director/runtime graph transitions"
                            .to_owned(),
                    ));
                }
            }
        }

        Ok(())
    }

    fn build_user_prompts(
        &self,
        request: &KeeperRequest<'_>,
    ) -> Result<(String, String), KeeperError> {
        let stable_prompt = render_sections(&[
            ("PLAYER_DESCRIPTION", request.player_description.to_owned()),
            (
                "KEEPER_PHASE",
                compact_json(&request.phase).map_err(KeeperError::SerializePromptData)?,
            ),
            ("PREVIOUS_NODE", render_optional_node(request.previous_node)),
            (
                "PREVIOUS_CAST",
                self.previous_cast_summaries(request)?
                    .map(|summaries| render_character_summaries(&summaries, request.player_name))
                    .unwrap_or_else(|| "null".to_owned()),
            ),
            ("CURRENT_NODE", render_node(request.current_node)),
            (
                "CURRENT_CAST",
                render_character_summaries(
                    &self.current_cast_summaries(request)?,
                    request.player_name,
                ),
            ),
            (
                "PLAYER_STATE_SCHEMA",
                render_state_schema_fields(&request.player_state_schema.fields),
            ),
        ]);
        let dynamic_prompt = render_sections(&[
            ("PLAYER_INPUT", request.player_input.to_owned()),
            (
                "WORLD_STATE",
                render_observable_world_state(request.world_state),
            ),
            (
                "COMPLETED_BEATS",
                render_keeper_beats(request.completed_beats),
            ),
        ]);

        Ok((stable_prompt, dynamic_prompt))
    }

    fn current_cast_summaries<'b>(
        &self,
        request: &KeeperRequest<'b>,
    ) -> Result<Vec<CharacterCardSummaryRef<'b>>, KeeperError> {
        cast_summaries(request.current_cast_ids, request.character_cards)
    }

    fn previous_cast_summaries<'b>(
        &self,
        request: &KeeperRequest<'b>,
    ) -> Result<Option<Vec<CharacterCardSummaryRef<'b>>>, KeeperError> {
        request
            .previous_node
            .map(|node| cast_summaries(&node.characters, request.character_cards))
            .transpose()
    }
}

fn structured_output_for_log(value: &Value) -> String {
    const MAX_LOG_CHARS: usize = 4_000;

    match serde_json::to_string_pretty(value) {
        Ok(serialized) if serialized.chars().count() > MAX_LOG_CHARS => {
            let truncated: String = serialized.chars().take(MAX_LOG_CHARS).collect();
            format!(
                "{truncated}... [truncated {MAX_LOG_CHARS}/{} chars]",
                serialized.chars().count()
            )
        }
        Ok(serialized) => serialized,
        Err(error) => format!("<failed to serialize structured output for log: {error}>"),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KeeperError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error(transparent)]
    ReadPrompt(std::io::Error),
    #[error(transparent)]
    SerializePromptData(serde_json::Error),
    #[error(transparent)]
    InvalidJson(serde_json::Error),
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error("missing json output")]
    MissingOutput,
    #[error("disallowed state op: {0}")]
    DisallowedOp(String),
}

fn cast_summaries<'a>(
    character_ids: &[String],
    character_cards: &'a [CharacterCard],
) -> Result<Vec<CharacterCardSummaryRef<'a>>, KeeperError> {
    let cards_by_id: HashMap<&str, &CharacterCard> = character_cards
        .iter()
        .map(|card| (card.id.as_str(), card))
        .collect();

    character_ids
        .iter()
        .map(|character_id| {
            cards_by_id
                .get(character_id.as_str())
                .map(|card| card.summary_ref())
                .ok_or_else(|| {
                    KeeperError::InvalidRequest(format!(
                        "missing character card for cast id '{character_id}'"
                    ))
                })
        })
        .collect()
}

fn render_keeper_beats(beats: &[KeeperBeat]) -> String {
    if beats.is_empty() {
        return "- none".to_owned();
    }

    beats
        .iter()
        .map(|beat| match beat {
            KeeperBeat::Narrator { purpose, text } => format!(
                "- [narrator|{}] {}",
                compact_json(purpose).unwrap_or_default(),
                normalize_inline_text(text)
            ),
            KeeperBeat::Actor {
                speaker_id,
                purpose,
                visible_segments,
            } => {
                let segments = visible_segments
                    .iter()
                    .map(|segment| {
                        format!(
                            "{}:{}",
                            compact_json(&segment.kind).unwrap_or_default(),
                            normalize_inline_text(&segment.text)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(" | ");
                format!(
                    "- [actor|{}|{}] {}",
                    speaker_id,
                    compact_json(purpose).unwrap_or_default(),
                    segments
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
