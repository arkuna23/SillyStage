use std::sync::Arc;

use agents::actor::CharacterCard;
use async_stream::stream;
use engine::{AgentApiIdOverrides, Engine, EngineEvent, RuntimeState};
use futures_util::StreamExt;
use protocol::{
    JsonRpcResponseMessage, ResponseResult, RunTurnParams, ServerEventMessage, StreamEventBody,
    TurnCompletedPayload, TurnStreamAcceptedPayload, UpdatePlayerDescriptionParams,
};

use crate::error::HandlerError;
use crate::store::SessionRecord;

use super::config::{effective_session_api_ids, validate_api_ids};
use super::{Handler, HandlerReply, require_session_id};

impl<'a> Handler<'a> {
    pub(crate) async fn handle_session_run_turn(
        &self,
        request_id: String,
        session_id: Option<String>,
        params: RunTurnParams,
    ) -> HandlerReply<'a> {
        let session_id = match session_id {
            Some(session_id) => session_id,
            None => {
                return HandlerReply::Unary(JsonRpcResponseMessage::err(
                    request_id,
                    None::<String>,
                    HandlerError::MissingSessionId.to_error_payload(),
                ));
            }
        };

        let setup = self
            .prepare_turn_stream_state(
                &session_id,
                params.player_input.clone(),
                params.api_overrides,
            )
            .await;

        let (mut engine, session_record, request_runtime_input) = match setup {
            Ok(value) => value,
            Err(error) => {
                return HandlerReply::Unary(JsonRpcResponseMessage::err(
                    request_id,
                    Some(session_id),
                    error.to_error_payload(),
                ));
            }
        };

        let ack = JsonRpcResponseMessage::ok(
            &request_id,
            Some(session_id.clone()),
            ResponseResult::TurnStreamAccepted(TurnStreamAcceptedPayload::default()),
        );
        let store = Arc::clone(&self.store);

