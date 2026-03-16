use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::StoreError;
use crate::record::{
    ApiGroupRecord, ApiRecord, CharacterCardDefinition, CharacterCardRecord, PlayerProfileRecord,
    PresetRecord, SchemaRecord, SessionCharacterRecord, SessionMessageRecord, SessionRecord,
    StoryDraftRecord, StoryRecord, StoryResourcesRecord,
};
use crate::store::Store;

const GLOBAL_DIR: &str = "global";
const APIS_DIR: &str = "apis";
const API_GROUPS_DIR: &str = "api_groups";
const PRESETS_DIR: &str = "presets";
const SCHEMAS_DIR: &str = "schemas";
const PLAYER_PROFILES_DIR: &str = "player_profiles";
const CHARACTERS_DIR: &str = "characters";
const CHARACTER_RECORD_FILE: &str = "record.json";
const CHARACTER_COVER_FILE: &str = "cover.bin";
const STORY_RESOURCES_DIR: &str = "story_resources";
const STORIES_DIR: &str = "stories";
const STORY_DRAFTS_DIR: &str = "story_drafts";
const SESSIONS_DIR: &str = "sessions";
const SESSION_CHARACTERS_DIR: &str = "session_characters";
const SESSION_MESSAGES_DIR: &str = "session_messages";

#[derive(Debug, Clone)]
pub struct FileSystemStore {
    root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CharacterCardRecordFile {
    character_id: String,
    content: CharacterCardDefinition,
    cover_file_name: Option<String>,
    cover_mime_type: Option<String>,
}

impl From<&CharacterCardRecord> for CharacterCardRecordFile {
    fn from(value: &CharacterCardRecord) -> Self {
        Self {
            character_id: value.character_id.clone(),
            content: value.content.clone(),
            cover_file_name: value.cover_file_name.clone(),
            cover_mime_type: value.cover_mime_type.clone(),
        }
    }
}

impl FileSystemStore {
    pub async fn new(root: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let store = Self { root: root.into() };
        store.ensure_layout().await?;
        Ok(store)
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    async fn ensure_layout(&self) -> Result<(), StoreError> {
        fs::create_dir_all(self.global_dir()).await?;
        fs::create_dir_all(self.apis_dir()).await?;
        fs::create_dir_all(self.api_groups_dir()).await?;
        fs::create_dir_all(self.presets_dir()).await?;
        fs::create_dir_all(self.schemas_dir()).await?;
        fs::create_dir_all(self.player_profiles_dir()).await?;
        fs::create_dir_all(self.characters_dir()).await?;
        fs::create_dir_all(self.story_resources_dir()).await?;
        fs::create_dir_all(self.stories_dir()).await?;
        fs::create_dir_all(self.story_drafts_dir()).await?;
        fs::create_dir_all(self.sessions_dir()).await?;
        fs::create_dir_all(self.session_characters_dir()).await?;
        fs::create_dir_all(self.session_messages_dir()).await?;
        Ok(())
    }

    fn global_dir(&self) -> PathBuf {
        self.root.join(GLOBAL_DIR)
    }

    fn api_groups_dir(&self) -> PathBuf {
        self.root.join(API_GROUPS_DIR)
    }

    fn apis_dir(&self) -> PathBuf {
        self.root.join(APIS_DIR)
    }

    fn api_path(&self, api_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .apis_dir()
            .join(format!("{}.json", validate_path_component(api_id)?)))
    }

    fn api_group_path(&self, api_group_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .api_groups_dir()
            .join(format!("{}.json", validate_path_component(api_group_id)?)))
    }

    fn presets_dir(&self) -> PathBuf {
        self.root.join(PRESETS_DIR)
    }

