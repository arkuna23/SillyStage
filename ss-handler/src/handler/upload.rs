use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use protocol::{
    CharacterArchive, CharacterCardUploadedPayload, CharacterDeleteParams, CharacterDeletedPayload,
    CharacterDetailPayload, CharacterGetParams, CharactersListedPayload, JsonRpcResponseMessage,
    ResponseResult, UploadChunkAcceptedPayload, UploadChunkParams, UploadCompleteParams,
    UploadInitParams, UploadInitializedPayload, UploadTargetKind,
};
use store::CharacterCardRecord;

use crate::error::HandlerError;
use crate::store::UploadRecord;

use super::Handler;

impl<'a> Handler<'a> {
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
                        cover_file_name: archive.manifest.cover_path.clone(),
                        cover_mime_type: serde_json::to_string(&archive.manifest.cover_mime_type)
                            .expect("cover mime type should serialize")
                            .trim_matches('"')
                            .to_owned(),
                        cover_bytes: archive.cover_bytes.clone(),
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
                cover_mime_type: serde_json::from_str(&format!("\"{}\"", record.cover_mime_type))
                    .expect("stored cover mime type should deserialize"),
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
            .map(|record| protocol::CharacterCardSummaryPayload {
                character_id: record.character_id,
                name: record.content.name,
                personality: record.content.personality,
                style: record.content.style,
                tendencies: record.content.tendencies,
                cover_file_name: record.cover_file_name,
                cover_mime_type: serde_json::from_str(&format!("\"{}\"", record.cover_mime_type))
                    .expect("stored cover mime type should deserialize"),
            })
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
