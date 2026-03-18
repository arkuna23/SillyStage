use std::collections::HashMap;
use std::sync::Arc;

use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

use crate::actor::{ActorResponse, ActorSegmentKind, CharacterCard, CharacterCardSummaryRef};
use crate::director::{ActorPurpose, NarratorPurpose};
use crate::narrator::NarratorResponse;
use crate::prompt::{
    PromptProfile, compact_json, normalize_inline_text, render_character_summaries,
    render_keeper_node, render_observable_world_state, render_player, render_prompt_entries,
    render_sections, render_state_schema_fields,
};
use state::{PlayerStateSchema, StateOp, StateUpdate, WorldState};
use story::{Condition, ConditionScope, NarrativeNode};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeeperPhase {
    AfterPlayerInput,
    AfterTurnOutputs,
}

#[derive(Debug, Clone)]
pub struct KeeperRequest<'a> {
    pub phase: KeeperPhase,
    pub player_input: &'a str,
    pub previous_node: Option<&'a NarrativeNode>,
    pub current_node: &'a NarrativeNode,
    pub character_cards: &'a [CharacterCard],
    pub current_cast_ids: &'a [String],
    pub lorebook_base: Option<String>,
    pub lorebook_matched: Option<String>,
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
    prompt_profile: PromptProfile,
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
            prompt_profile: PromptProfile::default(),
            temperature,
            max_tokens,
        })
    }

    pub fn with_prompt_profile(mut self, prompt_profile: PromptProfile) -> Self {
        self.prompt_profile = prompt_profile;
        self
    }

    pub async fn keep(&self, request: KeeperRequest<'_>) -> Result<KeeperResponse, KeeperError> {
        Self::validate_request(&request)?;

        let (stable_prompt, dynamic_prompt) = self.build_user_prompts(&request)?;
        let output = self
            .llm
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(&self.model)
                    .system_message(&self.prompt_profile.system_prompt)
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
        let keeper_phase =
            compact_json(&request.phase).map_err(KeeperError::SerializePromptData)?;
        let previous_cast = self
            .previous_cast_summaries(request)?
            .map(|summaries| render_character_summaries(&summaries, request.player_name))
            .unwrap_or_else(|| "null".to_owned());
        let current_cast =
            render_character_summaries(&self.current_cast_summaries(request)?, request.player_name);

        let stable_prompt =
            render_prompt_entries(&self.prompt_profile.stable_entries, |key| match key {
                "lorebook_base" => request.lorebook_base.as_deref().map(str::to_owned),
                "player" => Some(render_player(
                    request.player_name,
                    request.player_description,
                )),
                "keeper_phase" => Some(keeper_phase.clone()),
                "previous_node" => Some(
                    request
                        .previous_node
                        .map(render_keeper_node)
                        .unwrap_or_else(|| "null".to_owned()),
                ),
                "node_change" => Some(render_keeper_node_change(
                    request.previous_node,
                    request.current_node,
                )),
                "previous_cast" => Some(previous_cast.clone()),
                "current_node" => Some(render_keeper_node(request.current_node)),
                "progression_hints" => Some(render_keeper_progression_hints(request.current_node)),
                "current_cast" => Some(current_cast.clone()),
                "player_state_schema" => Some(render_state_schema_fields(
                    &request.player_state_schema.fields,
                )),
                _ => None,
            });

        let dynamic_prompt =
            render_prompt_entries(&self.prompt_profile.dynamic_entries, |key| match key {
                "player_input" => Some(request.player_input.to_owned()),
                "world_state" => Some(render_observable_world_state(request.world_state)),
                "completed_beats" => Some(render_keeper_beats(request.completed_beats)),
                "lorebook_matched" => request.lorebook_matched.as_deref().map(str::to_owned),
                _ => None,
            });

        Ok((stable_prompt, dynamic_prompt))
    }

    fn current_cast_summaries<'b>(
        &self,
        request: &KeeperRequest<'b>,
    ) -> Result<Vec<CharacterCardSummaryRef<'b>>, KeeperError> {
        cast_summaries(
            request.current_cast_ids,
            request.character_cards,
            request.world_state,
        )
    }

    fn previous_cast_summaries<'b>(
        &self,
        request: &KeeperRequest<'b>,
    ) -> Result<Option<Vec<CharacterCardSummaryRef<'b>>>, KeeperError> {
        request
            .previous_node
            .map(|node| {
                cast_summaries(
                    &node.characters,
                    request.character_cards,
                    request.world_state,
                )
            })
            .transpose()
    }
}