    fn preset_path(&self, preset_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .presets_dir()
            .join(format!("{}.json", validate_path_component(preset_id)?)))
    }

    fn schemas_dir(&self) -> PathBuf {
        self.root.join(SCHEMAS_DIR)
    }

    fn schema_path(&self, schema_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .schemas_dir()
            .join(format!("{}.json", validate_path_component(schema_id)?)))
    }

    fn player_profiles_dir(&self) -> PathBuf {
        self.root.join(PLAYER_PROFILES_DIR)
    }

    fn player_profile_path(&self, player_profile_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self.player_profiles_dir().join(format!(
            "{}.json",
            validate_path_component(player_profile_id)?
        )))
    }

    fn characters_dir(&self) -> PathBuf {
        self.root.join(CHARACTERS_DIR)
    }

    fn character_dir(&self, character_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .characters_dir()
            .join(validate_path_component(character_id)?))
    }

    fn character_record_path(&self, character_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .character_dir(character_id)?
            .join(CHARACTER_RECORD_FILE))
    }

    fn character_cover_path(&self, character_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self.character_dir(character_id)?.join(CHARACTER_COVER_FILE))
    }

    fn story_resources_dir(&self) -> PathBuf {
        self.root.join(STORY_RESOURCES_DIR)
    }

    fn story_resources_path(&self, resource_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .story_resources_dir()
            .join(format!("{}.json", validate_path_component(resource_id)?)))
    }

    fn stories_dir(&self) -> PathBuf {
        self.root.join(STORIES_DIR)
    }

    fn story_path(&self, story_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .stories_dir()
            .join(format!("{}.json", validate_path_component(story_id)?)))
    }

    fn story_drafts_dir(&self) -> PathBuf {
        self.root.join(STORY_DRAFTS_DIR)
    }

    fn story_draft_path(&self, draft_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .story_drafts_dir()
            .join(format!("{}.json", validate_path_component(draft_id)?)))
    }

    fn sessions_dir(&self) -> PathBuf {
        self.root.join(SESSIONS_DIR)
    }

    fn session_path(&self, session_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .sessions_dir()
            .join(format!("{}.json", validate_path_component(session_id)?)))
    }

    fn session_messages_dir(&self) -> PathBuf {
        self.root.join(SESSION_MESSAGES_DIR)
    }

    fn session_characters_dir(&self) -> PathBuf {
        self.root.join(SESSION_CHARACTERS_DIR)
    }

    fn session_character_path(&self, session_character_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self.session_characters_dir().join(format!(
            "{}.json",
            validate_path_component(session_character_id)?
        )))
    }

    fn session_message_path(&self, message_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .session_messages_dir()
            .join(format!("{}.json", validate_path_component(message_id)?)))
    }
}

#[async_trait]
impl Store for FileSystemStore {
    async fn get_api(&self, api_id: &str) -> Result<Option<ApiRecord>, StoreError> {
        read_optional_json_file(&self.api_path(api_id)?).await
    }

    async fn list_apis(&self) -> Result<Vec<ApiRecord>, StoreError> {
        list_json_records(&self.apis_dir()).await
    }

