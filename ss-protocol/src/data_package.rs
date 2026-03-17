use std::collections::BTreeSet;
use std::io::{Cursor, Read, Write};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::ResourceFileRefPayload;
use crate::character::CharacterCardContent;
use store::{
    LorebookRecord, PlayerProfileRecord, PresetRecord, SchemaRecord, StoryRecord,
    StoryResourcesRecord,
};

pub const DATA_PACKAGE_ARCHIVE_FORMAT: &str = "sillystage_data_package";
pub const DATA_PACKAGE_ARCHIVE_VERSION: u32 = 1;
pub const DATA_PACKAGE_ARCHIVE_MANIFEST_PATH: &str = "manifest.json";
pub const DATA_PACKAGE_ARCHIVE_CONTENT_TYPE: &str = "application/x-sillystage-data-package+zip";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct DataPackageExportPrepareParams {
    #[serde(default)]
    pub preset_ids: Vec<String>,
    #[serde(default)]
    pub schema_ids: Vec<String>,
    #[serde(default)]
    pub lorebook_ids: Vec<String>,
    #[serde(default)]
    pub player_profile_ids: Vec<String>,
    #[serde(default)]
    pub character_ids: Vec<String>,
    #[serde(default)]
    pub story_resource_ids: Vec<String>,
    #[serde(default)]
    pub story_ids: Vec<String>,
    #[serde(default = "default_include_dependencies")]
    pub include_dependencies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct DataPackageImportPrepareParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DataPackageImportCommitParams {
    pub import_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DataPackageResourceSummaryPayload {
    #[serde(default)]
    pub ids: Vec<String>,
    pub count: usize,
}

impl DataPackageResourceSummaryPayload {
    pub fn from_ids(mut ids: Vec<String>) -> Self {
        ids.sort();
        ids.dedup();
        Self {
            count: ids.len(),
            ids,
        }
    }

    pub fn validate(&self, label: &str) -> Result<(), DataPackageArchiveError> {
        if self.count != self.ids.len() {
            return Err(DataPackageArchiveError::InvalidManifest(format!(
                "{label} count {} does not match ids length {}",
                self.count,
                self.ids.len()
            )));
        }

        let unique = self.ids.iter().collect::<BTreeSet<_>>();
        if unique.len() != self.ids.len() {
            return Err(DataPackageArchiveError::InvalidManifest(format!(
                "{label} contains duplicate ids"
            )));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DataPackageContentsPayload {
    #[serde(default)]
    pub presets: DataPackageResourceSummaryPayload,
    #[serde(default)]
    pub schemas: DataPackageResourceSummaryPayload,
    #[serde(default)]
    pub lorebooks: DataPackageResourceSummaryPayload,
    #[serde(default)]
    pub player_profiles: DataPackageResourceSummaryPayload,
    #[serde(default)]
    pub characters: DataPackageResourceSummaryPayload,
    #[serde(default)]
    pub story_resources: DataPackageResourceSummaryPayload,
    #[serde(default)]
    pub stories: DataPackageResourceSummaryPayload,
}

impl DataPackageContentsPayload {
    pub fn validate(&self) -> Result<(), DataPackageArchiveError> {
        self.presets.validate("presets")?;
        self.schemas.validate("schemas")?;
        self.lorebooks.validate("lorebooks")?;
        self.player_profiles.validate("player_profiles")?;
        self.characters.validate("characters")?;
        self.story_resources.validate("story_resources")?;
        self.stories.validate("stories")?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageExportPreparedPayload {
    pub export_id: String,
    pub archive: ResourceFileRefPayload,
    pub contents: DataPackageContentsPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageImportPreparedPayload {
    pub import_id: String,
    pub archive: ResourceFileRefPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageImportCommittedPayload {
    pub import_id: String,
    pub contents: DataPackageContentsPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageCharacterManifestEntry {
    pub character_id: String,
    pub content_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_content_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageManifest {
    pub format: String,
    pub version: u32,
    pub created_at_ms: u64,
    pub contents: DataPackageContentsPayload,
    #[serde(default)]
    pub characters: Vec<DataPackageCharacterManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataPackageCharacterEntry {
    pub character_id: String,
    pub content: CharacterCardContent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_file_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_content_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct DataPackageArchive {
    pub manifest: DataPackageManifest,
    pub presets: Vec<PresetRecord>,
    pub schemas: Vec<SchemaRecord>,
    pub lorebooks: Vec<LorebookRecord>,
    pub player_profiles: Vec<PlayerProfileRecord>,
    pub characters: Vec<DataPackageCharacterEntry>,
    pub story_resources: Vec<StoryResourcesRecord>,
    pub stories: Vec<StoryRecord>,
}

impl DataPackageArchive {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        created_at_ms: u64,
        presets: Vec<PresetRecord>,
        schemas: Vec<SchemaRecord>,
        lorebooks: Vec<LorebookRecord>,
        player_profiles: Vec<PlayerProfileRecord>,
        characters: Vec<DataPackageCharacterEntry>,
        story_resources: Vec<StoryResourcesRecord>,
        stories: Vec<StoryRecord>,
    ) -> Self {
        let contents = DataPackageContentsPayload {
            presets: DataPackageResourceSummaryPayload::from_ids(
                presets
                    .iter()
                    .map(|record| record.preset_id.clone())
                    .collect(),
            ),
            schemas: DataPackageResourceSummaryPayload::from_ids(
                schemas
                    .iter()
                    .map(|record| record.schema_id.clone())
                    .collect(),
            ),
            lorebooks: DataPackageResourceSummaryPayload::from_ids(
                lorebooks
                    .iter()
                    .map(|record| record.lorebook_id.clone())
                    .collect(),
            ),
            player_profiles: DataPackageResourceSummaryPayload::from_ids(
                player_profiles
                    .iter()
                    .map(|record| record.player_profile_id.clone())
                    .collect(),
            ),
            characters: DataPackageResourceSummaryPayload::from_ids(
                characters
                    .iter()
                    .map(|record| record.character_id.clone())
                    .collect(),
            ),
            story_resources: DataPackageResourceSummaryPayload::from_ids(
                story_resources
                    .iter()
                    .map(|record| record.resource_id.clone())
                    .collect(),
            ),
            stories: DataPackageResourceSummaryPayload::from_ids(
                stories
                    .iter()
                    .map(|record| record.story_id.clone())
                    .collect(),
            ),
        };

        let manifest = DataPackageManifest {
            format: DATA_PACKAGE_ARCHIVE_FORMAT.to_owned(),
            version: DATA_PACKAGE_ARCHIVE_VERSION,
            created_at_ms,
            contents,
            characters: characters
                .iter()
                .map(character_manifest_entry)
                .collect::<Vec<_>>(),
        };

        Self {
            manifest,
            presets,
            schemas,
            lorebooks,
            player_profiles,
            characters,
            story_resources,
            stories,
        }
    }

    pub fn to_zip_bytes(&self) -> Result<Vec<u8>, DataPackageArchiveError> {
        self.validate()?;

        let mut cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        writer.start_file(DATA_PACKAGE_ARCHIVE_MANIFEST_PATH, options)?;
        writer.write_all(&serde_json::to_vec_pretty(&self.manifest)?)?;

        for preset in &self.presets {
            writer.start_file(preset_path(&preset.preset_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(preset)?)?;
        }
        for schema in &self.schemas {
            writer.start_file(schema_path(&schema.schema_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(schema)?)?;
        }
        for lorebook in &self.lorebooks {
            writer.start_file(lorebook_path(&lorebook.lorebook_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(lorebook)?)?;
        }
        for profile in &self.player_profiles {
            writer.start_file(player_profile_path(&profile.player_profile_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(profile)?)?;
        }
        for character in &self.characters {
            writer.start_file(character_content_path(&character.character_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(&character.content)?)?;
            if let Some(cover_bytes) = &character.cover_bytes {
                writer.start_file(character_cover_path(&character.character_id), options)?;
                writer.write_all(cover_bytes)?;
            }
        }
        for resource in &self.story_resources {
            writer.start_file(story_resources_path(&resource.resource_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(resource)?)?;
        }
        for story in &self.stories {
            writer.start_file(story_path(&story.story_id), options)?;
            writer.write_all(&serde_json::to_vec_pretty(story)?)?;
        }

        writer.finish()?;
        Ok(cursor.into_inner())
    }

    pub fn from_zip_bytes(bytes: &[u8]) -> Result<Self, DataPackageArchiveError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let manifest: DataPackageManifest =
            read_json_entry(&mut archive, DATA_PACKAGE_ARCHIVE_MANIFEST_PATH)?;
        validate_manifest(&manifest)?;

        let presets = manifest
            .contents
            .presets
            .ids
            .iter()
            .map(|id| read_json_entry(&mut archive, &preset_path(id)))
            .collect::<Result<Vec<_>, _>>()?;
        let schemas = manifest
            .contents
            .schemas
            .ids
            .iter()
            .map(|id| read_json_entry(&mut archive, &schema_path(id)))
            .collect::<Result<Vec<_>, _>>()?;
        let lorebooks = manifest
            .contents
            .lorebooks
            .ids
            .iter()
            .map(|id| read_json_entry(&mut archive, &lorebook_path(id)))
            .collect::<Result<Vec<_>, _>>()?;
        let player_profiles = manifest
            .contents
            .player_profiles
            .ids
            .iter()
            .map(|id| read_json_entry(&mut archive, &player_profile_path(id)))
            .collect::<Result<Vec<_>, _>>()?;
        let characters = manifest
            .characters
            .iter()
            .map(|entry| read_character_entry(&mut archive, entry))
            .collect::<Result<Vec<_>, _>>()?;
        let story_resources = manifest
            .contents
            .story_resources
            .ids
            .iter()
            .map(|id| read_json_entry(&mut archive, &story_resources_path(id)))
            .collect::<Result<Vec<_>, _>>()?;
        let stories = manifest
            .contents
            .stories
            .ids
            .iter()
            .map(|id| read_json_entry(&mut archive, &story_path(id)))
            .collect::<Result<Vec<_>, _>>()?;

        let package = Self {
            manifest,
            presets,
            schemas,
            lorebooks,
            player_profiles,
            characters,
            story_resources,
            stories,
        };
        package.validate()?;
        Ok(package)
    }

    pub fn contents(&self) -> DataPackageContentsPayload {
        self.manifest.contents.clone()
    }

    fn validate(&self) -> Result<(), DataPackageArchiveError> {
        validate_manifest(&self.manifest)?;

        validate_record_ids(
            "preset",
            &self.manifest.contents.presets.ids,
            self.presets.iter().map(|record| record.preset_id.as_str()),
        )?;
        validate_record_ids(
            "schema",
            &self.manifest.contents.schemas.ids,
            self.schemas.iter().map(|record| record.schema_id.as_str()),
        )?;
        validate_record_ids(
            "lorebook",
            &self.manifest.contents.lorebooks.ids,
            self.lorebooks
                .iter()
                .map(|record| record.lorebook_id.as_str()),
        )?;
        validate_record_ids(
            "player_profile",
            &self.manifest.contents.player_profiles.ids,
            self.player_profiles
                .iter()
                .map(|record| record.player_profile_id.as_str()),
        )?;
        validate_record_ids(
            "character",
            &self.manifest.contents.characters.ids,
            self.characters
                .iter()
                .map(|record| record.character_id.as_str()),
        )?;
        validate_record_ids(
            "story_resource",
            &self.manifest.contents.story_resources.ids,
            self.story_resources
                .iter()
                .map(|record| record.resource_id.as_str()),
        )?;
        validate_record_ids(
            "story",
            &self.manifest.contents.stories.ids,
            self.stories.iter().map(|record| record.story_id.as_str()),
        )?;

        let manifest_character_ids = self
            .manifest
            .characters
            .iter()
            .map(|entry| entry.character_id.as_str())
            .collect::<Vec<_>>();
        validate_record_ids(
            "character manifest",
            &self.manifest.contents.characters.ids,
            manifest_character_ids.into_iter(),
        )?;

        for character in &self.characters {
            character.validate()?;
        }

        Ok(())
    }
}

impl DataPackageCharacterEntry {
    fn validate(&self) -> Result<(), DataPackageArchiveError> {
        if self.character_id != self.content.id {
            return Err(DataPackageArchiveError::InvalidCharacterEntry(format!(
                "character id '{}' does not match content.id '{}'",
                self.character_id, self.content.id
            )));
        }

        match (
            &self.cover_file_name,
            &self.cover_content_type,
            &self.cover_bytes,
        ) {
            (None, None, None) => Ok(()),
            (Some(_), Some(content_type), Some(bytes)) => {
                if content_type.trim().is_empty() {
                    return Err(DataPackageArchiveError::InvalidCharacterEntry(
                        "character cover content_type must not be empty".to_owned(),
                    ));
                }
                if bytes.is_empty() {
                    return Err(DataPackageArchiveError::InvalidCharacterEntry(
                        "character cover bytes must not be empty".to_owned(),
                    ));
                }
                Ok(())
            }
            _ => Err(DataPackageArchiveError::InvalidCharacterEntry(
                "character cover metadata and bytes must either all exist or all be absent"
                    .to_owned(),
            )),
        }
    }
}

fn character_manifest_entry(
    entry: &DataPackageCharacterEntry,
) -> DataPackageCharacterManifestEntry {
    DataPackageCharacterManifestEntry {
        character_id: entry.character_id.clone(),
        content_path: character_content_path(&entry.character_id),
        cover_path: entry
            .cover_bytes
            .as_ref()
            .map(|_| character_cover_path(&entry.character_id)),
        cover_file_name: entry.cover_file_name.clone(),
        cover_content_type: entry.cover_content_type.clone(),
    }
}

fn read_character_entry(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    entry: &DataPackageCharacterManifestEntry,
) -> Result<DataPackageCharacterEntry, DataPackageArchiveError> {
    let content: CharacterCardContent = read_json_entry(archive, &entry.content_path)?;
    let cover_bytes = match entry.cover_path.as_deref() {
        Some(path) => {
            let mut file = archive
                .by_name(path)
                .map_err(|_| DataPackageArchiveError::MissingArchiveEntry(path.to_owned()))?;
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            Some(bytes)
        }
        None => None,
    };

    Ok(DataPackageCharacterEntry {
        character_id: entry.character_id.clone(),
        content,
        cover_file_name: entry.cover_file_name.clone(),
        cover_content_type: entry.cover_content_type.clone(),
        cover_bytes,
    })
}

fn read_json_entry<T: serde::de::DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    entry_name: &str,
) -> Result<T, DataPackageArchiveError> {
    let mut file = archive
        .by_name(entry_name)
        .map_err(|_| DataPackageArchiveError::MissingArchiveEntry(entry_name.to_owned()))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn validate_manifest(manifest: &DataPackageManifest) -> Result<(), DataPackageArchiveError> {
    if manifest.format != DATA_PACKAGE_ARCHIVE_FORMAT {
        return Err(DataPackageArchiveError::UnsupportedFormat(
            manifest.format.clone(),
        ));
    }
    if manifest.version != DATA_PACKAGE_ARCHIVE_VERSION {
        return Err(DataPackageArchiveError::UnsupportedVersion(
            manifest.version,
        ));
    }
    manifest.contents.validate()?;

    for entry in &manifest.characters {
        if entry.content_path != character_content_path(&entry.character_id) {
            return Err(DataPackageArchiveError::InvalidManifest(format!(
                "character '{}' content_path must be '{}', got '{}'",
                entry.character_id,
                character_content_path(&entry.character_id),
                entry.content_path
            )));
        }
        match (
            entry.cover_path.as_deref(),
            entry.cover_file_name.as_deref(),
            entry.cover_content_type.as_deref(),
        ) {
            (None, None, None) => {}
            (Some(path), Some(file_name), Some(content_type)) => {
                if path != character_cover_path(&entry.character_id) {
                    return Err(DataPackageArchiveError::InvalidManifest(format!(
                        "character '{}' cover_path must be '{}', got '{}'",
                        entry.character_id,
                        character_cover_path(&entry.character_id),
                        path
                    )));
                }
                if file_name.trim().is_empty() || content_type.trim().is_empty() {
                    return Err(DataPackageArchiveError::InvalidManifest(format!(
                        "character '{}' cover metadata must not be empty",
                        entry.character_id
                    )));
                }
            }
            _ => {
                return Err(DataPackageArchiveError::InvalidManifest(format!(
                    "character '{}' cover metadata must be complete or absent",
                    entry.character_id
                )));
            }
        }
    }

    Ok(())
}

fn validate_record_ids<'a>(
    label: &str,
    manifest_ids: &[String],
    actual_ids: impl Iterator<Item = &'a str>,
) -> Result<(), DataPackageArchiveError> {
    let manifest_ids = manifest_ids.iter().map(String::as_str).collect::<Vec<_>>();
    let actual_ids = actual_ids.collect::<Vec<_>>();

    if manifest_ids.len() != actual_ids.len() {
        return Err(DataPackageArchiveError::InvalidManifest(format!(
            "{label} ids length {} does not match payload count {}",
            manifest_ids.len(),
            actual_ids.len()
        )));
    }

    let manifest_set = manifest_ids.into_iter().collect::<BTreeSet<_>>();
    let actual_set = actual_ids.into_iter().collect::<BTreeSet<_>>();
    if manifest_set != actual_set {
        return Err(DataPackageArchiveError::InvalidManifest(format!(
            "{label} ids in manifest do not match archived records"
        )));
    }

    Ok(())
}

fn preset_path(id: &str) -> String {
    format!("presets/{id}.json")
}

fn schema_path(id: &str) -> String {
    format!("schemas/{id}.json")
}

fn lorebook_path(id: &str) -> String {
    format!("lorebooks/{id}.json")
}

fn player_profile_path(id: &str) -> String {
    format!("player_profiles/{id}.json")
}

fn character_content_path(id: &str) -> String {
    format!("characters/{id}/content.json")
}

fn character_cover_path(id: &str) -> String {
    format!("characters/{id}/cover.bin")
}

fn story_resources_path(id: &str) -> String {
    format!("story_resources/{id}.json")
}

fn story_path(id: &str) -> String {
    format!("stories/{id}.json")
}

const fn default_include_dependencies() -> bool {
    true
}

#[derive(Debug, Error)]
pub enum DataPackageArchiveError {
    #[error("unsupported data package format: {0}")]
    UnsupportedFormat(String),
    #[error("unsupported data package version: {0}")]
    UnsupportedVersion(u32),
    #[error("invalid data package manifest: {0}")]
    InvalidManifest(String),
    #[error("invalid data package character entry: {0}")]
    InvalidCharacterEntry(String),
    #[error("missing data package archive entry: {0}")]
    MissingArchiveEntry(String),
    #[error("failed to serialize or deserialize data package json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to read or write data package zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("failed to read or write data package bytes: {0}")]
    Io(#[from] std::io::Error),
}
