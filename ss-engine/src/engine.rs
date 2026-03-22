use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agents::actor::{
    Actor, ActorError, ActorRequest, ActorResponse, ActorStreamEvent, CharacterCard,
};
use agents::architect::{Architect, ArchitectError, ArchitectRequest, ArchitectResponse};
use agents::director::{
    ActorPurpose, Director, DirectorError, DirectorResult, NarratorPurpose, ResponseBeat,
    SessionCharacterAction,
};
use agents::keeper::{Keeper, KeeperBeat, KeeperError, KeeperPhase, KeeperRequest, KeeperResponse};
use agents::narrator::{
    Narrator, NarratorError, NarratorRequest, NarratorResponse, NarratorStreamEvent,
};
use agents::planner::{Planner, PlannerError, PlannerRequest, PlannerResponse};
use agents::{ArchitectPromptProfiles, PromptProfile};
use async_stream::stream;
use futures_core::Stream;
use futures_util::StreamExt;
use llm::LlmApi;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::{ActorMemoryEntry, ActorMemoryKind, StateOp, StateUpdate, WorldState};
use store::SessionCharacterRecord;
use story::{Condition, ConditionOperator, ConditionScope};
use tracing::{debug, info};

use crate::RuntimeSnapshot;
use crate::event::{EngineEvent, EngineStage};
use crate::history::DEFAULT_MESSAGE_HISTORY_LIMIT;
use crate::logging::{
    json_for_log, summarize_actor_response, summarize_architect_response,
    summarize_director_result, summarize_keeper_response, summarize_narrator_response,
    summarize_planner_response,
};
use crate::lorebook::{LorebookPromptSections, build_lorebook_prompt_sections};
use crate::runtime::{RuntimeError, RuntimeState, StoryResources};

const DEFAULT_ARCHITECT_GENERATE_MAX_TOKENS: u32 = 8_192;
const DEFAULT_ARCHITECT_GENERATE_TEMPERATURE: f32 = 0.0;

pub type EngineTurnStream<'a> = Pin<Box<dyn Stream<Item = EngineEvent> + Send + 'a>>;

#[derive(Clone)]
pub struct AgentModelConfig {
    pub client: Arc<dyn LlmApi>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub shared_history_limit: Option<usize>,
    pub private_memory_limit: Option<usize>,
    pub session_history_limit: Option<usize>,
    pub prompt_profile: PromptProfile,
}

impl AgentModelConfig {
    pub fn new(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
            temperature: None,
            max_tokens: None,
            shared_history_limit: None,
            private_memory_limit: None,
            session_history_limit: None,
            prompt_profile: PromptProfile::default(),
        }
    }

    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_shared_history_limit(mut self, limit: Option<usize>) -> Self {
        self.shared_history_limit = limit;
        self
    }

    pub fn with_private_memory_limit(mut self, limit: Option<usize>) -> Self {
        self.private_memory_limit = limit;
        self
    }

    pub fn with_session_history_limit(mut self, limit: Option<usize>) -> Self {
        self.session_history_limit = limit;
        self
    }

    pub fn with_prompt_profile(mut self, prompt_profile: PromptProfile) -> Self {
        self.prompt_profile = prompt_profile;
        self
    }
}

#[derive(Clone)]
pub struct ArchitectModelConfig {
    pub client: Arc<dyn LlmApi>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub prompt_profiles: ArchitectPromptProfiles,
}

impl ArchitectModelConfig {
    pub fn new(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
            temperature: None,
            max_tokens: None,
            prompt_profiles: ArchitectPromptProfiles {
                graph: PromptProfile::default(),
                draft_init: PromptProfile::default(),
                draft_continue: PromptProfile::default(),
                repair_system_prompt: String::new(),
            },
        }
    }

    pub fn with_temperature(mut self, temperature: Option<f32>) -> Self {
        self.temperature = temperature;
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: Option<u32>) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_prompt_profiles(mut self, prompt_profiles: ArchitectPromptProfiles) -> Self {
        self.prompt_profiles = prompt_profiles;
        self
    }
}

#[derive(Clone)]
pub struct StoryGenerationAgentConfigs {
    pub planner: AgentModelConfig,
    pub architect: ArchitectModelConfig,
}

impl StoryGenerationAgentConfigs {
    pub fn shared(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        let model = model.into();

        Self {
            planner: AgentModelConfig::new(Arc::clone(&client), model.clone()),
            architect: ArchitectModelConfig::new(client, model).with_max_tokens(Some(8_192)),
        }
    }
}

#[derive(Clone)]
pub struct RuntimeAgentConfigs {
    pub director: AgentModelConfig,
    pub actor: AgentModelConfig,
    pub narrator: AgentModelConfig,
    pub keeper: AgentModelConfig,
    pub shared_memory_limit: usize,
}

impl RuntimeAgentConfigs {
    pub fn shared(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        let model = model.into();

        Self {
            director: AgentModelConfig::new(Arc::clone(&client), model.clone()),
            actor: AgentModelConfig::new(Arc::clone(&client), model.clone()),
            narrator: AgentModelConfig::new(Arc::clone(&client), model.clone()),
            keeper: AgentModelConfig::new(client, model),
            shared_memory_limit: DEFAULT_MESSAGE_HISTORY_LIMIT,
        }
    }
}

