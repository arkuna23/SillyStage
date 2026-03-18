use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::error::StoreError;
use crate::record::{
    ApiGroupRecord, ApiRecord, BlobRecord, CharacterCardDefinition, CharacterCardRecord,
    LorebookRecord, PlayerProfileRecord, PresetRecord, SchemaRecord, SessionCharacterRecord,
    SessionMessageRecord, SessionRecord, StoryDraftRecord, StoryRecord, StoryResourcesRecord,
};
use crate::store::Store;

const GLOBAL_DIR: &str = "global";
const BLOBS_DIR: &str = "blobs";
const BLOB_RECORD_FILE: &str = "record.json";
const BLOB_DATA_FILE: &str = "data.bin";
const APIS_DIR: &str = "apis";
const API_GROUPS_DIR: &str = "api_groups";
const PRESETS_DIR: &str = "presets";
const SCHEMAS_DIR: &str = "schemas";
const LOREBOOKS_DIR: &str = "lorebooks";
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
    #[serde(default)]
    cover_blob_id: Option<String>,
    cover_file_name: Option<String>,
    cover_mime_type: Option<String>,
}

impl From<&CharacterCardRecord> for CharacterCardRecordFile {
    fn from(value: &CharacterCardRecord) -> Self {
        Self {
            character_id: value.character_id.clone(),
            content: value.content.clone(),
            cover_blob_id: value.cover_blob_id.clone(),
            cover_file_name: value.cover_file_name.clone(),
            cover_mime_type: value.cover_mime_type.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlobRecordFile {
    blob_id: String,
    file_name: Option<String>,
    content_type: String,
}

impl From<&BlobRecord> for BlobRecordFile {
    fn from(value: &BlobRecord) -> Self {
        Self {
            blob_id: value.blob_id.clone(),
            file_name: value.file_name.clone(),
            content_type: value.content_type.clone(),
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
        fs::create_dir_all(self.blobs_dir()).await?;
        fs::create_dir_all(self.apis_dir()).await?;
        fs::create_dir_all(self.api_groups_dir()).await?;
        fs::create_dir_all(self.presets_dir()).await?;
        fs::create_dir_all(self.schemas_dir()).await?;
        fs::create_dir_all(self.lorebooks_dir()).await?;
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

    fn blobs_dir(&self) -> PathBuf {
        self.root.join(BLOBS_DIR)
    }

    fn blob_dir(&self, blob_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self.blobs_dir().join(validate_path_component(blob_id)?))
    }

    fn blob_record_path(&self, blob_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self.blob_dir(blob_id)?.join(BLOB_RECORD_FILE))
    }

    fn blob_data_path(&self, blob_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self.blob_dir(blob_id)?.join(BLOB_DATA_FILE))
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

    fn lorebooks_dir(&self) -> PathBuf {
        self.root.join(LOREBOOKS_DIR)
    }

    fn lorebook_path(&self, lorebook_id: &str) -> Result<PathBuf, StoreError> {
        Ok(self
            .lorebooks_dir()
            .join(format!("{}.json", validate_path_component(lorebook_id)?)))
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
    async fn get_blob(&self, blob_id: &str) -> Result<Option<BlobRecord>, StoreError> {
        let record_path = self.blob_record_path(blob_id)?;
        if !path_exists(&record_path).await? {
            return Ok(None);
        }

        let record: BlobRecordFile = read_json_file(&record_path).await?;
        let bytes = fs::read(self.blob_data_path(blob_id)?).await?;
        Ok(Some(BlobRecord {
            blob_id: record.blob_id,
            file_name: record.file_name,
            content_type: record.content_type,
            bytes,
        }))
    }

    async fn save_blob(&self, record: BlobRecord) -> Result<(), StoreError> {
        let dir = self.blob_dir(&record.blob_id)?;
        fs::create_dir_all(&dir).await?;
        write_bytes_atomic(&dir.join(BLOB_DATA_FILE), &record.bytes).await?;
        write_json_atomic(&dir.join(BLOB_RECORD_FILE), &BlobRecordFile::from(&record)).await
    }

    async fn delete_blob(&self, blob_id: &str) -> Result<Option<BlobRecord>, StoreError> {
        let record = self.get_blob(blob_id).await?;
        if record.is_none() {
            return Ok(None);
        }

        fs::remove_dir_all(self.blob_dir(blob_id)?).await?;
        Ok(record)
    }

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

    async fn get_lorebook(&self, lorebook_id: &str) -> Result<Option<LorebookRecord>, StoreError> {
        read_optional_json_file(&self.lorebook_path(lorebook_id)?).await
    }

    async fn list_lorebooks(&self) -> Result<Vec<LorebookRecord>, StoreError> {
        list_json_records(&self.lorebooks_dir()).await
    }

    async fn save_lorebook(&self, record: LorebookRecord) -> Result<(), StoreError> {
        write_json_atomic(&self.lorebook_path(&record.lorebook_id)?, &record).await
    }

    async fn delete_lorebook(
        &self,
        lorebook_id: &str,
    ) -> Result<Option<LorebookRecord>, StoreError> {
        delete_optional_json_file(&self.lorebook_path(lorebook_id)?).await
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

        let mut record: CharacterCardRecordFile = read_json_file(&record_path).await?;
        if record.cover_blob_id.is_none()
            && record.cover_file_name.is_some()
            && path_exists(&self.character_cover_path(character_id)?).await?
        {
            self.migrate_legacy_character_cover(character_id, &mut record)
                .await?;
        }

        Ok(Some(CharacterCardRecord {
            character_id: record.character_id,
            content: record.content,
            cover_blob_id: record.cover_blob_id,
            cover_file_name: record.cover_file_name,
            cover_mime_type: record.cover_mime_type,
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
        if path_exists(&cover_path).await? {
            fs::remove_file(&cover_path).await?;
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

impl FileSystemStore {
    async fn migrate_legacy_character_cover(
        &self,
        character_id: &str,
        record: &mut CharacterCardRecordFile,
    ) -> Result<(), StoreError> {
        let cover_path = self.character_cover_path(character_id)?;
        let bytes = fs::read(&cover_path).await?;
        let blob_id = format!("character-cover-{character_id}");
        self.save_blob(BlobRecord {
            blob_id: blob_id.clone(),
            file_name: record.cover_file_name.clone(),
            content_type: record
                .cover_mime_type
                .clone()
                .unwrap_or_else(|| "application/octet-stream".to_owned()),
            bytes,
        })
        .await?;
        record.cover_blob_id = Some(blob_id);
        write_json_atomic(&self.character_record_path(character_id)?, record).await?;
        fs::remove_file(cover_path).await?;
        Ok(())
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

    if let Err(error) = fs::rename(&tmp_path, path).await {
        if should_retry_replace(&error) && path_exists(path).await? {
            fs::remove_file(path).await?;
            fs::rename(&tmp_path, path).await?;
        } else {
            let _ = fs::remove_file(&tmp_path).await;
            return Err(StoreError::Io(error));
        }
    }

    Ok(())
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos()
}

fn should_retry_replace(error: &std::io::Error) -> bool {
    cfg!(windows)
        && matches!(
            error.kind(),
            std::io::ErrorKind::AlreadyExists | std::io::ErrorKind::PermissionDenied
        )
}
