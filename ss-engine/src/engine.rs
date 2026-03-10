use std::pin::Pin;

use agents::actor::{Actor, ActorError, ActorRequest, ActorResponse, ActorStreamEvent};
use agents::architect::{Architect, ArchitectError, ArchitectRequest, ArchitectResponse};
use agents::director::{
    ActorPurpose, Director, DirectorError, DirectorResult, NarratorPurpose, ResponseBeat,
};
use agents::keeper::{Keeper, KeeperBeat, KeeperError, KeeperPhase, KeeperRequest, KeeperResponse};
use agents::narrator::{
    Narrator, NarratorError, NarratorRequest, NarratorResponse, NarratorStreamEvent,
};
use async_stream::stream;
use futures_core::Stream;
use futures_util::StreamExt;
use llm::LlmApi;
use serde::{Deserialize, Serialize};
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};

use crate::event::{EngineEvent, EngineStage};
use crate::runtime::{RuntimeError, RuntimeSnapshot, RuntimeState, StoryResources};

const DEFAULT_SHARED_MEMORY_LIMIT: usize = 8;

pub type EngineTurnStream<'a> = Pin<Box<dyn Stream<Item = EngineEvent> + Send + 'a>>;

pub struct Engine<'a> {
    runtime_state: RuntimeState,
    director: Director<'a>,
    actor: Actor<'a>,
    narrator: Narrator<'a>,
    keeper: Keeper<'a>,
}

impl<'a> Engine<'a> {
    pub fn new(
        llm: &'a dyn LlmApi,
        model: impl Into<String>,
        runtime_state: RuntimeState,
    ) -> Result<Self, EngineError> {
        let model = model.into();

        Ok(Self {
            runtime_state,
            director: Director::new(llm, model.clone())?,
            actor: Actor::new(llm, model.clone())?,
            narrator: Narrator::new(llm, model.clone())?,
            keeper: Keeper::new(llm, model)?,
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

            let recorded_entry = record_player_input(self.runtime_state.world_state_mut(), &player_input);
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
                let parts = self.runtime_state.engine_parts();
                self.director
                    .decide_strict(
                        parts.runtime_graph,
                        parts.world_state,
                        parts.character_cards,
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

            yield EngineEvent::DirectorCompleted {
                result: director_result.clone(),
                snapshot: Box::new(self.runtime_state.snapshot()),
            };

            let mut completed_beats = Vec::new();

            for (beat_index, beat) in director_result.response_plan.beats.iter().enumerate() {
                match beat {
                    ResponseBeat::Narrator { purpose } => {
                        let purpose = purpose.clone();
                        yield EngineEvent::NarratorStarted { beat_index, purpose: purpose.clone() };

                        let narrator_request = match self.build_narrator_request(&director_result, purpose.clone()) {
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
                                            EngineError::Runtime(RuntimeError::MissingCurrentNode(
                                                current_node_id.clone(),
                                            ))
                                        });

                                    match current_node {
                                        Ok(current_node) => {
                                            if !current_node.has_character(&speaker_id) {
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
                                                                purpose: purpose.clone(),
                                                                node: current_node,
                                                                memory_limit: None,
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
                        let mut actor_stream = match actor_stream_result {
                            Ok(stream) => stream,
                            Err(error) => {
                                yield EngineEvent::TurnFailed {
                                    stage: EngineStage::Actor,
                                    error: error.to_string(),
                                    snapshot: Box::new(actor_stage_snapshot.clone()),
                                };
                                return;
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

                        completed_beats.push(ExecutedBeat::Actor {
                            speaker_id,
                            purpose,
                            response,
                        });
                    }
                }
            }

            let second_keeper = match self
                .run_second_keeper(&player_input, &director_result, &completed_beats)
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    yield self.turn_failed_event(EngineStage::KeeperAfterTurnOutputs, error);
                    return;
                }
            };

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

        self.keeper
            .keep(KeeperRequest {
                phase: KeeperPhase::AfterPlayerInput,
                player_input,
                previous_node: None,
                current_node,
                character_cards: self.runtime_state.character_cards(),
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

        self.keeper
            .keep(KeeperRequest {
                phase: KeeperPhase::AfterTurnOutputs,
                player_input,
                previous_node: Some(previous_node),
                current_node,
                character_cards: self.runtime_state.character_cards(),
                player_state_schema: self.runtime_state.player_state_schema(),
                world_state: self.runtime_state.world_state(),
                completed_beats: &keeper_beats,
            })
            .await
            .map_err(EngineError::from)
    }

    fn build_narrator_request(
        &self,
        director_result: &DirectorResult,
        purpose: NarratorPurpose,
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

        Ok(NarratorRequest {
            purpose,
            previous_node,
            current_node: self.runtime_state.current_node()?,
            character_cards: self.runtime_state.character_cards(),
            player_state_schema: self.runtime_state.player_state_schema(),
            world_state: self.runtime_state.world_state(),
        })
    }

    fn turn_failed_event(&self, stage: EngineStage, error: impl Into<EngineError>) -> EngineEvent {
        let error = error.into();

        EngineEvent::TurnFailed {
            stage,
            error: error.to_string(),
            snapshot: Box::new(self.runtime_state.snapshot()),
        }
    }
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

pub async fn generate_story_graph(
    llm: &dyn LlmApi,
    model: impl Into<String>,
    resources: &StoryResources,
) -> Result<ArchitectResponse, EngineError> {
    let architect = Architect::new(llm, model);
    architect
        .generate_graph(ArchitectRequest {
            story_concept: resources.story_concept(),
            world_state_schema: resources.world_state_schema_seed(),
            available_characters: resources.character_cards(),
        })
        .await
        .map_err(EngineError::from)
}

fn record_player_input(world_state: &mut WorldState, player_input: &str) -> ActorMemoryEntry {
    let entry = ActorMemoryEntry {
        speaker_id: "player".to_owned(),
        speaker_name: "Player".to_owned(),
        kind: ActorMemoryKind::PlayerInput,
        text: player_input.to_owned(),
    };
    world_state.push_actor_shared_history(entry.clone(), DEFAULT_SHARED_MEMORY_LIMIT);
    entry
}
