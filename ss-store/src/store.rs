use async_trait::async_trait;

use crate::error::StoreError;
use crate::record::{
    ApiGroupRecord, ApiRecord, CharacterCardRecord, PlayerProfileRecord, PresetRecord,
    SchemaRecord, SessionMessageRecord, SessionRecord, StoryDraftRecord, StoryRecord,
    StoryResourcesRecord,
};

#[async_trait]
pub trait Store: Send + Sync {
    async fn get_api(&self, api_id: &str) -> Result<Option<ApiRecord>, StoreError>;
    async fn list_apis(&self) -> Result<Vec<ApiRecord>, StoreError>;
    async fn save_api(&self, record: ApiRecord) -> Result<(), StoreError>;
    async fn delete_api(&self, api_id: &str) -> Result<Option<ApiRecord>, StoreError>;

    async fn get_api_group(&self, api_group_id: &str)
    -> Result<Option<ApiGroupRecord>, StoreError>;
    async fn list_api_groups(&self) -> Result<Vec<ApiGroupRecord>, StoreError>;
    async fn save_api_group(&self, record: ApiGroupRecord) -> Result<(), StoreError>;
    async fn delete_api_group(
        &self,
        api_group_id: &str,
    ) -> Result<Option<ApiGroupRecord>, StoreError>;

    async fn get_preset(&self, preset_id: &str) -> Result<Option<PresetRecord>, StoreError>;
    async fn list_presets(&self) -> Result<Vec<PresetRecord>, StoreError>;
    async fn save_preset(&self, record: PresetRecord) -> Result<(), StoreError>;
    async fn delete_preset(&self, preset_id: &str) -> Result<Option<PresetRecord>, StoreError>;

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
