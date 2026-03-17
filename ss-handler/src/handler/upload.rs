use protocol::{
    CHARACTER_ARCHIVE_CONTENT_TYPE, CharacterArchive, CharacterArchiveManifest,
    CharacterCardSummaryPayload, CharacterCoverMimeType, CharacterCreateParams,
    CharacterCreatedPayload, CharacterDeleteParams, CharacterGetParams, CharacterSchemaPayload,
    CharacterUpdateParams, CharactersListedPayload, JsonRpcResponseMessage, ResourceFilePayload,
    ResponseResult,
};
use store::{BlobRecord, CharacterCardDefinition, CharacterCardRecord};

use crate::error::HandlerError;

use super::Handler;
use super::data_package::{
    PACKAGE_ARCHIVE_FILE_ID, PACKAGE_EXPORT_RESOURCE_PREFIX, PACKAGE_IMPORT_RESOURCE_PREFIX,
};

const CHARACTER_RESOURCE_PREFIX: &str = "character:";
const CHARACTER_COVER_FILE_ID: &str = "cover";
const CHARACTER_ARCHIVE_FILE_ID: &str = "archive";

#[derive(Debug, Clone)]
pub struct BinaryAsset {
    pub file_name: Option<String>,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

impl Handler {
    pub async fn upload_resource_file(
        &self,
        resource_id: &str,
        file_id: &str,
        file_name: Option<String>,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<ResourceFilePayload, HandlerError> {
        match ResourceFileTarget::parse(resource_id, file_id)? {
            ResourceFileTarget::CharacterCover {
                resource_id,
                character_id,
            } => {
                self.upload_character_cover(
                    &resource_id,
                    &character_id,
                    file_name,
                    content_type,
                    bytes,
                )
                .await
            }
            ResourceFileTarget::CharacterArchive {
                resource_id,
                character_id,
            } => {
                self.upload_character_archive(&resource_id, &character_id, file_name, bytes)
                    .await
            }
            ResourceFileTarget::PackageImportArchive {
                resource_id,
                import_id,
            } => {
                self.upload_package_import_archive(&import_id, &resource_id, file_name, bytes)
                    .await
            }
            ResourceFileTarget::PackageExportArchive { .. } => Err(
                HandlerError::InvalidFileReference(format!("{resource_id}/{file_id}")),
            ),
        }
    }

    pub async fn download_resource_file(
        &self,
        resource_id: &str,
        file_id: &str,
    ) -> Result<BinaryAsset, HandlerError> {
        match ResourceFileTarget::parse(resource_id, file_id)? {
            ResourceFileTarget::CharacterCover { character_id, .. } => {
                self.download_character_cover(&character_id).await
            }
            ResourceFileTarget::CharacterArchive { character_id, .. } => {
                self.download_character_archive(&character_id).await
            }
            ResourceFileTarget::PackageExportArchive { export_id, .. } => {
                self.download_package_export_archive(&export_id).await
            }
            ResourceFileTarget::PackageImportArchive { .. } => Err(
                HandlerError::InvalidFileReference(format!("{resource_id}/{file_id}")),
            ),
        }
    }

    pub(crate) async fn handle_character_create(
        &self,
        request_id: &str,
        params: CharacterCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let character_id = params.content.id.trim().to_owned();
        if character_id.is_empty() {
            return Err(HandlerError::EmptyCharacterId);
        }

        if self.store.get_character(&character_id).await?.is_some() {
            return Err(HandlerError::DuplicateCharacter(character_id));
        }
        self.ensure_schema_exists(&params.content.schema_id).await?;

        let mut content = params.content;
        content.id = character_id.clone();
        let record = CharacterCardRecord {
            character_id,
            content: character_definition_from_content(content),
            cover_blob_id: None,
            cover_file_name: None,
            cover_mime_type: None,
        };
        self.store.save_character(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharacterCreated(CharacterCreatedPayload {
                character_id: record.character_id.clone(),
                character_summary: character_summary_payload_from_record(&record),
            }),
        ))
    }

