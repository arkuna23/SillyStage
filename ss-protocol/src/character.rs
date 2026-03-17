use std::io::{Cursor, Read, Write};

use agents::actor::CharacterCard;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub const CHARACTER_ARCHIVE_FORMAT: &str = "sillystage_character_card";
pub const CHARACTER_ARCHIVE_VERSION: u32 = 1;
pub const CHARACTER_ARCHIVE_MANIFEST_PATH: &str = "manifest.json";
pub const CHARACTER_ARCHIVE_CONTENT_PATH: &str = "content.json";
pub const CHARACTER_ARCHIVE_CONTENT_TYPE: &str = "application/x-sillystage-character-card";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CharacterCoverMimeType {
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/webp")]
    Webp,
}

impl CharacterCoverMimeType {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
        }
    }

    pub const fn as_content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "image/png" => Some(Self::Png),
            "image/jpeg" => Some(Self::Jpeg),
            "image/webp" => Some(Self::Webp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterArchiveManifest {
    pub format: String,
    pub version: u32,
    pub character_id: String,
    pub content_path: String,
    pub cover_path: String,
    pub cover_mime_type: CharacterCoverMimeType,
}

impl CharacterArchiveManifest {
    pub fn new(
        character_id: impl Into<String>,
        cover_mime_type: CharacterCoverMimeType,
        cover_path: impl Into<String>,
    ) -> Self {
        Self {
            format: CHARACTER_ARCHIVE_FORMAT.to_owned(),
            version: CHARACTER_ARCHIVE_VERSION,
            character_id: character_id.into(),
            content_path: CHARACTER_ARCHIVE_CONTENT_PATH.to_owned(),
            cover_path: cover_path.into(),
            cover_mime_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardContent {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub style: String,
    pub schema_id: String,
    pub system_prompt: String,
}

impl From<&CharacterCard> for CharacterCardContent {
    fn from(value: &CharacterCard) -> Self {
        Self {
            id: value.id.clone(),
            name: value.name.clone(),
            personality: value.personality.clone(),
            style: value.style.clone(),
            schema_id: value.id.clone(),
            system_prompt: value.system_prompt.clone(),
        }
    }
}

impl From<CharacterCard> for CharacterCardContent {
    fn from(value: CharacterCard) -> Self {
        Self::from(&value)
    }
}

impl From<CharacterCardContent> for CharacterCard {
    fn from(value: CharacterCardContent) -> Self {
        Self {
            id: value.id,
            name: value.name,
            personality: value.personality,
            style: value.style,
            state_schema: Default::default(),
            system_prompt: value.system_prompt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterCardSummaryPayload {
    pub character_id: String,
    pub name: String,
    pub personality: String,
    pub style: String,
    pub cover_file_name: Option<String>,
    pub cover_mime_type: Option<CharacterCoverMimeType>,
}

#[derive(Debug, Clone)]
pub struct CharacterArchive {
    pub manifest: CharacterArchiveManifest,
    pub content: CharacterCardContent,
    pub cover_bytes: Vec<u8>,
}

impl CharacterArchive {
    pub fn new(
        content: CharacterCardContent,
        cover_mime_type: CharacterCoverMimeType,
        cover_bytes: Vec<u8>,
    ) -> Self {
        let cover_path = format!("cover.{}", cover_mime_type.extension());
        let manifest =
            CharacterArchiveManifest::new(content.id.clone(), cover_mime_type, cover_path);

        Self {
            manifest,
            content,
            cover_bytes,
        }
    }

    pub fn summary(&self) -> CharacterCardSummaryPayload {
        CharacterCardSummaryPayload {
            character_id: self.content.id.clone(),
            name: self.content.name.clone(),
            personality: self.content.personality.clone(),
            style: self.content.style.clone(),
            cover_file_name: Some(self.manifest.cover_path.clone()),
            cover_mime_type: Some(self.manifest.cover_mime_type),
        }
    }

    pub fn to_chr_bytes(&self) -> Result<Vec<u8>, CharacterArchiveError> {
        self.validate()?;

        let mut cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(&mut cursor);
        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        writer.start_file(CHARACTER_ARCHIVE_MANIFEST_PATH, options)?;
        writer.write_all(&serde_json::to_vec_pretty(&self.manifest)?)?;

        writer.start_file(&self.manifest.content_path, options)?;
        writer.write_all(&serde_json::to_vec_pretty(&self.content)?)?;

        writer.start_file(&self.manifest.cover_path, options)?;
        writer.write_all(&self.cover_bytes)?;

        writer.finish()?;
        Ok(cursor.into_inner())
    }

    pub fn from_chr_bytes(bytes: &[u8]) -> Result<Self, CharacterArchiveError> {
        let cursor = Cursor::new(bytes);
        let mut archive = ZipArchive::new(cursor)?;

        let manifest: CharacterArchiveManifest =
            read_json_entry(&mut archive, CHARACTER_ARCHIVE_MANIFEST_PATH)?;
        validate_manifest(&manifest)?;

        let content: CharacterCardContent = read_json_entry(&mut archive, &manifest.content_path)?;
        if content.id != manifest.character_id {
            return Err(CharacterArchiveError::CharacterIdMismatch {
                manifest_character_id: manifest.character_id.clone(),
                content_character_id: content.id.clone(),
            });
        }

        let mut cover_bytes = Vec::new();
        archive
            .by_name(&manifest.cover_path)
            .map_err(|_| CharacterArchiveError::MissingArchiveEntry(manifest.cover_path.clone()))?
            .read_to_end(&mut cover_bytes)?;

        Ok(Self {
            manifest,
            content,
            cover_bytes,
        })
    }

    fn validate(&self) -> Result<(), CharacterArchiveError> {
        validate_manifest(&self.manifest)?;

        if self.manifest.character_id != self.content.id {
            return Err(CharacterArchiveError::CharacterIdMismatch {
                manifest_character_id: self.manifest.character_id.clone(),
                content_character_id: self.content.id.clone(),
            });
        }

        if self.cover_bytes.is_empty() {
            return Err(CharacterArchiveError::EmptyCoverBytes);
        }

        Ok(())
    }
}

fn read_json_entry<T: serde::de::DeserializeOwned>(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    entry_name: &str,
) -> Result<T, CharacterArchiveError> {
    let mut file = archive
        .by_name(entry_name)
        .map_err(|_| CharacterArchiveError::MissingArchiveEntry(entry_name.to_owned()))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(serde_json::from_slice(&bytes)?)
}

fn validate_manifest(manifest: &CharacterArchiveManifest) -> Result<(), CharacterArchiveError> {
    if manifest.format != CHARACTER_ARCHIVE_FORMAT {
        return Err(CharacterArchiveError::UnsupportedFormat(
            manifest.format.clone(),
        ));
    }

    if manifest.version != CHARACTER_ARCHIVE_VERSION {
        return Err(CharacterArchiveError::UnsupportedVersion(manifest.version));
    }

    if manifest.content_path != CHARACTER_ARCHIVE_CONTENT_PATH {
        return Err(CharacterArchiveError::InvalidContentPath(
            manifest.content_path.clone(),
        ));
    }

    if !manifest.cover_path.starts_with("cover.") {
        return Err(CharacterArchiveError::InvalidCoverPath(
            manifest.cover_path.clone(),
        ));
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum CharacterArchiveError {
    #[error("unsupported character archive format: {0}")]
    UnsupportedFormat(String),
    #[error("unsupported character archive version: {0}")]
    UnsupportedVersion(u32),
    #[error("character archive content path must be content.json, got {0}")]
    InvalidContentPath(String),
    #[error("character archive cover path must start with cover., got {0}")]
    InvalidCoverPath(String),
    #[error("missing character archive entry: {0}")]
    MissingArchiveEntry(String),
    #[error(
        "character id mismatch between manifest ({manifest_character_id}) and content ({content_character_id})"
    )]
    CharacterIdMismatch {
        manifest_character_id: String,
        content_character_id: String,
    },
    #[error("character archive cover bytes must not be empty")]
    EmptyCoverBytes,
    #[error("failed to serialize or deserialize character archive json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("failed to read or write character archive zip: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("failed to read or write character archive bytes: {0}")]
    Io(#[from] std::io::Error),
}
