use async_trait::async_trait;

use crate::config::AgentApiIds;
use crate::error::StoreError;
use crate::record::{
    CharacterCardRecord, DefaultLlmConfigRecord, LlmApiRecord, PlayerProfileRecord, SchemaRecord,
    SessionMessageRecord, SessionRecord, StoryDraftRecord, StoryRecord, StoryResourcesRecord,
};

#[async_trait]
pub trait Store: Send + Sync {
    async fn get_global_config(&self) -> Result<Option<AgentApiIds>, StoreError>;
    async fn set_global_config(&self, config: AgentApiIds) -> Result<(), StoreError>;

    async fn get_default_llm_config(&self) -> Result<Option<DefaultLlmConfigRecord>, StoreError>;
    async fn set_default_llm_config(
        &self,
        config: DefaultLlmConfigRecord,
    ) -> Result<(), StoreError>;

    async fn get_llm_api(&self, api_id: &str) -> Result<Option<LlmApiRecord>, StoreError>;
    async fn list_llm_apis(&self) -> Result<Vec<LlmApiRecord>, StoreError>;
    async fn save_llm_api(&self, record: LlmApiRecord) -> Result<(), StoreError>;
    async fn delete_llm_api(&self, api_id: &str) -> Result<Option<LlmApiRecord>, StoreError>;

    async fn get_schema(&self, schema_id: &str) -> Result<Option<SchemaRecord>, StoreError>;
    async fn list_schemas(&self) -> Result<Vec<SchemaRecord>, StoreError>;
    async fn save_schema(&self, record: SchemaRecord) -> Result<(), StoreError>;
    async fn delete_schema(&self, schema_id: &str) -> Result<Option<SchemaRecord>, StoreError>;

    async fn get_player_profile(
        &self,
        player_profile_id: &str,
    ) -> Result<Option<PlayerProfileRecord>, StoreError>;
    async fn list_player_profiles(&self) -> Result<Vec<PlayerProfileRecord>, StoreError>;
    async fn save_player_profile(&self, record: PlayerProfileRecord) -> Result<(), StoreError>;
    async fn delete_player_profile(
        &self,
        player_profile_id: &str,
    ) -> Result<Option<PlayerProfileRecord>, StoreError>;

    async fn get_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError>;
    async fn list_characters(&self) -> Result<Vec<CharacterCardRecord>, StoreError>;
    async fn save_character(&self, record: CharacterCardRecord) -> Result<(), StoreError>;
    async fn delete_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError>;

    async fn get_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesRecord>, StoreError>;
    async fn list_story_resources(&self) -> Result<Vec<StoryResourcesRecord>, StoreError>;
    async fn save_story_resources(&self, resources: StoryResourcesRecord)
    -> Result<(), StoreError>;
    async fn delete_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesRecord>, StoreError>;

    async fn get_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError>;
    async fn list_stories(&self) -> Result<Vec<StoryRecord>, StoreError>;
    async fn save_story(&self, story: StoryRecord) -> Result<(), StoreError>;
    async fn delete_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError>;

    async fn get_story_draft(&self, draft_id: &str)
    -> Result<Option<StoryDraftRecord>, StoreError>;
    async fn list_story_drafts(&self) -> Result<Vec<StoryDraftRecord>, StoreError>;
    async fn save_story_draft(&self, draft: StoryDraftRecord) -> Result<(), StoreError>;
    async fn delete_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<Option<StoryDraftRecord>, StoreError>;

    async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError>;
    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, StoreError>;
    async fn save_session(&self, session: SessionRecord) -> Result<(), StoreError>;
    async fn delete_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError>;

    async fn get_session_message(
        &self,
        message_id: &str,
    ) -> Result<Option<SessionMessageRecord>, StoreError>;
    async fn list_session_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionMessageRecord>, StoreError>;
    async fn save_session_message(&self, message: SessionMessageRecord) -> Result<(), StoreError>;
    async fn delete_session_message(
        &self,
        message_id: &str,
    ) -> Result<Option<SessionMessageRecord>, StoreError>;
}
