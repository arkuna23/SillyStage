use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
use futures_util::{StreamExt, stream};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use crate::director::NarratorPurpose;
use crate::prompt::{
    compact_json, render_character_summaries, render_observable_world_state, render_optional_node,
    render_sections, render_state_schema_fields,
};
use state::{PlayerStateSchema, WorldState};
use story::NarrativeNode;

pub type NarratorEventStream<'a> =
    Pin<Box<dyn Stream<Item = Result<NarratorStreamEvent, NarratorError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct NarratorRequest<'a> {
    pub purpose: NarratorPurpose,
    pub previous_node: Option<&'a NarrativeNode>,
    pub current_node: &'a NarrativeNode,
    pub character_cards: &'a [CharacterCard],
    pub player_name: Option<&'a str>,
    pub player_description: &'a str,
    pub player_state_schema: &'a PlayerStateSchema,
    pub world_state: &'a WorldState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NarratorResponse {
    pub text: String,
    pub raw_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NarratorStreamEvent {
    TextDelta { delta: String },
    Done { response: NarratorResponse },
}

pub struct Narrator {
    llm: Arc<dyn LlmApi>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Narrator {
    pub fn new(llm: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, NarratorError> {
        Self::new_with_options(llm, model, None, None)
    }

    pub fn new_with_options(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Self, NarratorError> {
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt: include_str!("./prompts/narrator.txt").to_owned(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_prompt_file(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, NarratorError> {
        let system_prompt = fs::read_to_string(path).map_err(NarratorError::ReadPrompt)?;
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt,
            temperature: None,
            max_tokens: None,
        })
    }

    pub async fn narrate(
        &self,
        request: NarratorRequest<'_>,
    ) -> Result<NarratorResponse, NarratorError> {
        let mut stream = self.narrate_stream(request).await?;
        let mut final_response = None;

        while let Some(event) = stream.next().await {
            if let NarratorStreamEvent::Done { response } = event? {
                final_response = Some(response);
            }
        }

        final_response.ok_or_else(|| {
            NarratorError::StreamComplete(
                "narrator stream finished without a final response".to_owned(),
            )
        })
    }

    pub async fn narrate_stream<'b>(
        &'b self,
        request: NarratorRequest<'_>,
    ) -> Result<NarratorEventStream<'b>, NarratorError> {
        Self::validate_request(&request)?;

        let (stable_prompt, dynamic_prompt) = self.build_user_prompts(&request)?;
        let stream = self
            .llm
            .chat_stream({
                let mut builder = ChatRequest::builder()
                    .model(&self.model)
                    .system_message(&self.system_prompt)
                    .user_message(stable_prompt)
                    .user_message(dynamic_prompt);
                if let Some(temperature) = self.temperature {
                    builder = builder.temperature(temperature);
                }
                if let Some(max_tokens) = self.max_tokens {
                    builder = builder.max_tokens(max_tokens);
                }
                builder.build()?
            })
            .await?;

        let state = NarratorEventStreamState {
            llm_stream: stream,
            text: String::new(),
            raw_output: String::new(),
            llm_finished: false,
            done_emitted: false,
            terminated: false,
        };

        let stream = stream::unfold(state, |mut state| async move {
            if state.terminated || state.done_emitted {
                return None;
            }

            loop {
                if state.llm_finished {
                    if state.text.trim().is_empty() {
                        state.terminated = true;
                        return Some((
                            Err(NarratorError::StreamComplete(
                                "narrator output contained no text".to_owned(),
                            )),
                            state,
                        ));
                    }

                    state.done_emitted = true;
                    return Some((
                        Ok(NarratorStreamEvent::Done {
                            response: NarratorResponse {
                                text: state.text.clone(),
                                raw_output: state.raw_output.clone(),
                            },
                        }),
                        state,
                    ));
                }

                match state.llm_stream.next().await {
                    Some(Ok(chunk)) => {
                        if !chunk.delta.is_empty() {
                            state.text.push_str(&chunk.delta);
                            state.raw_output.push_str(&chunk.delta);
                            return Some((
                                Ok(NarratorStreamEvent::TextDelta { delta: chunk.delta }),
                                state,
                            ));
                        }

                        if chunk.done {
                            state.llm_finished = true;
                        }
                    }
                    Some(Err(error)) => {
                        state.terminated = true;
                        return Some((Err(NarratorError::Llm(error)), state));
                    }
                    None => {
                        state.llm_finished = true;
                    }
                }
            }
        });

        Ok(Box::pin(stream))
    }

    fn validate_request(request: &NarratorRequest<'_>) -> Result<(), NarratorError> {
        if matches!(request.purpose, NarratorPurpose::DescribeTransition)
            && request.previous_node.is_none()
        {
            return Err(NarratorError::InvalidRequest(
                "DescribeTransition requires previous_node".to_owned(),
            ));
        }

        let cards_by_id: HashMap<&str, &CharacterCard> = request
            .character_cards
            .iter()
            .map(|card| (card.id.as_str(), card))
            .collect();

        for character_id in &request.current_node.characters {
            if !cards_by_id.contains_key(character_id.as_str()) {
                return Err(NarratorError::InvalidRequest(format!(
                    "missing character card for current node id '{character_id}'"
                )));
            }
        }

        if let Some(previous_node) = &request.previous_node {
            for character_id in &previous_node.characters {
                if !cards_by_id.contains_key(character_id.as_str()) {
                    return Err(NarratorError::InvalidRequest(format!(
                        "missing character card for previous node id '{character_id}'"
                    )));
                }
            }
        }

        Ok(())
    }

    fn build_user_prompts(
        &self,
        request: &NarratorRequest<'_>,
    ) -> Result<(String, String), NarratorError> {
        let stable_prompt = render_sections(&[
            (
                "NARRATOR_PURPOSE",
                compact_json(&request.purpose).map_err(NarratorError::SerializePromptData)?,
            ),
            (
                "PLAYER_NAME",
                request.player_name.unwrap_or("null").to_owned(),
            ),
            ("PLAYER_DESCRIPTION", request.player_description.to_owned()),
            ("PREVIOUS_NODE", render_optional_node(request.previous_node)),
            (
                "PREVIOUS_CAST",
                self.previous_cast_summaries(request)?
                    .map(|summaries| render_character_summaries(&summaries))
                    .unwrap_or_else(|| "null".to_owned()),
            ),
            (
                "CURRENT_NODE",
                crate::prompt::render_node(request.current_node),
            ),
            (
                "CURRENT_CAST",
                render_character_summaries(&self.current_cast_summaries(request)?),
            ),
            (
                "PLAYER_STATE_SCHEMA",
                render_state_schema_fields(&request.player_state_schema.fields),
            ),
        ]);
        let dynamic_prompt = render_sections(&[(
            "WORLD_STATE",
            render_observable_world_state(&filtered_narrator_world_state(request.world_state)?),
        )]);

        Ok((stable_prompt, dynamic_prompt))
    }

    fn current_cast_summaries<'b>(
        &self,
        request: &NarratorRequest<'b>,
    ) -> Result<Vec<CharacterCardSummaryRef<'b>>, NarratorError> {
        cast_summaries(&request.current_node.characters, request.character_cards)
    }