        let events = stream! {
            let mut sequence = 0_u64;
            yield ServerEventMessage::started(
                request_id.clone(),
                Some(session_id.clone()),
                sequence,
            );
            sequence = sequence.saturating_add(1);

            let mut engine_stream = match engine.run_turn_stream(&request_runtime_input).await {
                Ok(stream) => stream,
                Err(error) => {
                    yield ServerEventMessage::failed(
                        request_id.clone(),
                        Some(session_id.clone()),
                        sequence,
                        HandlerError::Engine(error).to_error_payload(),
                    );
                    return;
                }
            };

            while let Some(event) = engine_stream.next().await {
                match event {
                    EngineEvent::TurnStarted { next_turn_index, player_input } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::TurnStarted {
                                next_turn_index,
                                player_input,
                            },
                        );
                    }
                    EngineEvent::PlayerInputRecorded { entry, snapshot } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::PlayerInputRecorded { entry, snapshot },
                        );
                    }
                    EngineEvent::KeeperApplied {
                        phase,
                        update,
                        snapshot,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::KeeperApplied {
                                phase,
                                update,
                                snapshot,
                            },
                        );
                    }
                    EngineEvent::DirectorCompleted { result, snapshot } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::DirectorCompleted { result, snapshot },
                        );
                    }
                    EngineEvent::NarratorStarted { beat_index, purpose } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorStarted { beat_index, purpose },
                        );
                    }
                    EngineEvent::NarratorTextDelta {
                        beat_index,
                        purpose,
                        delta,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorTextDelta {
                                beat_index,
                                purpose,
                                delta,
                            },
                        );
                    }
                    EngineEvent::NarratorCompleted {
                        beat_index,
                        purpose,
                        response,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorCompleted {
                                beat_index,
                                purpose,
                                response,
                            },
                        );
                    }
                    EngineEvent::ActorStarted {
                        beat_index,
                        speaker_id,
                        purpose,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorStarted {
                                beat_index,
                                speaker_id,
                                purpose,
                            },
                        );
                    }
                    EngineEvent::ActorThoughtDelta {
                        beat_index,
                        speaker_id,
                        delta,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorThoughtDelta {
                                beat_index,
                                speaker_id,
                                delta,
                            },
                        );
                    }
                    EngineEvent::ActorActionComplete {
                        beat_index,
                        speaker_id,
                        text,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorActionComplete {
                                beat_index,
                                speaker_id,
                                text,
                            },
                        );
                    }
                    EngineEvent::ActorDialogueDelta {
                        beat_index,
                        speaker_id,
                        delta,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorDialogueDelta {
                                beat_index,
                                speaker_id,
                                delta,
                            },
                        );
                    }
                    EngineEvent::ActorCompleted {
                        beat_index,
                        speaker_id,
                        purpose,
                        response,
                    } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorCompleted {
                                beat_index,
                                speaker_id,
                                purpose,
                                response,
                            },
                        );
                    }
                    EngineEvent::TurnCompleted { result } => {
                        let mut updated_session = session_record.clone();
                        updated_session.snapshot = result.snapshot.clone();

                        if let Err(error) = store.save_session(updated_session).await {
                            yield ServerEventMessage::failed(
                                request_id.clone(),
                                Some(session_id.clone()),
                                sequence,
                                HandlerError::Store(error).to_error_payload(),
                            );
                            return;
                        }

                        yield ServerEventMessage::completed(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            ResponseResult::TurnCompleted(Box::new(TurnCompletedPayload {
                                result: *result,
                            })),
                        );
                        return;
                    }
                    EngineEvent::TurnFailed {
                        stage,
                        error,
                        snapshot,
                    } => {
                        let mut updated_session = session_record.clone();
                        updated_session.snapshot = (*snapshot).clone();

                        if let Err(store_error) = store.save_session(updated_session).await {
                            yield ServerEventMessage::failed(
                                request_id.clone(),
                                Some(session_id.clone()),
                                sequence,
                                HandlerError::Store(store_error).to_error_payload(),
                            );
                            return;
                        }

                        yield ServerEventMessage::failed(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            HandlerError::Engine(engine::EngineError::TurnFailed {
                                stage,
                                message: error,
                            })
                            .to_error_payload(),
                        );
                        return;
                    }
                }

                sequence = sequence.saturating_add(1);
            }
        };

        HandlerReply::Stream {
            ack,
            events: Box::pin(events),
        }
    }

    pub(crate) async fn handle_session_update_player_description(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: UpdatePlayerDescriptionParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        session.snapshot.player_description = params.player_description;
        let snapshot = session.snapshot.clone();
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::PlayerDescriptionUpdated(Box::new(
                protocol::PlayerDescriptionUpdatedPayload { snapshot },
            )),
        ))
    }

    pub(crate) async fn handle_session_get_runtime_snapshot(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::RuntimeSnapshot(Box::new(protocol::RuntimeSnapshotPayload {
                snapshot: session.snapshot,
            })),
        ))
    }

    async fn prepare_turn_stream_state(
        &self,
        session_id: &str,
        player_input: String,
        api_overrides: Option<AgentApiIdOverrides>,
    ) -> Result<(Engine<'a>, SessionRecord, String), HandlerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.to_owned()))?;
        let story = self
            .store
            .get_story(&session.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(session.story_id.clone()))?;
        let character_cards = self
            .load_story_character_cards(&story.resource_id)
            .await?
            .into_iter()
            .map(|record| CharacterCard::from(record.archive.content))
            .collect::<Vec<_>>();
        let runtime_state = RuntimeState::from_snapshot(
            &story.story_id,
            story::runtime_graph::RuntimeStoryGraph::from_story_graph(
                story.generated.graph.clone(),
            )
            .map_err(engine::RuntimeError::GraphBuild)?,
            character_cards,
            story.generated.player_state_schema.clone(),
            session.snapshot.clone(),
        )?;
        let global = self.load_global_config().await?;
        let effective = effective_session_api_ids(&session.config, &global)
            .apply_overrides(&api_overrides.unwrap_or_default());
        validate_api_ids(&self.registry, &effective)?;
        let runtime_configs = self.registry.build_runtime_configs(&effective)?;
        let engine = Engine::new(runtime_configs, runtime_state)?;

        Ok((engine, session, player_input))
    }
}