    pub(crate) async fn handle_character_get(
        &self,
        request_id: &str,
        params: CharacterGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let record = self
            .store
            .get_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Character(Box::new(CharacterSchemaPayload {
                character_id: record.character_id,
                content: character_content_from_definition(&record.content),
                cover_file_name: record.cover_file_name,
                cover_mime_type: parse_cover_mime_type_option(record.cover_mime_type.as_deref()),
            })),
        ))
    }

    pub(crate) async fn handle_character_update(
        &self,
        request_id: &str,
        params: CharacterUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut record = self
            .store
            .get_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;

        if params.content.id.trim().is_empty() {
            return Err(HandlerError::EmptyCharacterId);
        }

        if params.content.id != params.character_id {
            return Err(HandlerError::CharacterIdMismatch {
                expected: params.character_id,
                got: params.content.id,
            });
        }

        self.ensure_schema_exists(&params.content.schema_id).await?;
        record.content = character_definition_from_content(params.content);
        self.store.save_character(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Character(Box::new(CharacterSchemaPayload {
                character_id: record.character_id,
                content: character_content_from_definition(&record.content),
                cover_file_name: record.cover_file_name,
                cover_mime_type: parse_cover_mime_type_option(record.cover_mime_type.as_deref()),
            })),
        ))
    }

    pub(crate) async fn handle_character_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let characters = self
            .store
            .list_characters()
            .await?
            .into_iter()
            .map(|record| character_summary_payload_from_record(&record))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharactersListed(CharactersListedPayload { characters }),
        ))
    }

    pub(crate) async fn handle_character_delete(
        &self,
        request_id: &str,
        params: CharacterDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if self
            .store
            .list_story_resources()
            .await?
            .into_iter()
            .any(|resource| resource.character_ids.contains(&params.character_id))
        {
            return Err(HandlerError::CharacterInUse(params.character_id));
        }

        let deleted = self
            .store
            .delete_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;
        if let Some(blob_id) = deleted.cover_blob_id.as_deref() {
            let _ = self.store.delete_blob(blob_id).await;
        }

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharacterDeleted(protocol::CharacterDeletedPayload {
                character_id: params.character_id,
            }),
        ))
    }

    async fn upload_character_cover(
        &self,
        resource_id: &str,
        character_id: &str,
        file_name: Option<String>,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<ResourceFilePayload, HandlerError> {
        let content_type = normalize_content_type(content_type);
        let Some(cover_mime_type) = CharacterCoverMimeType::parse(&content_type) else {
            return Err(HandlerError::InvalidCharacterCoverPayload(format!(
                "unsupported content type '{content_type}'"
            )));
        };

        if bytes.is_empty() {
            return Err(HandlerError::InvalidCharacterCoverPayload(
                "cover bytes must not be empty".to_owned(),
            ));
        }

        let mut record = self
            .store
            .get_character(character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(character_id.to_owned()))?;
        let cover_file_name = normalize_file_name(file_name)
            .unwrap_or_else(|| format!("cover.{}", cover_mime_type.extension()));
        let previous_blob_id = record.cover_blob_id.clone();
        let cover_blob = self
            .store_blob(
                Some(cover_file_name.clone()),
                cover_mime_type.as_content_type().to_owned(),
                bytes,
            )
            .await?;

        record.cover_blob_id = Some(cover_blob.blob_id.clone());
        record.cover_file_name = Some(cover_file_name.clone());
        record.cover_mime_type = Some(store_cover_mime_type(cover_mime_type));
        if let Err(error) = self.store.save_character(record).await {
            let _ = self.store.delete_blob(&cover_blob.blob_id).await;
            return Err(error.into());
        }

        if let Some(previous_blob_id) = previous_blob_id.as_deref() {
            if previous_blob_id != cover_blob.blob_id {
                let _ = self.store.delete_blob(previous_blob_id).await;
            }
        }

        Ok(ResourceFilePayload {
            resource_id: resource_id.to_owned(),
            file_id: CHARACTER_COVER_FILE_ID.to_owned(),
            file_name: Some(cover_file_name),
            content_type: cover_mime_type.as_content_type().to_owned(),
            size_bytes: cover_blob.bytes.len() as u64,
        })
    }

    async fn upload_character_archive(
        &self,
        resource_id: &str,
        character_id: &str,
        file_name: Option<String>,
        bytes: Vec<u8>,
    ) -> Result<ResourceFilePayload, HandlerError> {
        let archive = CharacterArchive::from_chr_bytes(&bytes)?;
        if archive.content.id != character_id {
            return Err(HandlerError::CharacterIdMismatch {
                expected: character_id.to_owned(),
                got: archive.content.id.clone(),
            });
        }

        if self.store.get_character(character_id).await?.is_some() {
            return Err(HandlerError::DuplicateCharacter(character_id.to_owned()));
        }
        self.ensure_schema_exists(&archive.content.schema_id)
            .await?;

        let cover_blob = self
            .store_blob(
                Some(archive.manifest.cover_path.clone()),
                archive
                    .manifest
                    .cover_mime_type
                    .as_content_type()
                    .to_owned(),
                archive.cover_bytes.clone(),
            )
            .await?;

        let record = CharacterCardRecord {
            character_id: archive.content.id.clone(),
            content: character_definition_from_content(archive.content),
            cover_blob_id: Some(cover_blob.blob_id.clone()),
            cover_file_name: Some(archive.manifest.cover_path.clone()),
            cover_mime_type: Some(store_cover_mime_type(archive.manifest.cover_mime_type)),
        };
        if let Err(error) = self.store.save_character(record).await {
            let _ = self.store.delete_blob(&cover_blob.blob_id).await;
            return Err(error.into());
        }

        Ok(ResourceFilePayload {
            resource_id: resource_id.to_owned(),
            file_id: CHARACTER_ARCHIVE_FILE_ID.to_owned(),
            file_name: normalize_file_name(file_name)
                .or_else(|| Some(format!("{character_id}.chr"))),
            content_type: CHARACTER_ARCHIVE_CONTENT_TYPE.to_owned(),
            size_bytes: bytes.len() as u64,
        })
    }

    async fn download_character_archive(
        &self,
        character_id: &str,
    ) -> Result<BinaryAsset, HandlerError> {
        let record = self
            .store
            .get_character(character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(character_id.to_owned()))?;
        let blob = self.character_cover_blob(&record).await?;
        let cover_file_name = record
            .cover_file_name
            .clone()
            .or(blob.file_name.clone())
            .ok_or_else(|| HandlerError::MissingCharacterCover(character_id.to_owned()))?;
        let cover_mime_type = parse_cover_mime_type_option(record.cover_mime_type.as_deref())
            .or_else(|| CharacterCoverMimeType::parse(&blob.content_type))
            .ok_or_else(|| HandlerError::MissingCharacterCover(character_id.to_owned()))?;
        let archive = CharacterArchive {
            manifest: CharacterArchiveManifest::new(
                record.character_id.clone(),
                cover_mime_type,
                cover_file_name,
            ),
            content: character_content_from_definition(&record.content),
            cover_bytes: blob.bytes,
        };
        let bytes = archive.to_chr_bytes()?;

        Ok(BinaryAsset {
            file_name: Some(format!("{}.chr", archive.content.id)),
            content_type: CHARACTER_ARCHIVE_CONTENT_TYPE.to_owned(),
            bytes,
        })
    }

    async fn download_character_cover(
        &self,
        character_id: &str,
    ) -> Result<BinaryAsset, HandlerError> {
        let record = self
            .store
            .get_character(character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(character_id.to_owned()))?;
        let blob = self.character_cover_blob(&record).await?;

        Ok(BinaryAsset {
            file_name: record.cover_file_name.clone().or(blob.file_name),
            content_type: blob.content_type,
            bytes: blob.bytes,
        })
    }

    async fn store_blob(
        &self,
        file_name: Option<String>,
        content_type: String,
        bytes: Vec<u8>,
    ) -> Result<BlobRecord, HandlerError> {
        let record = BlobRecord {
            blob_id: self.id_generator.next("blob"),
            file_name: normalize_file_name(file_name),
            content_type: normalize_content_type(content_type),
            bytes,
        };
        self.store.save_blob(record.clone()).await?;
        Ok(record)
    }

    async fn character_cover_blob(
        &self,
        record: &CharacterCardRecord,
    ) -> Result<BlobRecord, HandlerError> {
        let blob_id = record
            .cover_blob_id
            .as_deref()
            .ok_or_else(|| HandlerError::MissingCharacterCover(record.character_id.clone()))?;
        self.store
            .get_blob(blob_id)
            .await?
            .ok_or_else(|| HandlerError::MissingBlob(blob_id.to_owned()))
    }
}

