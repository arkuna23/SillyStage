use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use crate::prompt::{
    compact_json, render_character_summaries, render_director_world_state, render_node,
    render_sections, render_state_schema_fields,
};
use llm::{ChatRequest, LlmApi};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};

use state::{PlayerStateSchema, WorldState};
use story::node::NarrativeNode;
use story::runtime_graph::RuntimeStoryGraph;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectorResult {
    pub previous_node_id: String,
    pub current_node_id: String,
    pub transitioned: bool,
    pub response_plan: ResponsePlan,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsePlan {
    #[serde(default)]
    pub role_actions: Vec<SessionCharacterAction>,
    #[serde(default)]
    pub beats: Vec<ResponseBeat>,
}

impl ResponsePlan {
    pub fn new() -> Self {
        Self {
            role_actions: Vec::new(),
            beats: Vec::new(),
        }
    }

    pub fn add_role_action(&mut self, action: SessionCharacterAction) {
        self.role_actions.push(action);
    }

    pub fn add_beat(&mut self, beat: ResponseBeat) {
        self.beats.push(beat);
    }

    pub fn is_empty(&self) -> bool {
        self.role_actions.is_empty() && self.beats.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SessionCharacterAction {
    CreateAndEnter {
        session_character_id: String,
        display_name: String,
        personality: String,
        style: String,
        system_prompt: String,
    },
    LeaveScene {
        session_character_id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResponseBeat {
    Narrator {
        purpose: NarratorPurpose,
    },
    Actor {
        speaker_id: String,
        purpose: ActorPurpose,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NarratorPurpose {
    DescribeTransition,
    DescribeScene,
    DescribeResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorPurpose {
    AdvanceGoal,
    ReactToPlayer,
    CommentOnScene,
}

pub struct Director {
    llm: Arc<dyn LlmApi>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Director {
    pub fn new(llm: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, DirectorError> {
        Self::new_with_options(llm, model, None, None)
    }

    pub fn new_with_options(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Self, DirectorError> {
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt: include_str!("./prompts/director.txt").to_owned(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_prompt_file(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, DirectorError> {
        let system_prompt = fs::read_to_string(path).map_err(DirectorError::ReadPrompt)?;
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt,
            temperature: None,
            max_tokens: None,
        })
    }

    pub async fn decide(
        &self,
        runtime_graph: &RuntimeStoryGraph,
        world_state: &mut WorldState,
        character_cards: &[CharacterCard],
        player_name: Option<&str>,
        player_description: &str,
        player_state_schema: &PlayerStateSchema,
    ) -> Result<DirectorResult, DirectorError> {
        let player_persona = PlayerPersona {
            name: player_name,
            description: player_description,
        };
        self.decide_internal(
            runtime_graph,
            world_state,
            character_cards,
            player_persona,
            player_state_schema,
            true,
        )
        .await
    }

    pub async fn decide_strict(
        &self,
        runtime_graph: &RuntimeStoryGraph,
        world_state: &mut WorldState,
        character_cards: &[CharacterCard],
        player_name: Option<&str>,
        player_description: &str,
        player_state_schema: &PlayerStateSchema,
    ) -> Result<DirectorResult, DirectorError> {
        let player_persona = PlayerPersona {
            name: player_name,
            description: player_description,
        };
        self.decide_internal(
            runtime_graph,
            world_state,
            character_cards,
            player_persona,
            player_state_schema,
            false,
        )
        .await
    }

    async fn decide_internal(
        &self,
        runtime_graph: &RuntimeStoryGraph,
        world_state: &mut WorldState,
        character_cards: &[CharacterCard],
        player_persona: PlayerPersona<'_>,
        player_state_schema: &PlayerStateSchema,
        allow_fallback: bool,
    ) -> Result<DirectorResult, DirectorError> {
        let previous_node_id = world_state.current_node.clone();

        let current_index = runtime_graph
            .get_node_index(&world_state.current_node)
            .ok_or_else(|| DirectorError::NodeNotFound(world_state.current_node.clone()))?;

        let mut next_index = current_index;
        let mut transitioned = false;

        for edge in runtime_graph.graph.edges(current_index) {
            let matched = match &edge.weight().condition {
                Some(cond) => self.evaluate_condition(world_state, cond),
                None => true,
            };

            if matched {
                next_index = edge.target();
                transitioned = next_index != current_index;
                break;
            }
        }

        if transitioned {
            let next_node = &runtime_graph.graph[next_index];
            world_state.set_current_node(next_node.id.clone());
            world_state.set_active_characters(next_node.characters.clone());

            for op in next_node.on_enter_updates() {
                world_state.apply_op(op.clone());
            }
        }

        let current_node = &runtime_graph.graph[next_index];

        let response_plan = if allow_fallback {
            self.build_llm_response_plan(
                world_state,
                current_node,
                transitioned,
                character_cards,
                player_persona,
                player_state_schema,
            )
            .await
            .unwrap_or_else(|_| self.build_fallback_response_plan(current_node, transitioned))
        } else {
            self.build_llm_response_plan(
                world_state,
                current_node,
                transitioned,
                character_cards,
                player_persona,
                player_state_schema,
            )
            .await?
        };

        Ok(DirectorResult {
            previous_node_id,
            current_node_id: current_node.id.clone(),
            transitioned,
            response_plan,
        })
    }

    async fn build_llm_response_plan(
        &self,
        world_state: &WorldState,
        node: &NarrativeNode,
        transitioned: bool,
        character_cards: &[CharacterCard],
        player_persona: PlayerPersona<'_>,
        player_state_schema: &PlayerStateSchema,
    ) -> Result<ResponsePlan, DirectorError> {
        let (stable_prompt, dynamic_prompt) = self.build_user_prompts(
            world_state,
            node,
            transitioned,
            character_cards,
            player_persona,
            player_state_schema,
        )?;

        let value = self
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
            .await?
            .structured_output
            .ok_or(DirectorError::MissingJson)?;
        serde_json::from_value(value).map_err(DirectorError::InvalidPlanJson)
    }

    fn build_user_prompts(
        &self,
        world_state: &WorldState,
        node: &NarrativeNode,
        transitioned: bool,
        character_cards: &[CharacterCard],
        player_persona: PlayerPersona<'_>,
        player_state_schema: &PlayerStateSchema,
    ) -> Result<(String, String), DirectorError> {
        let stable_prompt = render_sections(&[
            ("PLAYER_DESCRIPTION", player_persona.description.to_owned()),
            (
                "CURRENT_CAST",
                render_character_summaries(&current_cast_summaries(
                    world_state.active_characters(),
                    character_cards,
                )?, player_persona.name),
            ),
            ("CURRENT_NODE", render_node(node)),
            (
                "PLAYER_STATE_SCHEMA",
                render_state_schema_fields(&player_state_schema.fields),
            ),
            (
                "TRANSITIONED_THIS_TURN",
                compact_json(&transitioned).map_err(DirectorError::SerializePromptData)?,
            ),
        ]);
        let dynamic_prompt =
            render_sections(&[("WORLD_STATE", render_director_world_state(world_state))]);

        Ok((stable_prompt, dynamic_prompt))
    }

    fn build_fallback_response_plan(
        &self,
        node: &NarrativeNode,
        transitioned: bool,
    ) -> ResponsePlan {
        let mut plan = ResponsePlan::new();

        if transitioned || node.characters.is_empty() {
            plan.add_beat(ResponseBeat::Narrator {
                purpose: if transitioned {
                    NarratorPurpose::DescribeTransition
                } else {
                    NarratorPurpose::DescribeScene
                },
            });
        }

        for (idx, character) in node.characters.iter().enumerate() {
            plan.add_beat(ResponseBeat::Actor {
                speaker_id: character.clone(),
                purpose: if idx == 0 {
                    ActorPurpose::AdvanceGoal
                } else {
                    ActorPurpose::ReactToPlayer
                },
            });
        }

        plan
    }

    fn evaluate_condition(
        &self,
        world_state: &WorldState,
        condition: &story::condition::Condition,
    ) -> bool {
        condition.matches(world_state)
    }
}

#[derive(Debug, Clone, Copy)]
struct PlayerPersona<'a> {
    name: Option<&'a str>,
    description: &'a str,
}

#[derive(Debug, thiserror::Error)]
pub enum DirectorError {
    #[error("{0}")]
    NodeNotFound(String),
    #[error("{0}")]
    MissingCharacterCard(String),
    #[error(transparent)]
    ReadPrompt(std::io::Error),
    #[error(transparent)]
    SerializePromptData(serde_json::Error),
    #[error(transparent)]
    InvalidPlanJson(serde_json::Error),
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error("missing json output")]
    MissingJson,
}

fn current_cast_summaries<'a>(
    current_character_ids: &[String],
    character_cards: &'a [CharacterCard],
) -> Result<Vec<CharacterCardSummaryRef<'a>>, DirectorError> {
    let cards_by_id: std::collections::HashMap<&str, &CharacterCard> = character_cards
        .iter()
        .map(|card| (card.id.as_str(), card))
        .collect();

    current_character_ids
        .iter()
        .map(|character_id| {
            cards_by_id
                .get(character_id.as_str())
                .map(|card| card.summary_ref())
                .ok_or_else(|| {
                    DirectorError::MissingCharacterCard(format!(
                        "missing character card for current cast id '{character_id}'"
                    ))
                })
        })
        .collect()
}
