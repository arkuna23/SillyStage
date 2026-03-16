use async_stream::stream;
use engine::{ReplyOption, SessionCharacterUpdate};
use futures_util::StreamExt;
use protocol::{
    CreateSessionMessageParams, DeleteSessionCharacterParams, DeleteSessionMessageParams,
    EnterSessionCharacterSceneParams, GetSessionCharacterParams, GetSessionMessageParams,
    JsonRpcResponseMessage, LeaveSessionCharacterSceneParams, ListSessionCharactersParams,
    ListSessionMessagesParams, ResponseResult, RunTurnParams, RuntimeSnapshotPayload,
    ServerEventMessage, SessionCharacterDeletedPayload, SessionCharacterPayload,
    SessionCharactersListedPayload, SessionDeletedPayload, SessionDetailPayload,
    SessionMessageDeletedPayload, SessionMessageKind as SessionMessagePayloadKind,
    SessionMessagePayload, SessionMessagesListedPayload, SessionStartedPayload,
    SessionSummaryPayload, SessionVariablesPayload, SessionsListedPayload, SetPlayerProfileParams,
    StreamEventBody, SuggestRepliesParams, SuggestedRepliesPayload, TurnCompletedPayload,
    TurnStreamAcceptedPayload, UpdatePlayerDescriptionParams, UpdateSessionCharacterParams,
    UpdateSessionMessageParams, UpdateSessionParams, UpdateSessionVariablesParams,
};
use state::{StateOp, StateUpdate, WorldState};
use std::time::{SystemTime, UNIX_EPOCH};
use store::{
    SessionCharacterRecord, SessionMessageKind, SessionMessageRecord, SessionRecord, Store,
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
        let history = load_session_message_payloads(self.store.as_ref(), &session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::Session(Box::new(build_session_detail_payload(
                &session, history, config,
            ))),
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
            .map(|session| build_session_summary_payload(&session))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::SessionsListed(SessionsListedPayload { sessions }),
        ))
    }

    pub(crate) async fn handle_session_update(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: UpdateSessionParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        session.display_name = params.display_name;
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;

        let config = build_session_config_payload(
            self.manager
                .get_resolved_session_config(&session_id)
                .await?,
        );
        let history = load_session_message_payloads(self.store.as_ref(), &session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id.clone()),
            ResponseResult::Session(Box::new(build_session_detail_payload(
                &session, history, config,
            ))),
        ))
    }

    pub(crate) async fn handle_session_delete(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        for message in self.store.list_session_messages(&session_id).await? {
            self.store
                .delete_session_message(&message.message_id)
                .await?;
        }
        for character in self.store.list_session_characters(&session_id).await? {
            self.store
                .delete_session_character(&character.session_character_id)
                .await?;
        }
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
            .run_turn_stream(&session_id, params.player_input)
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
                    Ok(engine::EngineEvent::SessionCharacterCreated { character, snapshot }) => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::SessionCharacterCreated {
                                session_character: Box::new(build_session_character_payload(
                                    &character,
                                    snapshot.world_state.active_characters(),
                                )),
                                snapshot,
                            },
                        );
                    }
                    Ok(engine::EngineEvent::SessionCharacterEnteredScene {
                        session_character_id,
                        snapshot,
                    }) => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::SessionCharacterEnteredScene {
                                session_character_id,
                                snapshot,
                            },
                        );
                    }
                    Ok(engine::EngineEvent::SessionCharacterLeftScene {
                        session_character_id,
                        snapshot,
                    }) => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::SessionCharacterLeftScene {
                                session_character_id,
                                snapshot,
                            },
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

    pub(crate) async fn handle_session_get_variables(
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
            ResponseResult::SessionVariables(Box::new(build_session_variables_payload(
                &session.snapshot.world_state,
            ))),
        ))
    }

    pub(crate) async fn handle_session_update_variables(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: UpdateSessionVariablesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        validate_session_variable_update(&params.update)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        session.snapshot.world_state.apply_update(params.update);
        session.updated_at_ms = Some(now_timestamp_ms());
        let payload = build_session_variables_payload(&session.snapshot.world_state);
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionVariables(Box::new(payload)),
        ))
    }

    pub(crate) async fn handle_session_suggest_replies(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: SuggestRepliesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let limit = parse_suggested_reply_limit(params.limit)?;
        let replies = self.manager.suggest_replies(&session_id, limit).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SuggestedReplies(SuggestedRepliesPayload {
                replies: replies
                    .into_iter()
                    .map(reply_option_payload_from_reply_option)
                    .collect(),
            }),
        ))
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
        let history = load_session_message_payloads(self.store.as_ref(), &session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::Session(Box::new(build_session_detail_payload(
                &session, history, config,
            ))),
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

    pub(crate) async fn handle_session_message_create(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: CreateSessionMessageParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let now = now_timestamp_ms();
        let sequence =
            next_session_message_sequence(&self.store.list_session_messages(&session_id).await?);
        let message = SessionMessageRecord {
            message_id: self.id_generator.next("session-message"),
            session_id: session_id.clone(),
            kind: session_message_kind_from_payload(params.kind),
            sequence,
            turn_index: session.snapshot.turn_index,
            recorded_at_ms: now,
            created_at_ms: now,
            updated_at_ms: now,
            speaker_id: params.speaker_id,
            speaker_name: params.speaker_name,
            text: params.text,
        };
        self.store.save_session_message(message.clone()).await?;
        session.updated_at_ms = Some(now);
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionMessage(Box::new(build_session_message_payload(&message))),
        ))
    }

    pub(crate) async fn handle_session_message_get(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: GetSessionMessageParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let message = self
            .store
            .get_session_message(&params.message_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSessionMessage(params.message_id.clone()))?;
        ensure_session_message_belongs_to(&session_id, &message)?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionMessage(Box::new(build_session_message_payload(&message))),
        ))
    }

    pub(crate) async fn handle_session_message_list(
        &self,
        request_id: &str,
        session_id: Option<String>,
        _params: ListSessionMessagesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        self.store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let messages = load_session_message_payloads(self.store.as_ref(), &session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionMessagesListed(SessionMessagesListedPayload { messages }),
        ))
    }

    pub(crate) async fn handle_session_message_update(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: UpdateSessionMessageParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let mut message = self
            .store
            .get_session_message(&params.message_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSessionMessage(params.message_id.clone()))?;
        ensure_session_message_belongs_to(&session_id, &message)?;

        message.kind = session_message_kind_from_payload(params.kind);
        message.speaker_id = params.speaker_id;
        message.speaker_name = params.speaker_name;
        message.text = params.text;
        message.updated_at_ms = now_timestamp_ms();
        self.store.save_session_message(message.clone()).await?;
        session.updated_at_ms = Some(message.updated_at_ms);
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionMessage(Box::new(build_session_message_payload(&message))),
        ))
    }

    pub(crate) async fn handle_session_message_delete(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: DeleteSessionMessageParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let message = self
            .store
            .get_session_message(&params.message_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSessionMessage(params.message_id.clone()))?;
        ensure_session_message_belongs_to(&session_id, &message)?;
        self.store
            .delete_session_message(&params.message_id)
            .await?;
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionMessageDeleted(SessionMessageDeletedPayload {
                message_id: params.message_id,
            }),
        ))
    }

    pub(crate) async fn handle_session_character_get(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: GetSessionCharacterParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let character = self
            .manager
            .get_session_character(&session_id, &params.session_character_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionCharacter(Box::new(build_session_character_payload(
                &character,
                session.snapshot.world_state.active_characters(),
            ))),
        ))
    }

    pub(crate) async fn handle_session_character_list(
        &self,
        request_id: &str,
        session_id: Option<String>,
        _params: ListSessionCharactersParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let session_characters = self.manager.list_session_characters(&session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionCharactersListed(SessionCharactersListedPayload {
                session_characters: session_characters
                    .iter()
                    .map(|character| {
                        build_session_character_payload(
                            character,
                            session.snapshot.world_state.active_characters(),
                        )
                    })
                    .collect(),
            }),
        ))
    }

    pub(crate) async fn handle_session_character_update(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: UpdateSessionCharacterParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let character = self
            .manager
            .update_session_character(
                &session_id,
                &params.session_character_id,
                SessionCharacterUpdate {
                    display_name: params.display_name,
                    personality: params.personality,
                    style: params.style,
                    system_prompt: params.system_prompt,
                },
            )
            .await?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionCharacter(Box::new(build_session_character_payload(
                &character,
                session.snapshot.world_state.active_characters(),
            ))),
        ))
    }

    pub(crate) async fn handle_session_character_delete(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: DeleteSessionCharacterParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let deleted = self
            .manager
            .delete_session_character(&session_id, &params.session_character_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionCharacterDeleted(SessionCharacterDeletedPayload {
                session_character_id: deleted.session_character_id,
            }),
        ))
    }

    pub(crate) async fn handle_session_character_enter_scene(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: EnterSessionCharacterSceneParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let (session, character) = self
            .manager
            .enter_session_character_scene(&session_id, &params.session_character_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionCharacter(Box::new(build_session_character_payload(
                &character,
                session.snapshot.world_state.active_characters(),
            ))),
        ))
    }

    pub(crate) async fn handle_session_character_leave_scene(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: LeaveSessionCharacterSceneParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let (session, character) = self
            .manager
            .leave_session_character_scene(&session_id, &params.session_character_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionCharacter(Box::new(build_session_character_payload(
                &character,
                session.snapshot.world_state.active_characters(),
            ))),
        ))
    }
}

fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis() as u64
}

pub(crate) fn build_session_started_payload(
    session: &SessionRecord,
    history: Vec<SessionMessagePayload>,
    character_summaries: Vec<protocol::CharacterCardSummaryPayload>,
    config: protocol::SessionConfigPayload,
) -> SessionStartedPayload {
    SessionStartedPayload {
        story_id: session.story_id.clone(),
        display_name: session.display_name.clone(),
        snapshot: session.snapshot.clone(),
        player_profile_id: session.player_profile_id.clone(),
        player_schema_id: session.player_schema_id.clone(),
        api_group_id: session.binding.api_group_id.clone(),
        preset_id: session.binding.preset_id.clone(),
        history,
        created_at_ms: session.created_at_ms,
        updated_at_ms: session.updated_at_ms,
        character_summaries,
        config,
    }
}

pub(crate) fn build_session_detail_payload(
    session: &SessionRecord,
    history: Vec<SessionMessagePayload>,
    config: protocol::SessionConfigPayload,
) -> SessionDetailPayload {
    SessionDetailPayload {
        session_id: session.session_id.clone(),
        story_id: session.story_id.clone(),
        display_name: session.display_name.clone(),
        player_profile_id: session.player_profile_id.clone(),
        player_schema_id: session.player_schema_id.clone(),
        api_group_id: session.binding.api_group_id.clone(),
        preset_id: session.binding.preset_id.clone(),
        snapshot: session.snapshot.clone(),
        history,
        created_at_ms: session.created_at_ms,
        updated_at_ms: session.updated_at_ms,
        config,
    }
}

