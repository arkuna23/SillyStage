use std::collections::HashMap;

use async_trait::async_trait;
use engine::{AgentApiIds, RuntimeSnapshot, SessionEngineConfig};
use protocol::{
    CharacterArchive, CharacterCardSummaryPayload, StoryGeneratedPayload, StoryResourcesPayload,
    UploadTargetKind,
};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct UploadRecord {
    pub upload_id: String,
    pub target_kind: UploadTargetKind,
    pub file_name: String,
    pub content_type: String,
    pub total_size: u64,
    pub sha256: String,
    pub next_chunk_index: u64,
    pub next_offset: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct CharacterCardRecord {
    pub character_id: String,
    pub archive: CharacterArchive,
    pub summary: CharacterCardSummaryPayload,
}

#[derive(Debug, Clone)]
pub struct StoryRecord {
    pub story_id: String,
    pub resource_id: String,
    pub generated: StoryGeneratedPayload,
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub session_id: String,
    pub story_id: String,
    pub snapshot: RuntimeSnapshot,
    pub config: SessionEngineConfig,
}

#[async_trait]
pub trait HandlerStore: Send + Sync {
    async fn get_global_config(&self) -> Result<Option<AgentApiIds>, StoreError>;
    async fn set_global_config(&self, config: AgentApiIds) -> Result<(), StoreError>;

    async fn get_upload(&self, upload_id: &str) -> Result<Option<UploadRecord>, StoreError>;
    async fn save_upload(&self, upload: UploadRecord) -> Result<(), StoreError>;
    async fn delete_upload(&self, upload_id: &str) -> Result<(), StoreError>;

    async fn get_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError>;
    async fn save_character(&self, record: CharacterCardRecord) -> Result<(), StoreError>;

    async fn get_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesPayload>, StoreError>;
    async fn save_story_resources(
        &self,
        resources: StoryResourcesPayload,
    ) -> Result<(), StoreError>;

    async fn get_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError>;
    async fn save_story(&self, story: StoryRecord) -> Result<(), StoreError>;

    async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError>;
    async fn save_session(&self, session: SessionRecord) -> Result<(), StoreError>;
}

#[derive(Default)]
pub struct InMemoryHandlerStore {
    global_config: RwLock<Option<AgentApiIds>>,
    uploads: RwLock<HashMap<String, UploadRecord>>,
    characters: RwLock<HashMap<String, CharacterCardRecord>>,
    story_resources: RwLock<HashMap<String, StoryResourcesPayload>>,
    stories: RwLock<HashMap<String, StoryRecord>>,
    sessions: RwLock<HashMap<String, SessionRecord>>,
}

impl InMemoryHandlerStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl HandlerStore for InMemoryHandlerStore {
    async fn get_global_config(&self) -> Result<Option<AgentApiIds>, StoreError> {
        Ok(self.global_config.read().await.clone())
    }

    async fn set_global_config(&self, config: AgentApiIds) -> Result<(), StoreError> {
        *self.global_config.write().await = Some(config);
        Ok(())
    }

    async fn get_upload(&self, upload_id: &str) -> Result<Option<UploadRecord>, StoreError> {
        Ok(self.uploads.read().await.get(upload_id).cloned())
    }

    async fn save_upload(&self, upload: UploadRecord) -> Result<(), StoreError> {
        self.uploads
            .write()
            .await
            .insert(upload.upload_id.clone(), upload);
        Ok(())
    }

    async fn delete_upload(&self, upload_id: &str) -> Result<(), StoreError> {
        self.uploads.write().await.remove(upload_id);
        Ok(())
    }

    async fn get_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError> {
        Ok(self.characters.read().await.get(character_id).cloned())
    }

    async fn save_character(&self, record: CharacterCardRecord) -> Result<(), StoreError> {
        self.characters
            .write()
            .await
            .insert(record.character_id.clone(), record);
        Ok(())
    }

    async fn get_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesPayload>, StoreError> {
        Ok(self.story_resources.read().await.get(resource_id).cloned())
    }

    async fn save_story_resources(
        &self,
        resources: StoryResourcesPayload,
    ) -> Result<(), StoreError> {
        self.story_resources
            .write()
            .await
            .insert(resources.resource_id.clone(), resources);
        Ok(())
    }

    async fn get_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError> {
        Ok(self.stories.read().await.get(story_id).cloned())
    }

    async fn save_story(&self, story: StoryRecord) -> Result<(), StoreError> {
        self.stories
            .write()
            .await
            .insert(story.story_id.clone(), story);
        Ok(())
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError> {
        Ok(self.sessions.read().await.get(session_id).cloned())
    }

    async fn save_session(&self, session: SessionRecord) -> Result<(), StoreError> {
        self.sessions
            .write()
            .await
            .insert(session.session_id.clone(), session);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("store backend error: {0}")]
    Backend(String),
}