    async fn save_api(&self, record: ApiRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.api_path(&record.api_id)?, &record).await
    }

    async fn delete_api(&self, api_id: &str) -> Result<Option<ApiRecord>, StoreError> {
        delete_optional_json_file(&self.api_path(api_id)?).await
    }

    async fn get_api_group(
        &self,
        api_group_id: &str,
    ) -> Result<Option<ApiGroupRecord>, StoreError> {
        read_optional_json_file(&self.api_group_path(api_group_id)?).await
    }

    async fn list_api_groups(&self) -> Result<Vec<ApiGroupRecord>, StoreError> {
        list_json_records(&self.api_groups_dir()).await
    }

    async fn save_api_group(&self, record: ApiGroupRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.api_group_path(&record.api_group_id)?, &record).await
    }

    async fn delete_api_group(
        &self,
        api_group_id: &str,
    ) -> Result<Option<ApiGroupRecord>, StoreError> {
        delete_optional_json_file(&self.api_group_path(api_group_id)?).await
    }

    async fn get_preset(&self, preset_id: &str) -> Result<Option<PresetRecord>, StoreError> {
        read_optional_json_file(&self.preset_path(preset_id)?).await
    }

    async fn list_presets(&self) -> Result<Vec<PresetRecord>, StoreError> {
        list_json_records(&self.presets_dir()).await
    }

    async fn save_preset(&self, record: PresetRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.preset_path(&record.preset_id)?, &record).await
    }

    async fn delete_preset(&self, preset_id: &str) -> Result<Option<PresetRecord>, StoreError> {
        delete_optional_json_file(&self.preset_path(preset_id)?).await
    }

    async fn get_schema(&self, schema_id: &str) -> Result<Option<SchemaRecord>, StoreError> {
        read_optional_json_file(&self.schema_path(schema_id)?).await
    }

    async fn list_schemas(&self) -> Result<Vec<SchemaRecord>, StoreError> {
        list_json_records(&self.schemas_dir()).await
    }

    async fn save_schema(&self, record: SchemaRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.schema_path(&record.schema_id)?, &record).await
    }

    async fn delete_schema(&self, schema_id: &str) -> Result<Option<SchemaRecord>, StoreError> {
        delete_optional_json_file(&self.schema_path(schema_id)?).await
    }

    async fn get_player_profile(
        &self,
        player_profile_id: &str,
    ) -> Result<Option<PlayerProfileRecord>, StoreError> {
        read_optional_json_file(&self.player_profile_path(player_profile_id)?).await
    }

    async fn list_player_profiles(&self) -> Result<Vec<PlayerProfileRecord>, StoreError> {
        list_json_records(&self.player_profiles_dir()).await
    }

    async fn save_player_profile(&self, record: PlayerProfileRecord) -> Result<(), StoreError> {
        write_json_atomic(
            &self.player_profile_path(&record.player_profile_id)?,
            &record,
        )
        .await
    }

    async fn delete_player_profile(
        &self,
        player_profile_id: &str,
    ) -> Result<Option<PlayerProfileRecord>, StoreError> {
        delete_optional_json_file(&self.player_profile_path(player_profile_id)?).await
    }

    async fn get_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError> {
        let record_path = self.character_record_path(character_id)?;
        if !path_exists(&record_path).await? {
            return Ok(None);
        }

        let record: CharacterCardRecordFile = read_json_file(&record_path).await?;
        let cover_bytes = if record.cover_file_name.is_some() {
            Some(fs::read(self.character_cover_path(character_id)?).await?)
        } else {
            None
        };

        Ok(Some(CharacterCardRecord {
            character_id: record.character_id,
            content: record.content,
            cover_file_name: record.cover_file_name,
            cover_mime_type: record.cover_mime_type,
            cover_bytes,
        }))
    }

    async fn list_characters(&self) -> Result<Vec<CharacterCardRecord>, StoreError> {
        let mut entries = fs::read_dir(self.characters_dir()).await?;
        let mut records = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }
            let character_id = entry.file_name().to_string_lossy().into_owned();
            if let Some(record) = self.get_character(&character_id).await? {
                records.push(record);
            }
        }

        Ok(records)
    }

    async fn save_character(&self, record: CharacterCardRecord) -> Result<(), StoreError> {
        let dir = self.character_dir(&record.character_id)?;
        fs::create_dir_all(&dir).await?;
        let cover_path = dir.join(CHARACTER_COVER_FILE);
        match &record.cover_bytes {
            Some(bytes) => write_bytes_atomic(&cover_path, bytes).await?,
            None if path_exists(&cover_path).await? => {
                fs::remove_file(&cover_path).await?;
            }
            None => {}
        }
        write_json_atomic(
            &dir.join(CHARACTER_RECORD_FILE),
            &CharacterCardRecordFile::from(&record),
        )
        .await
    }

    async fn delete_character(
        &self,
        character_id: &str,
    ) -> Result<Option<CharacterCardRecord>, StoreError> {
        let record = self.get_character(character_id).await?;
        if record.is_none() {
            return Ok(None);
        }

        fs::remove_dir_all(self.character_dir(character_id)?).await?;
        Ok(record)
    }

    async fn get_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesRecord>, StoreError> {
        read_optional_json_file(&self.story_resources_path(resource_id)?).await
    }

    async fn list_story_resources(&self) -> Result<Vec<StoryResourcesRecord>, StoreError> {
        list_json_records(&self.story_resources_dir()).await
    }

    async fn save_story_resources(
        &self,
        resources: StoryResourcesRecord,
    ) -> Result<(), StoreError> {
        write_json_atomic(
            &self.story_resources_path(&resources.resource_id)?,
            &resources,
        )
        .await
    }

    async fn delete_story_resources(
        &self,
        resource_id: &str,
    ) -> Result<Option<StoryResourcesRecord>, StoreError> {
        delete_optional_json_file(&self.story_resources_path(resource_id)?).await
    }

    async fn get_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError> {
        read_optional_json_file(&self.story_path(story_id)?).await
    }

    async fn list_stories(&self) -> Result<Vec<StoryRecord>, StoreError> {
        list_json_records(&self.stories_dir()).await
    }

    async fn save_story(&self, story: StoryRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.story_path(&story.story_id)?, &story).await
    }

    async fn delete_story(&self, story_id: &str) -> Result<Option<StoryRecord>, StoreError> {
        delete_optional_json_file(&self.story_path(story_id)?).await
    }

    async fn get_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<Option<StoryDraftRecord>, StoreError> {
        read_optional_json_file(&self.story_draft_path(draft_id)?).await
    }

    async fn list_story_drafts(&self) -> Result<Vec<StoryDraftRecord>, StoreError> {
        list_json_records(&self.story_drafts_dir()).await
    }

    async fn save_story_draft(&self, draft: StoryDraftRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.story_draft_path(&draft.draft_id)?, &draft).await
    }

    async fn delete_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<Option<StoryDraftRecord>, StoreError> {
        delete_optional_json_file(&self.story_draft_path(draft_id)?).await
    }

    async fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError> {
        read_optional_json_file(&self.session_path(session_id)?).await
    }

    async fn list_sessions(&self) -> Result<Vec<SessionRecord>, StoreError> {
        list_json_records(&self.sessions_dir()).await
    }

    async fn save_session(&self, session: SessionRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.session_path(&session.session_id)?, &session).await
    }

    async fn delete_session(&self, session_id: &str) -> Result<Option<SessionRecord>, StoreError> {
        delete_optional_json_file(&self.session_path(session_id)?).await
    }

    async fn get_session_character(
        &self,
        session_character_id: &str,
    ) -> Result<Option<SessionCharacterRecord>, StoreError> {
        read_optional_json_file(&self.session_character_path(session_character_id)?).await
    }

    async fn list_session_characters(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionCharacterRecord>, StoreError> {
        let mut records = list_json_records(&self.session_characters_dir()).await?;
        records.retain(|character: &SessionCharacterRecord| character.session_id == session_id);
        Ok(records)
    }

    async fn save_session_character(
        &self,
        character: SessionCharacterRecord,
    ) -> Result<(), StoreError> {
        write_json_atomic(
            &self.session_character_path(&character.session_character_id)?,
            &character,
        )
        .await
    }

    async fn delete_session_character(
        &self,
        session_character_id: &str,
    ) -> Result<Option<SessionCharacterRecord>, StoreError> {
        delete_optional_json_file(&self.session_character_path(session_character_id)?).await
    }

    async fn get_session_message(
        &self,
        message_id: &str,
    ) -> Result<Option<SessionMessageRecord>, StoreError> {
        read_optional_json_file(&self.session_message_path(message_id)?).await
    }

    async fn list_session_messages(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionMessageRecord>, StoreError> {
        let mut records = list_json_records(&self.session_messages_dir()).await?;
        records.retain(|message: &SessionMessageRecord| message.session_id == session_id);
        Ok(records)
    }

    async fn save_session_message(&self, message: SessionMessageRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.session_message_path(&message.message_id)?, &message).await
    }

    async fn delete_session_message(
        &self,
        message_id: &str,
    ) -> Result<Option<SessionMessageRecord>, StoreError> {
        delete_optional_json_file(&self.session_message_path(message_id)?).await
    }
}