pub(crate) fn build_session_summary_payload(session: &SessionRecord) -> SessionSummaryPayload {
    SessionSummaryPayload {
        session_id: session.session_id.clone(),
        story_id: session.story_id.clone(),
        display_name: session.display_name.clone(),
        player_profile_id: session.player_profile_id.clone(),
        player_schema_id: session.player_schema_id.clone(),
        api_group_id: session.binding.api_group_id.clone(),
        preset_id: session.binding.preset_id.clone(),
        turn_index: session.snapshot.turn_index,
        created_at_ms: session.created_at_ms,
        updated_at_ms: session.updated_at_ms,
    }
}

pub(crate) async fn load_session_message_payloads(
    store: &dyn Store,
    session_id: &str,
) -> Result<Vec<SessionMessagePayload>, HandlerError> {
    let mut messages = store.list_session_messages(session_id).await?;
    messages.sort_by_key(|message| message.sequence);
    Ok(messages.iter().map(build_session_message_payload).collect())
}

fn build_session_message_payload(message: &SessionMessageRecord) -> SessionMessagePayload {
    SessionMessagePayload {
        message_id: message.message_id.clone(),
        kind: match message.kind {
            SessionMessageKind::PlayerInput => SessionMessagePayloadKind::PlayerInput,
            SessionMessageKind::Narration => SessionMessagePayloadKind::Narration,
            SessionMessageKind::Dialogue => SessionMessagePayloadKind::Dialogue,
            SessionMessageKind::Action => SessionMessagePayloadKind::Action,
        },
        sequence: message.sequence,
        turn_index: message.turn_index,
        recorded_at_ms: message.recorded_at_ms,
        created_at_ms: message.created_at_ms,
        updated_at_ms: message.updated_at_ms,
        speaker_id: message.speaker_id.clone(),
        speaker_name: message.speaker_name.clone(),
        text: message.text.clone(),
    }
}

