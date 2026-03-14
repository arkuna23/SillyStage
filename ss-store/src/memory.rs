use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::config::AgentApiIds;
use crate::error::StoreError;
use crate::record::{
    CharacterCardRecord, LlmApiRecord, PlayerProfileRecord, SchemaRecord, SessionRecord,
    StoryRecord, StoryResourcesRecord,
};
use crate::store::Store;

#[derive(Default)]
pub struct InMemoryStore {
    global_config: RwLock<Option<AgentApiIds>>,
    llm_apis: RwLock<HashMap<String, LlmApiRecord>>,
    schemas: RwLock<HashMap<String, SchemaRecord>>,
    player_profiles: RwLock<HashMap<String, PlayerProfileRecord>>,
    characters: RwLock<HashMap<String, CharacterCardRecord>>,
    story_resources: RwLock<HashMap<String, StoryResourcesRecord>>,
    stories: RwLock<HashMap<String, StoryRecord>>,
    sessions: RwLock<HashMap<String, SessionRecord>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl Store for InMemoryStore {
    async fn get_global_config(&self) -> Result<Option<AgentApiIds>, StoreError> {
        Ok(self.global_config.read().await.clone())
    }

    async fn set_global_config(&self, config: AgentApiIds) -> Result<(), StoreError> {
        *self.global_config.write().await = Some(config);
        Ok(())
    }

    async fn get_llm_api(&self, api_id: &str) -> Result<Option<LlmApiRecord>, StoreError> {
        Ok(self.llm_apis.read().await.get(api_id).cloned())
    }

    async fn list_llm_apis(&self) -> Result<Vec<LlmApiRecord>, StoreError> {
        Ok(self.llm_apis.read().await.values().cloned().collect())
    }

    async fn save_llm_api(&self, record: LlmApiRecord) -> Result<(), StoreError> {
        self.llm_apis
            .write()
            .await
            .insert(record.api_id.clone(), record);
        Ok(())
    }

    async fn delete_llm_api(&self, api_id: &str) -> Result<Option<LlmApiRecord>, StoreError> {
        Ok(self.llm_apis.write().await.remove(api_id))
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
}
