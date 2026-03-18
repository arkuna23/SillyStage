use async_stream::stream;
use futures_util::StreamExt;
use store::{RuntimeSnapshot, SessionRecord};

use crate::{Engine, EngineEvent, RuntimeApiRecords};

use super::util::{build_session_messages, next_session_message_sequence, now_timestamp_ms};
use super::{EngineManager, ManagedTurnStream, ManagerError, ResolvedSessionConfig};

impl EngineManager {
    pub async fn start_session_from_story(
        &self,
        story_id: &str,
        display_name: Option<String>,
        player_profile_id: Option<String>,
        api_group_id: Option<String>,
        preset_id: Option<String>,
    ) -> Result<SessionRecord, ManagerError> {
        let story = self
            .store
            .get_story(story_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStory(story_id.to_owned()))?;
        let (_api_group, _preset, binding) = self
            .resolve_api_group_and_preset(api_group_id.as_deref(), preset_id.as_deref())
            .await?;

        let (player_name, player_description) = self
            .resolve_player_identity(player_profile_id.as_deref())
            .await?;
        let runtime_state = self
            .build_runtime_state_from_story(&story, player_name, player_description)
            .await?;
        let session_id = format!("session-{}", self.store.list_sessions().await?.len());
        let now = now_timestamp_ms();
        let session = SessionRecord {
            session_id,
            display_name: display_name.unwrap_or_else(|| story.display_name.clone()),
            story_id: story.story_id,
            player_profile_id,
            player_schema_id: story.player_schema_id,
            binding,
            snapshot: runtime_state.snapshot(),
            created_at_ms: Some(now),
            updated_at_ms: Some(now),
        };

        self.store.save_session(session.clone()).await?;
        Ok(session)
    }

    pub async fn get_runtime_snapshot(
        &self,
        session_id: &str,
    ) -> Result<RuntimeSnapshot, ManagerError> {
        self.store
            .get_session(session_id)
            .await?
            .map(|session| session.snapshot)
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))
    }

    pub async fn update_player_description(
        &self,
        session_id: &str,
        player_description: String,
    ) -> Result<RuntimeSnapshot, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session.player_profile_id = None;
        session.snapshot.player_description = player_description;
        session.updated_at_ms = Some(now_timestamp_ms());
        let snapshot = session.snapshot.clone();
        self.store.save_session(session).await?;
        Ok(snapshot)
    }

    pub async fn set_player_profile(
        &self,
        session_id: &str,
        player_profile_id: Option<String>,
    ) -> Result<SessionRecord, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;

        let (_player_name, player_description) = self
            .resolve_player_identity(player_profile_id.as_deref())
            .await?;
        session.player_profile_id = player_profile_id;
        session.snapshot.player_description = player_description;
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;
        Ok(session)
    }

    pub async fn get_resolved_session_config(
        &self,
        session_id: &str,
    ) -> Result<ResolvedSessionConfig, ManagerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        Ok(ResolvedSessionConfig {
            binding: session.binding,
        })
    }

    pub async fn update_session_config(
        &self,
        session_id: &str,
        api_group_id: Option<String>,
        preset_id: Option<String>,
    ) -> Result<ResolvedSessionConfig, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let binding = self
            .resolve_api_group_and_preset(
                api_group_id
                    .as_deref()
                    .or(Some(session.binding.api_group_id.as_str())),
                preset_id
                    .as_deref()
                    .or(Some(session.binding.preset_id.as_str())),
            )
            .await?
            .2;
        session.binding = binding.clone();
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session).await?;

        Ok(ResolvedSessionConfig { binding })
    }

    pub async fn run_turn_stream(
        &self,
        session_id: &str,
        player_input: String,
    ) -> Result<ManagedTurnStream<'static>, ManagerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let story = self
            .store
            .get_story(&session.story_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStory(session.story_id.clone()))?;
        let runtime_state = self
            .build_runtime_state_from_session(&story, &session)
            .await?;
        let api_group = self
            .resolve_api_group(&session.binding.api_group_id)
            .await?;
        let preset = self.resolve_preset(&session.binding.preset_id).await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let runtime_configs = self.registry.build_runtime_configs(
            RuntimeApiRecords {
                director: &apis.director,
                actor: &apis.actor,
                narrator: &apis.narrator,
                keeper: &apis.keeper,
            },
            &preset,
        )?;
        let mut engine = Engine::new(runtime_configs, runtime_state)?;
        let store = std::sync::Arc::clone(&self.store);
        let session_record = session.clone();

        let stream = stream! {
            let mut updated_session = session_record.clone();
            let mut engine_stream = match engine.run_turn_stream(&player_input).await {
                Ok(stream) => stream,
                Err(error) => {
                    yield Err(ManagerError::Engine(error));
                    return;
                }
            };

            while let Some(event) = engine_stream.next().await {
                match &event {
                    EngineEvent::SessionCharacterCreated { character, snapshot } => {
                        let mut record = character.clone();
                        record.session_id = session_record.session_id.clone();
                        if let Err(error) = store.save_session_character(record).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                    }
                    EngineEvent::SessionCharacterEnteredScene { snapshot, .. }
                    | EngineEvent::SessionCharacterLeftScene { snapshot, .. } => {
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                    }
                    EngineEvent::TurnCompleted { result } => {
                        updated_session.snapshot = result.snapshot.clone();
                        let recorded_at_ms = now_timestamp_ms();
                        updated_session.updated_at_ms = Some(recorded_at_ms);
                        if let Err(error) = store.save_session(updated_session.clone()).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                        let messages = build_session_messages(
                            &session_record.session_id,
                            &session_record,
                            result,
                            recorded_at_ms,
                            match store.list_session_messages(&session_record.session_id).await {
                                Ok(existing) => next_session_message_sequence(&existing),
                                Err(error) => {
                                    yield Err(ManagerError::Store(error));
                                    return;
                                }
                            },
                        );
                        for message in messages {
                            if let Err(error) = store.save_session_message(message).await {
                                yield Err(ManagerError::Store(error));
                                return;
                            }
                        }
                    }
                    EngineEvent::TurnFailed { snapshot, .. } => {
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                        if let Err(error) = store.save_session(updated_session.clone()).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                    }
                    _ => {}
                }

                yield Ok(event);
            }
        };

        Ok(Box::pin(stream))
    }
}