fn build_session_character_payload(
    character: &SessionCharacterRecord,
    active_characters: &[String],
) -> SessionCharacterPayload {
    SessionCharacterPayload {
        session_character_id: character.session_character_id.clone(),
        display_name: character.display_name.clone(),
        personality: character.personality.clone(),
        style: character.style.clone(),
        system_prompt: character.system_prompt.clone(),
        in_scene: active_characters
            .iter()
            .any(|id| id == &character.session_character_id),
        created_at_ms: character.created_at_ms,
        updated_at_ms: character.updated_at_ms,
    }
}

fn session_message_kind_from_payload(kind: SessionMessagePayloadKind) -> SessionMessageKind {
    match kind {
        SessionMessagePayloadKind::PlayerInput => SessionMessageKind::PlayerInput,
        SessionMessagePayloadKind::Narration => SessionMessageKind::Narration,
        SessionMessagePayloadKind::Dialogue => SessionMessageKind::Dialogue,
        SessionMessagePayloadKind::Action => SessionMessageKind::Action,
    }
}

fn ensure_session_message_belongs_to(
    session_id: &str,
    message: &SessionMessageRecord,
) -> Result<(), HandlerError> {
    if message.session_id == session_id {
        return Ok(());
    }

    Err(HandlerError::MissingSessionMessage(
        message.message_id.clone(),
    ))
}

fn next_session_message_sequence(existing: &[SessionMessageRecord]) -> u64 {
    existing
        .iter()
        .map(|message| message.sequence)
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(0)
}

fn parse_suggested_reply_limit(limit: Option<u32>) -> Result<usize, HandlerError> {
    let limit = limit.unwrap_or(3);
    if !(2..=5).contains(&limit) {
        return Err(HandlerError::InvalidSuggestedReplyLimit(limit));
    }

    Ok(limit as usize)
}

fn build_session_variables_payload(world_state: &WorldState) -> SessionVariablesPayload {
    SessionVariablesPayload {
        custom: world_state.custom.clone(),
        player_state: world_state.player_state.clone(),
        character_state: world_state.character_state.clone(),
    }
}

fn validate_session_variable_update(update: &StateUpdate) -> Result<(), HandlerError> {
    for op in &update.ops {
        match op {
            StateOp::SetState { .. }
            | StateOp::RemoveState { .. }
            | StateOp::SetPlayerState { .. }
            | StateOp::RemovePlayerState { .. }
            | StateOp::SetCharacterState { .. }
            | StateOp::RemoveCharacterState { .. } => {}
            StateOp::SetCurrentNode { .. }
            | StateOp::SetActiveCharacters { .. }
            | StateOp::AddActiveCharacter { .. }
            | StateOp::RemoveActiveCharacter { .. } => {
                return Err(HandlerError::InvalidSessionVariableUpdate(format!(
                    "op '{}' is not allowed",
                    session_variable_op_name(op)
                )));
            }
        }
    }

    Ok(())
}

fn session_variable_op_name(op: &StateOp) -> &'static str {
    match op {
        StateOp::SetCurrentNode { .. } => "SetCurrentNode",
        StateOp::SetActiveCharacters { .. } => "SetActiveCharacters",
        StateOp::AddActiveCharacter { .. } => "AddActiveCharacter",
        StateOp::RemoveActiveCharacter { .. } => "RemoveActiveCharacter",
        StateOp::SetState { .. } => "SetState",
        StateOp::RemoveState { .. } => "RemoveState",
        StateOp::SetPlayerState { .. } => "SetPlayerState",
        StateOp::RemovePlayerState { .. } => "RemovePlayerState",
        StateOp::SetCharacterState { .. } => "SetCharacterState",
        StateOp::RemoveCharacterState { .. } => "RemoveCharacterState",
    }
}

fn reply_option_payload_from_reply_option(reply: ReplyOption) -> protocol::ReplyOptionPayload {
    protocol::ReplyOptionPayload {
        reply_id: reply.id,
        text: reply.text,
    }
}