fn validate_path_component(value: &str) -> Result<&str, StoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
    {
        return Err(StoreError::InvalidPathComponent(value.to_owned()));
    }

    Ok(value)
}

async fn path_exists(path: &Path) -> Result<bool, StoreError> {
    match fs::metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(StoreError::Io(error)),
    }
}

async fn read_optional_json_file<T: DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, StoreError> {
    if !path_exists(path).await? {
        return Ok(None);
    }

    read_json_file(path).await.map(Some)
}

async fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<T, StoreError> {
    let bytes = fs::read(path).await?;
    serde_json::from_slice(&bytes).map_err(StoreError::Deserialize)
}

async fn list_json_records<T: DeserializeOwned>(dir: &Path) -> Result<Vec<T>, StoreError> {
    let mut entries = fs::read_dir(dir).await?;
    let mut records = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        records.push(read_json_file(&entry.path()).await?);
    }

    Ok(records)
}

async fn delete_optional_json_file<T: DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, StoreError> {
    let record = read_optional_json_file(path).await?;
    if record.is_none() {
        return Ok(None);
    }

    fs::remove_file(path).await?;
    Ok(record)
}

async fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), StoreError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(StoreError::Serialize)?;
    write_bytes_atomic(path, &bytes).await
}

async fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let parent = path
        .parent()
        .ok_or_else(|| StoreError::MissingParentDirectory(path.to_path_buf()))?;
    fs::create_dir_all(parent).await?;

    let tmp_name = format!(
        ".{}.tmp-{}",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("record"),
        unique_suffix()
    );
    let tmp_path = parent.join(tmp_name);

    let mut file = fs::File::create(&tmp_path).await?;
    file.write_all(bytes).await?;
    file.flush().await?;
    drop(file);

    fs::rename(&tmp_path, path).await?;
    Ok(())
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}
