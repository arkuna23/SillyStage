use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use protocol::{
    CharacterArchive, CharacterCardUploadedPayload, JsonRpcResponseMessage, ResponseResult,
    UploadChunkAcceptedPayload, UploadChunkParams, UploadCompleteParams, UploadInitParams,
    UploadInitializedPayload, UploadTargetKind,
};

use crate::error::HandlerError;
use crate::store::{CharacterCardRecord, UploadRecord};

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
            file_name: params.file_name,
            content_type: params.content_type,
            total_size: params.total_size,
            sha256: params.sha256,
            next_chunk_index: 0,
            next_offset: 0,
            bytes: Vec::new(),
        };

        self.store.save_upload(record).await?;

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
            .store
            .get_upload(&params.upload_id)
            .await?
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

        self.store.save_upload(upload.clone()).await?;

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
            .store
            .get_upload(&params.upload_id)
            .await?
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
                        archive,
                        summary: summary.clone(),
                    })
                    .await?;
                self.store.delete_upload(&upload.upload_id).await?;

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
}
