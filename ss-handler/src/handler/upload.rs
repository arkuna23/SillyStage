use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use protocol::{
    CHARACTER_ARCHIVE_CONTENT_TYPE, CharacterArchive, CharacterArchiveManifest,
    CharacterCardSummaryPayload, CharacterCardUploadedPayload, CharacterChrExportPayload,
    CharacterCoverMimeType, CharacterCoverPayload, CharacterCoverUpdatedPayload,
    CharacterCreateParams, CharacterCreatedPayload, CharacterDeleteParams,
    CharacterDeletedPayload, CharacterDetailPayload, CharacterExportChrParams,
    CharacterGetCoverParams, CharacterGetParams, CharacterSetCoverParams,
    CharactersListedPayload, JsonRpcResponseMessage, ResponseResult,
    UploadChunkAcceptedPayload, UploadChunkParams, UploadCompleteParams, UploadInitParams,
    UploadInitializedPayload, UploadTargetKind,
};
use store::CharacterCardRecord;

use crate::error::HandlerError;
use crate::store::UploadRecord;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_upload_init(
        &self,
        request_id: &str,
        params: UploadInitParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let upload_id = self.id_generator.next("upload");
        let record = UploadRecord {
            upload_id: upload_id.clone(),
            target_kind: params.target_kind,
            total_size: params.total_size,
            next_chunk_index: 0,
            next_offset: 0,
            bytes: Vec::new(),
        };

        self.uploads.save(record).await;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::UploadInitialized(UploadInitializedPayload {
                upload_id,
                chunk_size_hint: 64 * 1024,
            }),
        ))
    }

    pub(crate) async fn handle_upload_chunk(
        &self,
        request_id: &str,
        params: UploadChunkParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut upload = self
            .uploads
            .get(&params.upload_id)
            .await
            .ok_or_else(|| HandlerError::MissingUpload(params.upload_id.clone()))?;

        if params.chunk_index != upload.next_chunk_index {
            return Err(HandlerError::InvalidChunkIndex {
                expected: upload.next_chunk_index,
                got: params.chunk_index,
            });
        }

        if params.offset != upload.next_offset {
            return Err(HandlerError::InvalidChunkOffset {
                expected: upload.next_offset,
                got: params.offset,
            });
        }

        let bytes = BASE64_STANDARD
            .decode(params.payload_base64)
            .map_err(|error| HandlerError::InvalidUploadChunkPayload(error.to_string()))?;
        upload.bytes.extend_from_slice(&bytes);
        upload.next_chunk_index = upload.next_chunk_index.saturating_add(1);
        upload.next_offset = upload.bytes.len() as u64;

        if upload.next_offset > upload.total_size {
            return Err(HandlerError::UploadSizeMismatch {
                expected: upload.total_size,
                got: upload.next_offset,
            });
        }

        self.uploads.save(upload.clone()).await;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::UploadChunkAccepted(UploadChunkAcceptedPayload {
                upload_id: upload.upload_id,
                received_chunk_index: params.chunk_index,
                received_bytes: upload.next_offset,
            }),
        ))
    }

    pub(crate) async fn handle_upload_complete(
        &self,
        request_id: &str,
        params: UploadCompleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let upload = self
            .uploads
            .get(&params.upload_id)
            .await
            .ok_or_else(|| HandlerError::MissingUpload(params.upload_id.clone()))?;

        if upload.bytes.len() as u64 != upload.total_size {
            return Err(HandlerError::UploadSizeMismatch {
                expected: upload.total_size,
                got: upload.bytes.len() as u64,
            });
        }

        match upload.target_kind {
            UploadTargetKind::CharacterCard => {
                let archive = CharacterArchive::from_chr_bytes(&upload.bytes)?;
                let summary = archive.summary();
                let character_id = summary.character_id.clone();

                if self.store.get_character(&character_id).await?.is_some() {
                    return Err(HandlerError::DuplicateCharacter(character_id));
                }

                self.store
                    .save_character(CharacterCardRecord {
                        character_id: summary.character_id.clone(),
                        content: archive.content.clone().into(),
                        cover_file_name: Some(archive.manifest.cover_path.clone()),
                        cover_mime_type: Some(store_cover_mime_type(archive.manifest.cover_mime_type)),
                        cover_bytes: Some(archive.cover_bytes.clone()),
                    })
                    .await?;
                self.uploads.delete(&upload.upload_id).await;

                Ok(JsonRpcResponseMessage::ok(
                    request_id,
                    None::<String>,
                    ResponseResult::CharacterCardUploaded(CharacterCardUploadedPayload {
                        character_id: summary.character_id.clone(),
                        character_summary: summary,
                    }),
                ))
            }
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

        let mut content = params.content;
        content.id = character_id.clone();
        let record = CharacterCardRecord {
            character_id,
            content: content.into(),
            cover_file_name: None,
            cover_mime_type: None,
            cover_bytes: None,
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
            ResponseResult::Character(Box::new(CharacterDetailPayload {
                character_id: record.character_id,
                content: (&record.content).into(),
                cover_file_name: record.cover_file_name,
                cover_mime_type: parse_cover_mime_type_option(record.cover_mime_type.as_deref()),
            })),
        ))
    }

    pub(crate) async fn handle_character_get_cover(
        &self,
        request_id: &str,
        params: CharacterGetCoverParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let record = self
            .store
            .get_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;
        let cover_file_name = record
            .cover_file_name
            .clone()
            .ok_or_else(|| HandlerError::MissingCharacterCover(params.character_id.clone()))?;
        let cover_mime_type = parse_cover_mime_type_option(record.cover_mime_type.as_deref())
            .ok_or_else(|| HandlerError::MissingCharacterCover(params.character_id.clone()))?;
        let cover_bytes = record
            .cover_bytes
            .clone()
            .ok_or_else(|| HandlerError::MissingCharacterCover(params.character_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharacterCover(Box::new(CharacterCoverPayload {
                character_id: record.character_id,
                cover_file_name,
                cover_mime_type,
                cover_base64: BASE64_STANDARD.encode(cover_bytes),
            })),
        ))
    }

    pub(crate) async fn handle_character_export_chr(
        &self,
        request_id: &str,
        params: CharacterExportChrParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let record = self
            .store
            .get_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;
        let cover_file_name = record
            .cover_file_name
            .clone()
            .ok_or_else(|| HandlerError::MissingCharacterCover(params.character_id.clone()))?;
        let cover_mime_type = parse_cover_mime_type_option(record.cover_mime_type.as_deref())
            .ok_or_else(|| HandlerError::MissingCharacterCover(params.character_id.clone()))?;
        let cover_bytes = record
            .cover_bytes
            .clone()
            .ok_or_else(|| HandlerError::MissingCharacterCover(params.character_id.clone()))?;
        let archive = CharacterArchive {
            manifest: CharacterArchiveManifest::new(
                record.character_id.clone(),
                cover_mime_type,
                cover_file_name,
            ),
            content: (&record.content).into(),
            cover_bytes,
        };
        let chr_bytes = archive.to_chr_bytes()?;
        let character_id = archive.content.id.clone();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharacterChrExport(Box::new(CharacterChrExportPayload {
                character_id: character_id.clone(),
                file_name: format!("{character_id}.chr"),
                content_type: CHARACTER_ARCHIVE_CONTENT_TYPE.to_owned(),
                chr_base64: BASE64_STANDARD.encode(chr_bytes),
            })),
        ))
    }

    pub(crate) async fn handle_character_set_cover(
        &self,
        request_id: &str,
        params: CharacterSetCoverParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut record = self
            .store
            .get_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;

        let cover_bytes = BASE64_STANDARD
            .decode(&params.cover_base64)
            .map_err(|error| HandlerError::InvalidCharacterCoverPayload(error.to_string()))?;
        if cover_bytes.is_empty() {
            return Err(HandlerError::InvalidCharacterCoverPayload(
                "cover bytes must not be empty".to_owned(),
            ));
        }

        let cover_file_name = format!("cover.{}", params.cover_mime_type.extension());
        record.cover_file_name = Some(cover_file_name.clone());
        record.cover_mime_type = Some(store_cover_mime_type(params.cover_mime_type));
        record.cover_bytes = Some(cover_bytes);
        self.store.save_character(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharacterCoverUpdated(CharacterCoverUpdatedPayload {
                character_id: record.character_id,
                cover_file_name,
                cover_mime_type: params.cover_mime_type,
            }),
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

        self.store
            .delete_character(&params.character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(params.character_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::CharacterDeleted(CharacterDeletedPayload {
                character_id: params.character_id,
            }),
        ))
    }
}

fn parse_cover_mime_type_option(value: Option<&str>) -> Option<CharacterCoverMimeType> {
    value.map(|mime_type| {
        serde_json::from_str(&format!("\"{mime_type}\""))
            .expect("stored cover mime type should deserialize")
    })
}

fn store_cover_mime_type(value: CharacterCoverMimeType) -> String {
    serde_json::to_string(&value)
        .expect("cover mime type should serialize")
        .trim_matches('"')
        .to_owned()
}

fn character_summary_payload_from_record(record: &CharacterCardRecord) -> CharacterCardSummaryPayload {
    CharacterCardSummaryPayload {
        character_id: record.character_id.clone(),
        name: record.content.name.clone(),
        personality: record.content.personality.clone(),
        style: record.content.style.clone(),
        tendencies: record.content.tendencies.clone(),
        cover_file_name: record.cover_file_name.clone(),
        cover_mime_type: parse_cover_mime_type_option(record.cover_mime_type.as_deref()),
    }
}