#[derive(Debug, Clone)]
enum ResourceFileTarget {
    CharacterCover {
        resource_id: String,
        character_id: String,
    },
    CharacterArchive {
        resource_id: String,
        character_id: String,
    },
    PackageExportArchive {
        export_id: String,
    },
    PackageImportArchive {
        resource_id: String,
        import_id: String,
    },
}

impl ResourceFileTarget {
    fn parse(resource_id: &str, file_id: &str) -> Result<Self, HandlerError> {
        let resource_id = resource_id.trim();
        let file_id = file_id.trim();
        let invalid = || {
            HandlerError::InvalidFileReference(format!(
                "{}/{}",
                resource_id,
                if file_id.is_empty() {
                    "<empty>"
                } else {
                    file_id
                }
            ))
        };

        if let Some(character_id) = resource_id
            .strip_prefix(CHARACTER_RESOURCE_PREFIX)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return match file_id {
                CHARACTER_COVER_FILE_ID => Ok(Self::CharacterCover {
                    resource_id: resource_id.to_owned(),
                    character_id: character_id.to_owned(),
                }),
                CHARACTER_ARCHIVE_FILE_ID => Ok(Self::CharacterArchive {
                    resource_id: resource_id.to_owned(),
                    character_id: character_id.to_owned(),
                }),
                _ => Err(invalid()),
            };
        }

