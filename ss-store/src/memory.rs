use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::error::StoreError;
use crate::record::{
    ApiGroupRecord, ApiRecord, CharacterCardRecord, PlayerProfileRecord, PresetRecord,
    SchemaRecord, SessionMessageRecord, SessionRecord, StoryDraftRecord, StoryRecord,
    StoryResourcesRecord,
};
use crate::store::Store;

#[derive(Default)]
pub struct InMemoryStore {
    apis: RwLock<HashMap<String, ApiRecord>>,
    api_groups: RwLock<HashMap<String, ApiGroupRecord>>,
    presets: RwLock<HashMap<String, PresetRecord>>,
    schemas: RwLock<HashMap<String, SchemaRecord>>,
    player_profiles: RwLock<HashMap<String, PlayerProfileRecord>>,
    characters: RwLock<HashMap<String, CharacterCardRecord>>,
    story_resources: RwLock<HashMap<String, StoryResourcesRecord>>,
    stories: RwLock<HashMap<String, StoryRecord>>,
    story_drafts: RwLock<HashMap<String, StoryDraftRecord>>,
    sessions: RwLock<HashMap<String, SessionRecord>>,
    session_messages: RwLock<HashMap<String, SessionMessageRecord>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Store for InMemoryStore {
    async fn get_api(&self, api_id: &str) -> Result<Option<ApiRecord>, StoreError> {
        Ok(self.apis.read().await.get(api_id).cloned())
    }

    async fn list_apis(&self) -> Result<Vec<ApiRecord>, StoreError> {
        Ok(self.apis.read().await.values().cloned().collect())
    }

    async fn save_api(&self, record: ApiRecord) -> Result<(), StoreError> {
        self.apis
            .write()
            .await
            .insert(record.api_id.clone(), record);
        Ok(())
    }

    async fn delete_api(&self, api_id: &str) -> Result<Option<ApiRecord>, StoreError> {
        Ok(self.apis.write().await.remove(api_id))
    }

    async fn get_api_group(
        &self,
        api_group_id: &str,
    ) -> Result<Option<ApiGroupRecord>, StoreError> {
        Ok(self.api_groups.read().await.get(api_group_id).cloned())
    }

    async fn list_api_groups(&self) -> Result<Vec<ApiGroupRecord>, StoreError> {
        Ok(self.api_groups.read().await.values().cloned().collect())
    }

    async fn save_api_group(&self, record: ApiGroupRecord) -> Result<(), StoreError> {
        self.api_groups
            .write()
            .await
            .insert(record.api_group_id.clone(), record);
        Ok(())
    }

    async fn delete_api_group(
        &self,
        api_group_id: &str,
    ) -> Result<Option<ApiGroupRecord>, StoreError> {
        Ok(self.api_groups.write().await.remove(api_group_id))
    }

    async fn get_preset(&self, preset_id: &str) -> Result<Option<PresetRecord>, StoreError> {
        Ok(self.presets.read().await.get(preset_id).cloned())
    }

    async fn list_presets(&self) -> Result<Vec<PresetRecord>, StoreError> {
        Ok(self.presets.read().await.values().cloned().collect())
    }

    async fn save_preset(&self, record: PresetRecord) -> Result<(), StoreError> {
        self.presets
            .write()
            .await
            .insert(record.preset_id.clone(), record);
        Ok(())
    }

    async fn delete_preset(&self, preset_id: &str) -> Result<Option<PresetRecord>, StoreError> {
        Ok(self.presets.write().await.remove(preset_id))
    }

    async fn get_schema(&self, schema_id: &str) -> Result<Option<SchemaRecord>, StoreError> {
        Ok(self.schemas.read().await.get(schema_id).cloned())
    }

    async fn list_schemas(&self) -> Result<Vec<SchemaRecord>, StoreError> {
        Ok(self.schemas.read().await.values().cloned().collect())
    }

    async fn save_schema(&self, record: SchemaRecord) -> Result<(), StoreError> {
        self.schemas
            .write()
            .await
            .insert(record.schema_id.clone(), record);
        Ok(())
    }

    async fn delete_schema(&self, schema_id: &str) -> Result<Option<SchemaRecord>, StoreError> {
        Ok(self.schemas.write().await.remove(schema_id))
    }

    async fn get_player_profile(
        &self,
        player_profile_id: &str,
    ) -> Result<Option<PlayerProfileRecord>, StoreError> {
        Ok(self
            .player_profiles
            .read()
            .await
            .get(player_profile_id)
            .cloned())
    }

    async fn list_player_profiles(&self) -> Result<Vec<PlayerProfileRecord>, StoreError> {
        Ok(self
            .player_profiles
            .read()
            .await
            .values()
            .cloned()
            .collect())
    }

    async fn save_player_profile(&self, record: PlayerProfileRecord) -> Result<(), StoreError> {
        self.player_profiles
            .write()
            .await
            .insert(record.player_profile_id.clone(), record);
        Ok(())
    }

    async fn delete_player_profile(
        &self,
        player_profile_id: &str,
    ) -> Result<Option<PlayerProfileRecord>, StoreError> {
        Ok(self.player_profiles.write().await.remove(player_profile_id))
    }

    async fn get_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError> {
        Ok(self.characters.read().await.get(character_id).cloned())
    }

    async fn list_characters(&self) -> Result<Vec<CharacterCardRecord>, StoreError> {
        Ok(self.characters.read().await.values().cloned().collect())
    }

    async fn save_character(&self, record: CharacterCardRecord) -> Result<(), StoreError> {
        self.characters
            .write()
            .await
            .insert(record.character_id.clone(), record);
        Ok(())
    }

    async fn delete_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError> {
        Ok(self.characters.write().await.remove(character_id))
    }

    async fn get_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesRecord>, StoreError> {
        Ok(self.story_resources.read().await.get(resource_id).cloned())
    }

    async fn list_story_resources(&self) -> Result<Vec<StoryResourcesRecord>, StoreError> {
        Ok(self
            .story_resources
            .read()
            .await
            .values()
            .cloned()
            .collect())
    }

    async fn save_story_resources(
        &self,
        resources: StoryResourcesRecord,
    ) -> Result<(), StoreError> {
        self.story_resources
            .write()
            .await
            .insert(resources.resource_id.clone(), resources);
        Ok(())
    }

    async fn delete_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesRecord>, StoreError> {
        Ok(self.story_resources.write().await.remove(resource_id))
    }

    async fn get_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError> {
        Ok(self.stories.read().await.get(story_id).cloned())
    }

    async fn list_stories(&self) -> Result<Vec<StoryRecord>, StoreError> {
        Ok(self.stories.read().await.values().cloned().collect())
    }

    async fn save_story(&self, story: StoryRecord) -> Result<(), StoreError> {
        self.stories
            .write()
            .await
            .insert(story.story_id.clone(), story);
        Ok(())
    }

    async fn delete_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError> {
        Ok(self.stories.write().await.remove(story_id))
    }

    async fn get_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<Option<StoryDraftRecord>, StoreError> {
        Ok(self.story_drafts.read().await.get(draft_id).cloned())
    }

    async fn list_story_drafts(&self) -> Result<Vec<StoryDraftRecord>, StoreError> {
        Ok(self.story_drafts.read().await.values().cloned().collect())
    }

    async fn save_story_draft(&self, draft: StoryDraftRecord) -> Result<(), StoreError> {
        self.story_drafts
            .write()
            .await
            .insert(draft.draft_id.clone(), draft);
        Ok(())
    }

    async fn delete_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<Option<StoryDraftRecord>, StoreError> {
        Ok(self.story_drafts.write().await.remove(draft_id))
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError> {
        Ok(self.sessions.read().await.get(session_id).cloned())
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, StoreError> {
        Ok(self.sessions.read().await.values().cloned().collect())
    }

    async fn save_session(&self, session: SessionRecord) -> Result<(), StoreError> {
        self.sessions
            .write()
            .await
            .insert(session.session_id.clone(), session);
        Ok(())
    }

    async fn delete_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError> {
        Ok(self.sessions.write().await.remove(session_id))
    }

    async fn get_session_message(
        &self,
        message_id: &str,
    ) -> Result<Option<SessionMessageRecord>, StoreError> {
        Ok(self.session_messages.read().await.get(message_id).cloned())
    }

    async fn list_session_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionMessageRecord>, StoreError> {
        Ok(self
            .session_messages
            .read()
            .await
            .values()
            .filter(|message| message.session_id == session_id)
            .cloned()
            .collect())
    }

    async fn save_session_message(&self, message: SessionMessageRecord) -> Result<(), StoreError> {
        self.session_messages
            .write()
            .await
            .insert(message.message_id.clone(), message);
        Ok(())
    }

    async fn delete_session_message(
        &self,
        message_id: &str,
    ) -> Result<Option<SessionMessageRecord>, StoreError> {
        Ok(self.session_messages.write().await.remove(message_id))
    }
}
