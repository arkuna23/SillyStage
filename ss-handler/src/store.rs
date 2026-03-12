use std::collections::HashMap;

use protocol::UploadTargetKind;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct UploadRecord {
    pub upload_id: String,
    pub target_kind: UploadTargetKind,
    pub total_size: u64,
    pub next_chunk_index: u64,
    pub next_offset: u64,
    pub bytes: Vec<u8>,
}

#[derive(Default)]
pub struct UploadStore {
    uploads: RwLock<HashMap<String, UploadRecord>>,
}

impl UploadStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, upload_id: &str) -> Option<UploadRecord> {
        self.uploads.read().await.get(upload_id).cloned()
    }

    pub async fn save(&self, upload: UploadRecord) {
        self.uploads
            .write()
            .await
            .insert(upload.upload_id.clone(), upload);
    }

    pub async fn delete(&self, upload_id: &str) -> Option<UploadRecord> {
        self.uploads.write().await.remove(upload_id)
    }
}
