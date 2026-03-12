use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("store backend error: {0}")]
    Backend(String),
    #[error("store I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to serialize store JSON: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to deserialize store JSON: {0}")]
    Deserialize(serde_json::Error),
    #[error("invalid store id for filesystem path: {0}")]
    InvalidPathComponent(String),
    #[error("missing parent directory for path: {0}")]
    MissingParentDirectory(PathBuf),
}