pub struct Engine {
    runtime_state: RuntimeState,
    director: Director,
    actor: Actor,
    narrator: Narrator,
    keeper: Keeper,
    shared_memory_limit: usize,
}

impl Engine {
    pub fn new(
        agent_configs: RuntimeAgentConfigs,
        runtime_state: RuntimeState,
    ) -> Result<Self, EngineError> {
        let director_shared_history_limit = agent_configs
            .director
            .shared_history_limit
            .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT);
        let actor_shared_history_limit = agent_configs
            .actor
            .shared_history_limit
            .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT);
        let actor_private_memory_limit = agent_configs
            .actor
            .private_memory_limit
            .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT);
        let narrator_shared_history_limit = agent_configs
            .narrator
            .shared_history_limit
            .unwrap_or(DEFAULT_MESSAGE_HISTORY_LIMIT);

        Ok(Self {
            runtime_state,
            director: Director::new_with_options(
                agent_configs.director.client,
                agent_configs.director.model,
                agent_configs.director.temperature,
                agent_configs.director.max_tokens,
            )?
            .with_shared_history_limit(director_shared_history_limit)
            .with_prompt_profile(agent_configs.director.prompt_profile.clone()),
            actor: Actor::new_with_options(
                agent_configs.actor.client,
                agent_configs.actor.model,
                agent_configs.actor.temperature,
                agent_configs.actor.max_tokens,
            )?
            .with_shared_history_limit(actor_shared_history_limit)
            .with_private_memory_limit(actor_private_memory_limit)
            .with_prompt_profile(agent_configs.actor.prompt_profile.clone()),
            narrator: Narrator::new_with_options(
                agent_configs.narrator.client,
                agent_configs.narrator.model,
                agent_configs.narrator.temperature,
                agent_configs.narrator.max_tokens,
            )?
            .with_shared_history_limit(narrator_shared_history_limit)
            .with_prompt_profile(agent_configs.narrator.prompt_profile.clone()),
            keeper: Keeper::new_with_options(
                agent_configs.keeper.client,
                agent_configs.keeper.model,
                agent_configs.keeper.temperature,
                agent_configs.keeper.max_tokens,
            )?
            .with_prompt_profile(agent_configs.keeper.prompt_profile.clone()),
            shared_memory_limit: agent_configs.shared_memory_limit,
        })
    }

    pub fn runtime_state(&self) -> &RuntimeState {
        &self.runtime_state
    }

    pub fn runtime_state_mut(&mut self) -> &mut RuntimeState {
        &mut self.runtime_state
    }

    pub async fn run_turn(&mut self, player_input: &str) -> Result<EngineTurnResult, EngineError> {
        let mut stream = self.run_turn_stream(player_input).await?;

        while let Some(event) = stream.next().await {
            match event {
                EngineEvent::TurnCompleted { result } => return Ok(*result),
                EngineEvent::TurnFailed { stage, error, .. } => {
                    return Err(EngineError::TurnFailed {
                        stage,
                        message: error,
                    });
                }
                _ => {}
            }
        }

        Err(EngineError::IncompleteTurn)
    }

    pub async fn run_turn_stream<'b>(
        &'b mut self,
        player_input: &str,
    ) -> Result<EngineTurnStream<'b>, EngineError> {
        let player_input = player_input.to_owned();

        let stream = stream! {
            yield EngineEvent::TurnStarted {
                next_turn_index: self.runtime_state.turn_index().saturating_add(1),
                player_input: player_input.clone(),
            };

            let recorded_entry = record_player_input(
                self.runtime_state.world_state_mut(),
                &player_input,
                self.shared_memory_limit,
            );
            yield EngineEvent::PlayerInputRecorded {
                entry: recorded_entry,
                snapshot: Box::new(self.runtime_state.snapshot()),
            };

            let first_keeper = match self.run_first_keeper(&player_input).await {
                Ok(response) => response,
                Err(error) => {
                    yield self.turn_failed_event(EngineStage::KeeperAfterPlayerInput, error);
                    return;
                }
            };

            info!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index().saturating_add(1),
                summary = %json_for_log(&summarize_keeper_response(KeeperPhase::AfterPlayerInput, &first_keeper)),
                "keeper produced after-player-input update"
            );
            debug!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index().saturating_add(1),
                payload = %json_for_log(&first_keeper),
                "keeper after-player-input payload"
            );

            self.runtime_state
                .world_state_mut()
                .apply_update(first_keeper.update.clone());
            self.runtime_state.advance_turn();

            yield EngineEvent::KeeperApplied {
                phase: KeeperPhase::AfterPlayerInput,
                update: first_keeper.update.clone(),
                snapshot: Box::new(self.runtime_state.snapshot()),
            };

            let director_result = {
                let current_node_id = self.runtime_state.world_state().current_node().to_owned();
                let lorebook_sections =
                    self.runtime_lorebook_sections(&current_node_id, &player_input);
                let parts = self.runtime_state.engine_parts();
                self.director
                    .decide_strict(
                        parts.runtime_graph,
                        parts.world_state,
                        parts.character_cards,
                        lorebook_sections.base.as_deref(),
                        lorebook_sections.matched.as_deref(),
                        parts.player_name,
                        parts.player_description,
                        parts.player_state_schema,
                    )
                    .await
            };
            let director_result = match director_result {
                Ok(result) => result,
                Err(error) => {
                    yield self.turn_failed_event(EngineStage::Director, error);
                    return;
                }
            };

            info!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                summary = %json_for_log(&summarize_director_result(&director_result)),
                "director produced response plan"
            );
            debug!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                payload = %json_for_log(&director_result),
                "director response payload"
            );

            yield EngineEvent::DirectorCompleted {
                result: director_result.clone(),
                snapshot: Box::new(self.runtime_state.snapshot()),
            };

            for action in &director_result.response_plan.role_actions {
                match self.apply_session_character_action(action) {
                    Ok(SessionCharacterActionOutcome::CreatedAndEntered { character }) => {
                        yield EngineEvent::SessionCharacterCreated {
                            character,
                            snapshot: Box::new(self.runtime_state.snapshot()),
                        };
                        yield EngineEvent::SessionCharacterEnteredScene {
                            session_character_id: action_session_character_id(action).to_owned(),
                            snapshot: Box::new(self.runtime_state.snapshot()),
                        };
                    }
                    Ok(SessionCharacterActionOutcome::EnteredScene { session_character_id }) => {
                        yield EngineEvent::SessionCharacterEnteredScene {
                            session_character_id,
                            snapshot: Box::new(self.runtime_state.snapshot()),
                        };
                    }
                    Ok(SessionCharacterActionOutcome::LeftScene { session_character_id }) => {
                        yield EngineEvent::SessionCharacterLeftScene {
                            session_character_id,
                            snapshot: Box::new(self.runtime_state.snapshot()),
                        };
                    }
                    Ok(SessionCharacterActionOutcome::Noop) => {}
                    Err(error) => {
                        yield self.turn_failed_event(EngineStage::Director, error);
                        return;
                    }
                }
            }

            let mut completed_beats = Vec::new();

            for (beat_index, beat) in director_result.response_plan.beats.iter().enumerate() {
                match beat {
                    ResponseBeat::Narrator { purpose } => {
                        let purpose = purpose.clone();
                        yield EngineEvent::NarratorStarted { beat_index, purpose: purpose.clone() };

                        let narrator_request = match self.build_narrator_request(&director_result, purpose.clone(), &player_input) {
                            Ok(request) => request,
                            Err(error) => {
                                yield self.turn_failed_event(EngineStage::Narrator, error);
                                return;
                            }
                        };

                        let mut narrator_stream = match self.narrator.narrate_stream(narrator_request).await {
                            Ok(stream) => stream,
                            Err(error) => {
                                yield self.turn_failed_event(EngineStage::Narrator, error);
                                return;
                            }
                        };

                        let mut final_response = None;

                        while let Some(event) = narrator_stream.next().await {
                            match event {
                                Ok(NarratorStreamEvent::TextDelta { delta }) => {
                                    yield EngineEvent::NarratorTextDelta {
                                        beat_index,
                                        purpose: purpose.clone(),
                                        delta,
                                    };
                                }
                                Ok(NarratorStreamEvent::Done { response }) => {
                                    yield EngineEvent::NarratorCompleted {
                                        beat_index,
                                        purpose: purpose.clone(),
                                        response: Box::new(response.clone()),
                                    };
                                    final_response = Some(response);
                                }
                                Err(error) => {
                                    drop(narrator_stream);
                                    yield self.turn_failed_event(EngineStage::Narrator, error);
                                    return;
                                }
                            }
                        }

                        let Some(response) = final_response else {
                            yield self.turn_failed_event(
                                EngineStage::Narrator,
                                EngineError::MissingFinalBeat("narrator stream finished without Done".to_owned()),
                            );
                            return;
                        };

                        info!(
                            story_id = %self.runtime_state.story_id(),
                            turn_index = self.runtime_state.turn_index(),
                            beat_index,
                            purpose = ?purpose,
                            summary = %json_for_log(&summarize_narrator_response(&response)),
                            "narrator completed beat"
                        );
                        debug!(
                            story_id = %self.runtime_state.story_id(),
                            turn_index = self.runtime_state.turn_index(),
                            beat_index,
                            purpose = ?purpose,
                            payload = %json_for_log(&response),
                            "narrator response payload"
                        );

                        record_narration(
                            self.runtime_state.world_state_mut(),
                            &response,
                            self.shared_memory_limit,
                        );
                        completed_beats.push(ExecutedBeat::Narrator { purpose, response });
                    }
                    ResponseBeat::Actor { speaker_id, purpose } => {
                        let speaker_id = speaker_id.clone();
                        let purpose = purpose.clone();
                        let actor_stage_snapshot = self.runtime_state.snapshot();

                        yield EngineEvent::ActorStarted {
                            beat_index,
                            speaker_id: speaker_id.clone(),
                            purpose: purpose.clone(),
                        };

                        let mut actor_stream = {
                            let current_node_id =
                                self.runtime_state.world_state().current_node().to_owned();
                            let lorebook_sections =
                                self.runtime_lorebook_sections(&current_node_id, &player_input);
                            let actor_stream_result = {
                                let actor = &self.actor;
                                let parts = self.runtime_state.engine_parts();
                                let current_node_id = parts.world_state.current_node().to_owned();
                                let current_node_index = parts
                                    .runtime_graph
                                    .get_node_index(&current_node_id)
                                    .ok_or_else(|| {
                                        EngineError::Runtime(RuntimeError::MissingCurrentNode(
                                            current_node_id.clone(),
                                        ))
                                    });

                                match current_node_index {
                                    Ok(current_node_index) => {
                                        let current_node = parts
                                            .runtime_graph
                                            .graph
                                            .node_weight(current_node_index)
                                            .ok_or_else(|| {
                                                EngineError::Runtime(
                                                    RuntimeError::MissingCurrentNode(
                                                        current_node_id.clone(),
                                                    ),
                                                )
                                            });
                                        match current_node {
                                            Ok(current_node) => {
                                                let current_cast_ids =
                                                    parts.world_state.active_characters().to_vec();
                                                if !parts
                                                    .world_state
                                                    .active_characters()
                                                    .iter()
                                                    .any(|id| id == &speaker_id)
                                                {
                                                    Err(EngineError::InvalidBeatSpeaker {
                                                        speaker_id: speaker_id.clone(),
                                                        node_id: current_node.id.clone(),
                                                    })
                                                } else {
                                                    match parts
                                                        .character_cards
                                                        .iter()
                                                        .find(|card| card.id == speaker_id)
                                                    {
                                                        Some(character) => actor
                                                            .perform_stream(
                                                                ActorRequest {
                                                                    character,
                                                                    cast: parts.character_cards,
                                                                    current_cast_ids: &current_cast_ids,
                                                                    lorebook_base: lorebook_sections.base.clone(),
                                                                    lorebook_matched: lorebook_sections.matched.clone(),
                                                                    player_name: parts.player_name,
                                                                    player_description: parts.player_description,
                                                                    purpose: purpose.clone(),
                                                                    node: current_node,
                                                                },
                                                                parts.world_state,
                                                            )
                                                            .await
                                                            .map_err(EngineError::from),
                                                        None => Err(EngineError::InvalidBeatSpeaker {
                                                            speaker_id: speaker_id.clone(),
                                                            node_id: current_node.id.clone(),
                                                        }),
                                                    }
                                                }
                                            }
                                            Err(error) => Err(error),
                                        }
                                    }
                                    Err(error) => Err(error),
                                }
                            };

                            match actor_stream_result {
                                Ok(stream) => stream,
                                Err(error) => {
                                    yield EngineEvent::TurnFailed {
                                        stage: EngineStage::Actor,
                                        error: error.to_string(),
                                        snapshot: Box::new(actor_stage_snapshot.clone()),
                                    };
                                    return;
                                }
                            }
                        };

                        let mut final_response = None;

                        while let Some(event) = actor_stream.next().await {
                            match event {
                                Ok(ActorStreamEvent::ThoughtDelta { delta }) => {
                                    yield EngineEvent::ActorThoughtDelta {
                                        beat_index,
                                        speaker_id: speaker_id.clone(),
                                        delta,
                                    };
                                }
                                Ok(ActorStreamEvent::ActionComplete { text }) => {
                                    yield EngineEvent::ActorActionComplete {
                                        beat_index,
                                        speaker_id: speaker_id.clone(),
                                        text,
                                    };
                                }
                                Ok(ActorStreamEvent::DialogueDelta { delta }) => {
                                    yield EngineEvent::ActorDialogueDelta {
                                        beat_index,
                                        speaker_id: speaker_id.clone(),
                                        delta,
                                    };
                                }
                                Ok(ActorStreamEvent::Done { response }) => {
                                    yield EngineEvent::ActorCompleted {
                                        beat_index,
                                        speaker_id: speaker_id.clone(),
                                        purpose: purpose.clone(),
                                        response: Box::new(response.clone()),
                                    };
                                    final_response = Some(response);
                                }
                                Err(error) => {
                                    drop(actor_stream);
                                    yield EngineEvent::TurnFailed {
                                        stage: EngineStage::Actor,
                                        error: error.to_string(),
                                        snapshot: Box::new(actor_stage_snapshot.clone()),
                                    };
                                    return;
                                }
                            }
                        }

                        let Some(response) = final_response else {
                            drop(actor_stream);
                            yield EngineEvent::TurnFailed {
                                stage: EngineStage::Actor,
                                error: EngineError::MissingFinalBeat(
                                    "actor stream finished without Done".to_owned(),
                                )
                                .to_string(),
                                snapshot: Box::new(actor_stage_snapshot.clone()),
                            };
                            return;
                        };
                        drop(actor_stream);

                        info!(
                            story_id = %self.runtime_state.story_id(),
                            turn_index = self.runtime_state.turn_index(),
                            beat_index,
                            speaker_id = %speaker_id,
                            purpose = ?purpose,
                            summary = %json_for_log(&summarize_actor_response(&response)),
                            "actor completed beat"
                        );
                        debug!(
                            story_id = %self.runtime_state.story_id(),
                            turn_index = self.runtime_state.turn_index(),
                            beat_index,
                            speaker_id = %speaker_id,
                            purpose = ?purpose,
                            payload = %json_for_log(&response),
                            "actor response payload"
                        );

                        completed_beats.push(ExecutedBeat::Actor {
                            speaker_id,
                            purpose,
                            response,
                        });
                    }
                }
            }

            let mut second_keeper = match self
                .run_second_keeper(&player_input, &director_result, &completed_beats)
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    yield self.turn_failed_event(EngineStage::KeeperAfterTurnOutputs, error);
                    return;
                }
            };

            self.apply_second_keeper_progression_fallback(&director_result, &mut second_keeper);

            info!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                summary = %json_for_log(&summarize_keeper_response(KeeperPhase::AfterTurnOutputs, &second_keeper)),
                "keeper produced after-turn update"
            );
            debug!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                payload = %json_for_log(&second_keeper),
                "keeper after-turn payload"
            );

            self.runtime_state
                .world_state_mut()
                .apply_update(second_keeper.update.clone());

            yield EngineEvent::KeeperApplied {
                phase: KeeperPhase::AfterTurnOutputs,
                update: second_keeper.update.clone(),
                snapshot: Box::new(self.runtime_state.snapshot()),
            };

            let result = EngineTurnResult {
                turn_index: self.runtime_state.turn_index(),
                player_input,
                first_keeper,
                director: director_result,
                completed_beats,
                second_keeper,
                snapshot: self.runtime_state.snapshot(),
            };

            yield EngineEvent::TurnCompleted {
                result: Box::new(result),
            };
        };

        Ok(Box::pin(stream))
    }

    async fn run_first_keeper(&self, player_input: &str) -> Result<KeeperResponse, EngineError> {
        let current_node = self.runtime_state.current_node()?;
        let lorebook_sections = self.runtime_lorebook_sections(&current_node.id, player_input);

        debug!(
            story_id = %self.runtime_state.story_id(),
            turn_index = self.runtime_state.turn_index().saturating_add(1),
            phase = ?KeeperPhase::AfterPlayerInput,
            previous_node_id = ?Option::<&str>::None,
            current_node_id = %current_node.id,
            candidate_transition_keys = %json_for_log(&candidate_transition_keys(current_node)),
            "running keeper"
        );

        self.keeper
            .keep(KeeperRequest {
                phase: KeeperPhase::AfterPlayerInput,
                player_input,
                previous_node: None,
                current_node,
                character_cards: self.runtime_state.character_cards(),
                current_cast_ids: self.runtime_state.world_state().active_characters(),
                lorebook_base: lorebook_sections.base.clone(),
                lorebook_matched: lorebook_sections.matched.clone(),
                player_name: self.runtime_state.player_name(),
                player_description: self.runtime_state.player_description(),
                player_state_schema: self.runtime_state.player_state_schema(),
                world_state: self.runtime_state.world_state(),
                completed_beats: &[],
            })
            .await
            .map_err(EngineError::from)
    }

    async fn run_second_keeper(
        &self,
        player_input: &str,
        director_result: &DirectorResult,
        completed_beats: &[ExecutedBeat],
    ) -> Result<KeeperResponse, EngineError> {
        let current_node = self.runtime_state.current_node()?;
        let lorebook_sections = self.runtime_lorebook_sections(&current_node.id, player_input);
        let previous_node_index = self
            .runtime_state
            .runtime_graph()
            .get_node_index(&director_result.previous_node_id)
            .ok_or_else(|| {
                EngineError::MissingPreviousNode(director_result.previous_node_id.clone())
            })?;
        let previous_node = self
            .runtime_state
            .runtime_graph()
            .graph
            .node_weight(previous_node_index)
            .ok_or_else(|| {
                EngineError::MissingPreviousNode(director_result.previous_node_id.clone())
            })?;
        let keeper_beats: Vec<KeeperBeat> = completed_beats
            .iter()
            .map(ExecutedBeat::to_keeper_beat)
            .collect();

        debug!(
            story_id = %self.runtime_state.story_id(),
            turn_index = self.runtime_state.turn_index(),
            phase = ?KeeperPhase::AfterTurnOutputs,
            previous_node_id = %previous_node.id,
            current_node_id = %current_node.id,
            candidate_transition_keys = %json_for_log(&candidate_transition_keys(current_node)),
            matched_transition_keys = %json_for_log(&matched_transition_keys(previous_node, current_node.id.as_str())),
            "running keeper"
        );

        self.keeper
            .keep(KeeperRequest {
                phase: KeeperPhase::AfterTurnOutputs,
                player_input,
                previous_node: Some(previous_node),
                current_node,
                character_cards: self.runtime_state.character_cards(),
                current_cast_ids: self.runtime_state.world_state().active_characters(),
                lorebook_base: lorebook_sections.base.clone(),
                lorebook_matched: lorebook_sections.matched.clone(),
                player_name: self.runtime_state.player_name(),
                player_description: self.runtime_state.player_description(),
                player_state_schema: self.runtime_state.player_state_schema(),
                world_state: self.runtime_state.world_state(),
                completed_beats: &keeper_beats,
            })
            .await
            .map_err(EngineError::from)
    }

    fn apply_second_keeper_progression_fallback(
        &self,
        director_result: &DirectorResult,
        response: &mut KeeperResponse,
    ) {
        if !director_result.transitioned {
            return;
        }

        let Some(previous_node) = self
            .runtime_state
            .runtime_graph()
            .get_node_index(&director_result.previous_node_id)
            .and_then(|index| self.runtime_state.runtime_graph().graph.node_weight(index))
        else {
            debug!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                previous_node_id = %director_result.previous_node_id,
                current_node_id = %director_result.current_node_id,
                "skipping keeper progression fallback because previous node is missing"
            );
            return;
        };

        let Some(transition) = previous_node
            .transitions
            .iter()
            .find(|transition| transition.to == director_result.current_node_id)
        else {
            return;
        };

        let Some(condition) = transition.condition.as_ref() else {
            debug!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                previous_node_id = %director_result.previous_node_id,
                current_node_id = %director_result.current_node_id,
                "skipping keeper progression fallback because matched transition is unconditional"
            );
            return;
        };

        if condition_key_touched(&response.update, condition) {
            return;
        }

        let Some(op) = state_op_from_simple_transition_condition(condition) else {
            debug!(
                story_id = %self.runtime_state.story_id(),
                turn_index = self.runtime_state.turn_index(),
                previous_node_id = %director_result.previous_node_id,
                current_node_id = %director_result.current_node_id,
                transition_condition = %describe_condition(condition),
                "skipping keeper progression fallback because transition condition is not a simple equality"
            );
            return;
        };

        info!(
            story_id = %self.runtime_state.story_id(),
            turn_index = self.runtime_state.turn_index(),
            previous_node_id = %director_result.previous_node_id,
            current_node_id = %director_result.current_node_id,
            transition_condition = %describe_condition(condition),
            fallback_op = %json_for_log(&op),
            "applied keeper progression fallback"
        );
        response.update.add_op(op);
    }

    fn build_narrator_request(
        &self,
        director_result: &DirectorResult,
        purpose: NarratorPurpose,
        player_input: &str,
    ) -> Result<NarratorRequest<'_>, EngineError> {
        let previous_node = if matches!(purpose, NarratorPurpose::DescribeTransition) {
            let previous_node_index = self
                .runtime_state
                .runtime_graph()
                .get_node_index(&director_result.previous_node_id)
                .ok_or_else(|| {
                    EngineError::MissingPreviousNode(director_result.previous_node_id.clone())
                })?;
            Some(
                self.runtime_state
                    .runtime_graph()
                    .graph
                    .node_weight(previous_node_index)
                    .ok_or_else(|| {
                        EngineError::MissingPreviousNode(director_result.previous_node_id.clone())
                    })?,
            )
        } else {
            None
        };
        let current_node = self.runtime_state.current_node()?;
        let lorebook_sections = self.runtime_lorebook_sections(&current_node.id, player_input);

        Ok(NarratorRequest {
            purpose,
            previous_node,
            current_node,
            character_cards: self.runtime_state.character_cards(),
            current_cast_ids: self.runtime_state.world_state().active_characters(),
            lorebook_base: lorebook_sections.base,
            lorebook_matched: lorebook_sections.matched,
            player_name: self.runtime_state.player_name(),
            player_description: self.runtime_state.player_description(),
            player_state_schema: self.runtime_state.player_state_schema(),
            world_state: self.runtime_state.world_state(),
        })
    }

    fn runtime_lorebook_sections(
        &self,
        current_node_id: &str,
        player_input: &str,
    ) -> LorebookPromptSections {
        let node_texts = self
            .runtime_state
            .runtime_graph()
            .get_node_index(current_node_id)
            .and_then(|index| self.runtime_state.runtime_graph().graph.node_weight(index))
            .map(|node| vec![node.title.as_str(), node.scene.as_str(), node.goal.as_str()])
            .unwrap_or_default();
        let history_texts = self
            .runtime_state
            .world_state()
            .actor_shared_history()
            .iter()
            .rev()
            .take(self.shared_memory_limit)
            .map(|entry| entry.text.as_str())
            .collect::<Vec<_>>();
        let mut match_inputs = Vec::with_capacity(node_texts.len() + history_texts.len() + 1);
        match_inputs.extend(node_texts);
        match_inputs.extend(history_texts);
        match_inputs.push(player_input);

        build_lorebook_prompt_sections(self.runtime_state.lorebook_entries(), &match_inputs)
    }

    fn turn_failed_event(&self, stage: EngineStage, error: impl Into<EngineError>) -> EngineEvent {
        let error = error.into();

        EngineEvent::TurnFailed {
            stage,
            error: error.to_string(),
            snapshot: Box::new(self.runtime_state.snapshot()),
        }
    }

    fn apply_session_character_action(
        &mut self,
        action: &SessionCharacterAction,
    ) -> Result<SessionCharacterActionOutcome, EngineError> {
        match action {
            SessionCharacterAction::CreateAndEnter {
                session_character_id,
                display_name,
                personality,
                style,
                system_prompt,
            } => {
                let already_exists = self
                    .runtime_state
                    .has_session_character(session_character_id);
                let now = now_timestamp_ms();
                let record = SessionCharacterRecord {
                    session_character_id: session_character_id.clone(),
                    session_id: String::new(),
                    display_name: display_name.clone(),
                    personality: personality.clone(),
                    style: style.clone(),
                    system_prompt: system_prompt.clone(),
                    created_at_ms: now,
                    updated_at_ms: now,
                };
                self.runtime_state.upsert_session_character(
                    session_character_record_to_character_card(&record),
                )?;
                self.runtime_state
                    .world_state_mut()
                    .add_active_character(session_character_id.clone());

                if already_exists {
                    Ok(SessionCharacterActionOutcome::EnteredScene {
                        session_character_id: session_character_id.clone(),
                    })
                } else {
                    Ok(SessionCharacterActionOutcome::CreatedAndEntered { character: record })
                }
            }
            SessionCharacterAction::LeaveScene {
                session_character_id,
            } => {
                if self
                    .runtime_state
                    .world_state_mut()
                    .remove_active_character(session_character_id)
                {
                    Ok(SessionCharacterActionOutcome::LeftScene {
                        session_character_id: session_character_id.clone(),
                    })
                } else {
                    Ok(SessionCharacterActionOutcome::Noop)
                }
            }
        }
    }
}