        if let Some(export_id) = resource_id
            .strip_prefix(PACKAGE_EXPORT_RESOURCE_PREFIX)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return match file_id {
                PACKAGE_ARCHIVE_FILE_ID => Ok(Self::PackageExportArchive {
                    export_id: export_id.to_owned(),
                }),
                _ => Err(invalid()),
            };
        }

        if let Some(import_id) = resource_id
            .strip_prefix(PACKAGE_IMPORT_RESOURCE_PREFIX)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return match file_id {
                PACKAGE_ARCHIVE_FILE_ID => Ok(Self::PackageImportArchive {
                    resource_id: resource_id.to_owned(),
                    import_id: import_id.to_owned(),
                }),
                _ => Err(invalid()),
            };
        }

        Err(invalid())
    }
}

fn parse_cover_mime_type_option(value: Option<&str>) -> Option<CharacterCoverMimeType> {
    value.and_then(CharacterCoverMimeType::parse)
}

fn store_cover_mime_type(value: CharacterCoverMimeType) -> String {
    value.as_content_type().to_owned()
}

fn character_summary_payload_from_record(
    record: &CharacterCardRecord,
) -> CharacterCardSummaryPayload {
    CharacterCardSummaryPayload {
        character_id: record.character_id.clone(),
        name: record.content.name.clone(),
        personality: record.content.personality.clone(),
        style: record.content.style.clone(),
        cover_file_name: record.cover_file_name.clone(),
        cover_mime_type: parse_cover_mime_type_option(record.cover_mime_type.as_deref()),
    }
}

fn character_definition_from_content(
    content: protocol::CharacterCardContent,
) -> CharacterCardDefinition {
    CharacterCardDefinition {
        id: content.id,
        name: content.name,
        personality: content.personality,
        style: content.style,
        schema_id: content.schema_id,
        system_prompt: content.system_prompt,
    }
}

fn character_content_from_definition(
    definition: &CharacterCardDefinition,
) -> protocol::CharacterCardContent {
    protocol::CharacterCardContent {
        id: definition.id.clone(),
        name: definition.name.clone(),
        personality: definition.personality.clone(),
        style: definition.style.clone(),
        schema_id: definition.schema_id.clone(),
        system_prompt: definition.system_prompt.clone(),
    }
}

fn normalize_file_name(file_name: Option<String>) -> Option<String> {
    file_name.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn normalize_content_type(content_type: String) -> String {
    let trimmed = content_type.trim();
    if trimmed.is_empty() {
        "application/octet-stream".to_owned()
    } else {
        trimmed.to_owned()
    }
}