fn render_keeper_node_change(
    previous_node: Option<&NarrativeNode>,
    current_node: &NarrativeNode,
) -> String {
    let Some(previous_node) = previous_node else {
        return "null".to_owned();
    };

    let transitioned = previous_node.id != current_node.id;
    let matched_transition_lines = previous_node
        .transitions
        .iter()
        .filter(|transition| transition.to == current_node.id)
        .map(render_keeper_progression_hint_line)
        .collect::<Vec<_>>();

    render_sections(&[
        ("transitioned", compact_bool(transitioned)),
        ("from", previous_node.id.clone()),
        ("to", current_node.id.clone()),
        (
            "matched_transition_hints",
            if matched_transition_lines.is_empty() {
                "- none".to_owned()
            } else {
                matched_transition_lines
                    .into_iter()
                    .map(|line| format!("- {line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
        ),
    ])
}

fn render_keeper_progression_hints(node: &NarrativeNode) -> String {
    if node.transitions.is_empty() {
        return "- none".to_owned();
    }

    node.transitions
        .iter()
        .map(render_keeper_progression_hint_line)
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_keeper_progression_hint_line(transition: &story::Transition) -> String {
    match &transition.condition {
        Some(condition) => {
            let (scope, key, character) = keeper_condition_tracking_hint(condition);
            match character {
                Some(character) => format!(
                    "target_node={} | condition={} | likely_state_scope={} | tracked_key={} | tracked_character={}",
                    transition.to,
                    render_condition_for_keeper(condition),
                    scope,
                    key,
                    character
                ),
                None => format!(
                    "target_node={} | condition={} | likely_state_scope={} | tracked_key={}",
                    transition.to,
                    render_condition_for_keeper(condition),
                    scope,
                    key
                ),
            }
        }
        None => format!(
            "target_node={} | condition=always | likely_state_scope=none | tracked_key=none",
            transition.to
        ),
    }
}

fn keeper_condition_tracking_hint(condition: &Condition) -> (&str, &str, Option<&str>) {
    match condition.scope {
        ConditionScope::Global => ("global", condition.key.as_str(), None),
        ConditionScope::Player => ("player", condition.key.as_str(), None),
        ConditionScope::Character => (
            "character",
            condition.key.as_str(),
            condition.character.as_deref(),
        ),
    }
}

fn render_condition_for_keeper(condition: &Condition) -> String {
    let left = match condition.scope {
        ConditionScope::Global => format!("global.{}", condition.key),
        ConditionScope::Player => format!("player.{}", condition.key),
        ConditionScope::Character => format!(
            "character[{}].{}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key
        ),
    };

    format!(
        "{left} {} {}",
        keeper_operator_symbol(&condition.op),
        condition.value
    )
}

fn keeper_operator_symbol(operator: &story::ConditionOperator) -> &'static str {
    match operator {
        story::ConditionOperator::Eq => "==",
        story::ConditionOperator::Ne => "!=",
        story::ConditionOperator::Gt => ">",
        story::ConditionOperator::Gte => ">=",
        story::ConditionOperator::Lt => "<",
        story::ConditionOperator::Lte => "<=",
        story::ConditionOperator::Contains => "contains",
    }
}

fn compact_bool(value: bool) -> String {
    if value {
        "true".to_owned()
    } else {
        "false".to_owned()
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
    world_state: &'a WorldState,
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
                .map(|card| card.summary_ref(world_state.character_states(character_id)))
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