enum SessionCharacterActionOutcome {
    CreatedAndEntered { character: SessionCharacterRecord },
    EnteredScene { session_character_id: String },
    LeftScene { session_character_id: String },
    Noop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineTurnResult {
    pub turn_index: u64,
    pub player_input: String,
    pub first_keeper: KeeperResponse,
    pub director: DirectorResult,
    pub completed_beats: Vec<ExecutedBeat>,
    pub second_keeper: KeeperResponse,
    pub snapshot: RuntimeSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExecutedBeat {
    Narrator {
        purpose: NarratorPurpose,
        response: NarratorResponse,
    },
    Actor {
        speaker_id: String,
        purpose: ActorPurpose,
        response: ActorResponse,
    },
}

impl ExecutedBeat {
    fn to_keeper_beat(&self) -> KeeperBeat {
        match self {
            Self::Narrator { purpose, response } => {
                KeeperBeat::from_narrator_response(purpose.clone(), response)
            }
            Self::Actor {
                purpose, response, ..
            } => KeeperBeat::from_actor_response(purpose.clone(), response),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Director(#[from] DirectorError),
    #[error(transparent)]
    Actor(#[from] ActorError),
    #[error(transparent)]
    Narrator(#[from] NarratorError),
    #[error(transparent)]
    Keeper(#[from] KeeperError),
    #[error(transparent)]
    Architect(#[from] ArchitectError),
    #[error(transparent)]
    Planner(#[from] PlannerError),
    #[error("missing previous node '{0}' in runtime graph")]
    MissingPreviousNode(String),
    #[error("actor beat references invalid speaker_id '{speaker_id}' for node '{node_id}'")]
    InvalidBeatSpeaker { speaker_id: String, node_id: String },
    #[error("turn stream ended without a final result")]
    IncompleteTurn,
    #[error("{0}")]
    MissingFinalBeat(String),
    #[error("turn failed during {stage:?}: {message}")]
    TurnFailed { stage: EngineStage, message: String },
}

fn action_session_character_id(action: &SessionCharacterAction) -> &str {
    match action {
        SessionCharacterAction::CreateAndEnter {
            session_character_id,
            ..
        }
        | SessionCharacterAction::LeaveScene {
            session_character_id,
        } => session_character_id,
    }
}

fn session_character_record_to_character_card(character: &SessionCharacterRecord) -> CharacterCard {
    CharacterCard {
        id: character.session_character_id.clone(),
        name: character.display_name.clone(),
        personality: character.personality.clone(),
        style: character.style.clone(),
        state_schema: Default::default(),
        system_prompt: character.system_prompt.clone(),
    }
}

fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

pub async fn generate_story_plan(
    agent_configs: &StoryGenerationAgentConfigs,
    resources: &StoryResources,
) -> Result<PlannerResponse, EngineError> {
    let planner = Planner::new_with_options(
        Arc::clone(&agent_configs.planner.client),
        agent_configs.planner.model.clone(),
        agent_configs.planner.temperature,
        agent_configs.planner.max_tokens,
    )?
    .with_prompt_profile(agent_configs.planner.prompt_profile.clone());
    let lorebook_sections = build_lorebook_prompt_sections(
        resources.lorebook_entries(),
        &[
            resources.story_concept(),
            resources.planned_story().unwrap_or(""),
        ],
    );
    planner
        .plan(PlannerRequest {
            story_concept: resources.story_concept(),
            available_characters: resources.character_cards(),
            lorebook_base: lorebook_sections.base,
            lorebook_matched: lorebook_sections.matched,
        })
        .await
        .inspect(|response| {
            info!(
                story_id = %resources.story_id(),
                summary = %json_for_log(&summarize_planner_response(response)),
                "planner generated story outline"
            );
            debug!(
                story_id = %resources.story_id(),
                payload = %json_for_log(&response),
                "planner response payload"
            );
        })
        .map_err(EngineError::from)
}

pub async fn generate_story_graph(
    agent_configs: &StoryGenerationAgentConfigs,
    resources: &StoryResources,
) -> Result<ArchitectResponse, EngineError> {
    let architect = Architect::new_with_options(
        Arc::clone(&agent_configs.architect.client),
        agent_configs.architect.model.clone(),
        Some(
            agent_configs
                .architect
                .temperature
                .unwrap_or(DEFAULT_ARCHITECT_GENERATE_TEMPERATURE),
        ),
        Some(
            agent_configs
                .architect
                .max_tokens
                .unwrap_or(DEFAULT_ARCHITECT_GENERATE_MAX_TOKENS),
        ),
    )
    .with_prompt_profiles(agent_configs.architect.prompt_profiles.clone());
    let lorebook_sections = build_lorebook_prompt_sections(
        resources.lorebook_entries(),
        &[
            resources.story_concept(),
            resources.planned_story().unwrap_or(""),
        ],
    );
    architect
        .generate_graph(ArchitectRequest {
            story_concept: resources.story_concept(),
            planned_story: resources.planned_story(),
            world_state_schema: resources.world_state_schema_seed(),
            player_state_schema: resources.player_state_schema_seed(),
            available_characters: resources.character_cards(),
            lorebook_base: lorebook_sections.base.as_deref(),
            lorebook_matched: lorebook_sections.matched.as_deref(),
        })
        .await
        .inspect(|response| {
            info!(
                story_id = %resources.story_id(),
                summary = %json_for_log(&summarize_architect_response(response)),
                "architect generated story graph"
            );
            debug!(
                story_id = %resources.story_id(),
                payload = %json_for_log(&response),
                "architect response payload"
            );
        })
        .map_err(EngineError::from)
}

fn record_player_input(
    world_state: &mut WorldState,
    player_input: &str,
    shared_memory_limit: usize,
) -> ActorMemoryEntry {
    let entry = ActorMemoryEntry {
        speaker_id: "player".to_owned(),
        speaker_name: "Player".to_owned(),
        kind: ActorMemoryKind::PlayerInput,
        text: player_input.to_owned(),
    };
    world_state.push_actor_shared_history(entry.clone(), shared_memory_limit);
    entry
}

fn record_narration(
    world_state: &mut WorldState,
    response: &NarratorResponse,
    shared_memory_limit: usize,
) {
    let text = response.text.trim();
    if text.is_empty() {
        return;
    }

    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "narrator".to_owned(),
            speaker_name: "Narrator".to_owned(),
            kind: ActorMemoryKind::Narration,
            text: text.to_owned(),
        },
        shared_memory_limit,
    );
}

fn candidate_transition_keys(node: &story::NarrativeNode) -> Vec<String> {
    node.transitions
        .iter()
        .filter_map(|transition| transition.condition.as_ref())
        .map(condition_key_label)
        .collect()
}

fn matched_transition_keys(
    previous_node: &story::NarrativeNode,
    current_node_id: &str,
) -> Vec<String> {
    previous_node
        .transitions
        .iter()
        .filter(|transition| transition.to == current_node_id)
        .filter_map(|transition| transition.condition.as_ref())
        .map(condition_key_label)
        .collect()
}

fn condition_key_label(condition: &Condition) -> String {
    match condition.scope {
        ConditionScope::Global => format!("global:{}", condition.key),
        ConditionScope::Player => format!("player:{}", condition.key),
        ConditionScope::Character => format!(
            "character:{}:{}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key
        ),
    }
}

fn condition_key_touched(update: &StateUpdate, condition: &Condition) -> bool {
    update.ops.iter().any(|op| match (op, &condition.scope) {
        (StateOp::SetState { key, .. }, ConditionScope::Global)
        | (StateOp::RemoveState { key }, ConditionScope::Global) => key == &condition.key,
        (StateOp::SetPlayerState { key, .. }, ConditionScope::Player)
        | (StateOp::RemovePlayerState { key }, ConditionScope::Player) => key == &condition.key,
        (StateOp::SetCharacterState { character, key, .. }, ConditionScope::Character)
        | (StateOp::RemoveCharacterState { character, key }, ConditionScope::Character) => {
            key == &condition.key
                && condition
                    .character
                    .as_deref()
                    .is_some_and(|expected| expected == character)
        }
        _ => false,
    })
}

fn state_op_from_simple_transition_condition(condition: &Condition) -> Option<StateOp> {
    if condition.op != ConditionOperator::Eq {
        return None;
    }

    Some(match condition.scope {
        ConditionScope::Global => StateOp::SetState {
            key: condition.key.clone(),
            value: condition.value.clone(),
        },
        ConditionScope::Player => StateOp::SetPlayerState {
            key: condition.key.clone(),
            value: condition.value.clone(),
        },
        ConditionScope::Character => StateOp::SetCharacterState {
            character: condition.character.clone()?,
            key: condition.key.clone(),
            value: condition.value.clone(),
        },
    })
}

fn describe_condition(condition: &Condition) -> String {
    let left = match condition.scope {
        ConditionScope::Global => format!("global.{}", condition.key),
        ConditionScope::Player => format!("player.{}", condition.key),
        ConditionScope::Character => format!(
            "character[{}].{}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key
        ),
    };

    let op = match condition.op {
        ConditionOperator::Eq => "==",
        ConditionOperator::Ne => "!=",
        ConditionOperator::Gt => ">",
        ConditionOperator::Gte => ">=",
        ConditionOperator::Lt => "<",
        ConditionOperator::Lte => "<=",
        ConditionOperator::Contains => "contains",
    };

    format!("{left} {op} {}", compact_value(&condition.value))
}

fn compact_value(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}
