use store::SessionCharacterRecord;

use super::util::{ensure_session_character_belongs, now_timestamp_ms};
use super::{EngineManager, ManagerError, SessionCharacterUpdate};

impl EngineManager {
    pub async fn get_session_character(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<SessionCharacterRecord, ManagerError> {
        self.store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let character = self
            .store
            .get_session_character(session_character_id)
            .await?
            .ok_or_else(|| {
                ManagerError::MissingSessionCharacter(session_character_id.to_owned())
            })?;
        ensure_session_character_belongs(session_id, &character)?;
        Ok(character)
    }

    pub async fn list_session_characters(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionCharacterRecord>, ManagerError> {
        self.store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let mut characters = self.store.list_session_characters(session_id).await?;
        characters.sort_by(|left, right| {
            left.created_at_ms
                .cmp(&right.created_at_ms)
                .then_with(|| left.session_character_id.cmp(&right.session_character_id))
        });
        Ok(characters)
    }

    pub async fn update_session_character(
        &self,
        session_id: &str,
        session_character_id: &str,
        update: SessionCharacterUpdate,
    ) -> Result<SessionCharacterRecord, ManagerError> {
        let mut character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        character.display_name = update.display_name;
        character.personality = update.personality;
        character.style = update.style;
        character.system_prompt = update.system_prompt;
        character.updated_at_ms = now_timestamp_ms();
        self.store.save_session_character(character.clone()).await?;
        Ok(character)
    }

    pub async fn delete_session_character(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<SessionCharacterRecord, ManagerError> {
        let character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session
            .snapshot
            .world_state
            .remove_active_character(session_character_id);
        session
            .snapshot
            .world_state
            .character_state
            .remove(session_character_id);
        session
            .snapshot
            .world_state
            .actor_private_memory
            .remove(session_character_id);
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store
            .delete_session_character(session_character_id)
            .await?
            .ok_or_else(|| {
                ManagerError::MissingSessionCharacter(session_character_id.to_owned())
            })?;
        self.store.save_session(session).await?;
        Ok(character)
    }

    pub async fn enter_session_character_scene(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<(store::SessionRecord, SessionCharacterRecord), ManagerError> {
        let character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session
            .snapshot
            .world_state
            .add_active_character(session_character_id.to_owned());
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;
        Ok((session, character))
    }

    pub async fn leave_session_character_scene(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<(store::SessionRecord, SessionCharacterRecord), ManagerError> {
        let character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session
            .snapshot
            .world_state
            .remove_active_character(session_character_id);
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;
        Ok((session, character))
    }
}