    fn previous_cast_summaries<'b>(
        &self,
        request: &NarratorRequest<'b>,
    ) -> Result<Option<Vec<CharacterCardSummaryRef<'b>>>, NarratorError> {
        request
            .previous_node
            .map(|node| cast_summaries(&node.characters, request.character_cards))
            .transpose()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum NarratorError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error(transparent)]
    ReadPrompt(std::io::Error),
    #[error(transparent)]
    SerializePromptData(serde_json::Error),
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error("stream error: {0}")]
    StreamComplete(String),
}

struct NarratorEventStreamState {
    llm_stream: llm::ChatStream,
    text: String,
    raw_output: String,
    llm_finished: bool,
    done_emitted: bool,
    terminated: bool,
}

fn cast_summaries<'a>(
    character_ids: &[String],
    character_cards: &'a [CharacterCard],
) -> Result<Vec<CharacterCardSummaryRef<'a>>, NarratorError> {
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
                    NarratorError::InvalidRequest(format!(
                        "missing character card for cast id '{character_id}'"
                    ))
                })
        })
        .collect()
}

fn filtered_narrator_world_state(world_state: &WorldState) -> Result<WorldState, NarratorError> {
    let mut clone = world_state.clone();
    clone.actor_shared_history.retain(|entry| entry.speaker_id != "narrator");
    Ok(clone)
}
