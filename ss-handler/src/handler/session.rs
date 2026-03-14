use async_stream::stream;
use futures_util::StreamExt;
use protocol::{
    JsonRpcResponseMessage, ResponseResult, RunTurnParams, RuntimeSnapshotPayload,
    ServerEventMessage, SessionDeletedPayload, SessionDetailPayload, SessionSummaryPayload,
    SessionsListedPayload, SetPlayerProfileParams, StreamEventBody, TurnCompletedPayload,
    TurnStreamAcceptedPayload, UpdatePlayerDescriptionParams,
};

use crate::error::HandlerError;

use super::config::build_session_config_payload;
use super::{Handler, HandlerReply, require_session_id};

impl Handler {
    pub(crate) async fn handle_session_get(
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
        let config = build_session_config_payload(
            self.manager
                .get_resolved_session_config(&session_id)
                .await?,
        );

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::Session(Box::new(SessionDetailPayload {
                session_id: session.session_id,
                story_id: session.story_id,
                display_name: session.display_name,
                player_profile_id: session.player_profile_id,
                player_schema_id: session.player_schema_id,
                snapshot: session.snapshot,
                config,
            })),
        ))
    }

    pub(crate) async fn handle_session_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let sessions = self
            .store
            .list_sessions()
            .await?
            .into_iter()
            .map(|session| SessionSummaryPayload {
                session_id: session.session_id,
                story_id: session.story_id,
                display_name: session.display_name,
                player_profile_id: session.player_profile_id,
                player_schema_id: session.player_schema_id,
                turn_index: session.snapshot.turn_index,
            })
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::SessionsListed(SessionsListedPayload { sessions }),
        ))
    }

    pub(crate) async fn handle_session_delete(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        self.store
            .delete_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::SessionDeleted(SessionDeletedPayload { session_id }),
        ))
    }

    pub(crate) async fn handle_session_run_turn(
        &self,
        request_id: String,
        session_id: Option<String>,
        params: RunTurnParams,
    ) -> HandlerReply {
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

        let managed_stream = match self
            .manager
            .run_turn_stream(&session_id, params.player_input, params.api_overrides)
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                return HandlerReply::Unary(JsonRpcResponseMessage::err(
                    request_id,
                    Some(session_id),
                    HandlerError::from(error).to_error_payload(),
                ));
            }
        };

        let ack = JsonRpcResponseMessage::ok(
            &request_id,
            Some(session_id.clone()),
            ResponseResult::TurnStreamAccepted(TurnStreamAcceptedPayload::default()),
        );

        let events = stream! {
            let mut sequence = 0_u64;
            yield ServerEventMessage::started(
                request_id.clone(),
                Some(session_id.clone()),
                sequence,
            );
            sequence = sequence.saturating_add(1);

            let mut managed_stream = managed_stream;
            while let Some(event) = managed_stream.next().await {
                match event {
                    Ok(engine::EngineEvent::TurnStarted { next_turn_index, player_input }) => {
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
                    Ok(engine::EngineEvent::PlayerInputRecorded { entry, snapshot }) => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::PlayerInputRecorded { entry, snapshot },
                        );
                    }
                    Ok(engine::EngineEvent::KeeperApplied { phase, update, snapshot }) => {
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
                    Ok(engine::EngineEvent::DirectorCompleted { result, snapshot }) => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::DirectorCompleted { result, snapshot },
                        );
                    }
                    Ok(engine::EngineEvent::NarratorStarted { beat_index, purpose }) => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorStarted { beat_index, purpose },
                        );
                    }
                    Ok(engine::EngineEvent::NarratorTextDelta {
                        beat_index,
                        purpose,
                        delta,
                    }) => {
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
                    Ok(engine::EngineEvent::NarratorCompleted {
                        beat_index,
                        purpose,
                        response,
                    }) => {
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
                    Ok(engine::EngineEvent::ActorStarted {
                        beat_index,
                        speaker_id,
                        purpose,
                    }) => {
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
                    Ok(engine::EngineEvent::ActorThoughtDelta {
                        beat_index,
                        speaker_id,
                        delta,
                    }) => {
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
                    Ok(engine::EngineEvent::ActorActionComplete {
                        beat_index,
                        speaker_id,
                        text,
                    }) => {
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
                    Ok(engine::EngineEvent::ActorDialogueDelta {
                        beat_index,
                        speaker_id,
                        delta,
                    }) => {
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
                    Ok(engine::EngineEvent::ActorCompleted {
                        beat_index,
                        speaker_id,
                        purpose,
                        response,
                    }) => {
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
                    Ok(engine::EngineEvent::TurnCompleted { result }) => {
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
                    Ok(engine::EngineEvent::TurnFailed { stage, error, .. }) => {
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
                    Err(error) => {
                        yield ServerEventMessage::failed(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            HandlerError::from(error).to_error_payload(),
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
        let snapshot = self
            .manager
            .update_player_description(&session_id, params.player_description)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::PlayerDescriptionUpdated(Box::new(
                protocol::PlayerDescriptionUpdatedPayload { snapshot },
            )),
        ))
    }

    pub(crate) async fn handle_session_set_player_profile(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: SetPlayerProfileParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .manager
            .set_player_profile(&session_id, params.player_profile_id)
            .await?;
        let config = build_session_config_payload(
            self.manager
                .get_resolved_session_config(&session_id)
                .await?,
        );

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::Session(Box::new(SessionDetailPayload {
                session_id: session.session_id,
                story_id: session.story_id,
                display_name: session.display_name,
                player_profile_id: session.player_profile_id,
                player_schema_id: session.player_schema_id,
                snapshot: session.snapshot,
                config,
            })),
        ))
    }

    pub(crate) async fn handle_session_get_runtime_snapshot(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let snapshot = self.manager.get_runtime_snapshot(&session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::RuntimeSnapshot(Box::new(RuntimeSnapshotPayload { snapshot })),
        ))
    }
}
