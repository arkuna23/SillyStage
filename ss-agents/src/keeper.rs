use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::actor::{ActorResponse, ActorSegmentKind, CharacterCard, CharacterCardSummaryRef};
use crate::director::{ActorPurpose, NarratorPurpose};
use crate::narrator::NarratorResponse;
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
}

impl Keeper {
    pub fn new(llm: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, KeeperError> {
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt: include_str!("./prompts/keeper.txt").to_owned(),
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
        })
    }

    pub async fn keep(&self, request: KeeperRequest<'_>) -> Result<KeeperResponse, KeeperError> {
        Self::validate_request(&request)?;

        let user_prompt = self.build_user_prompt(&request)?;
        let output = self
            .llm
            .chat(
                ChatRequest::builder()
                    .model(&self.model)
                    .system_message(&self.system_prompt)
                    .user_message(user_prompt)
                    .response_format(llm::ResponseFormat::JsonObject)
                    .build()?,
            )
            .await?;

        let update: StateUpdate = output
            .structured_output
            .as_ref()
            .ok_or(KeeperError::MissingOutput)
            .and_then(|value| {
                serde_json::from_value(value.clone()).map_err(KeeperError::InvalidJson)
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

        for character_id in &request.current_node.characters {
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

    fn build_user_prompt(&self, request: &KeeperRequest<'_>) -> Result<String, KeeperError> {
        let phase_json =
            serde_json::to_string(&request.phase).map_err(KeeperError::SerializePromptData)?;
        let player_input_json = serde_json::to_string_pretty(&request.player_input)
            .map_err(KeeperError::SerializePromptData)?;
        let previous_node_json =
            serialize_optional(&request.previous_node).map_err(KeeperError::SerializePromptData)?;
        let previous_cast_json = serialize_optional(&self.previous_cast_summaries(request)?)
            .map_err(KeeperError::SerializePromptData)?;
        let current_node_json = serde_json::to_string_pretty(&request.current_node)
            .map_err(KeeperError::SerializePromptData)?;
        let current_cast_json =
            serde_json::to_string_pretty(&self.current_cast_summaries(request)?)
                .map_err(KeeperError::SerializePromptData)?;
        let player_state_schema_json = serde_json::to_string_pretty(request.player_state_schema)
            .map_err(KeeperError::SerializePromptData)?;
        let world_state_json =
            serde_json::to_string_pretty(&request.world_state.observable_prompt_view())
                .map_err(KeeperError::SerializePromptData)?;
        let completed_beats_json = serde_json::to_string_pretty(&request.completed_beats)
            .map_err(KeeperError::SerializePromptData)?;

        Ok(format!(
            "KEEPER_PHASE:\n{}\n\nPLAYER_INPUT:\n{}\n\nPLAYER_DESCRIPTION:\n{}\n\nPREVIOUS_NODE:\n{}\n\nPREVIOUS_CAST:\n{}\n\nCURRENT_NODE:\n{}\n\nCURRENT_CAST:\n{}\n\nPLAYER_STATE_SCHEMA:\n{}\n\nWORLD_STATE:\n{}\n\nCOMPLETED_BEATS:\n{}",
            phase_json,
            player_input_json,
            request.player_description,
            previous_node_json,
            previous_cast_json,
            current_node_json,
            current_cast_json,
            player_state_schema_json,
            world_state_json,
            completed_beats_json
        ))
    }

    fn current_cast_summaries<'b>(
        &self,
        request: &KeeperRequest<'b>,
    ) -> Result<Vec<CharacterCardSummaryRef<'b>>, KeeperError> {
        cast_summaries(&request.current_node.characters, request.character_cards)
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

fn serialize_optional<T>(value: &Option<T>) -> Result<String, serde_json::Error>
where
    T: Serialize,
{
    serde_json::to_string_pretty(&value.as_ref().map_or(Value::Null, |value| {
        serde_json::to_value(value).unwrap_or(Value::Null)
    }))
}
