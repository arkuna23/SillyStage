use crate::config::ConfigError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Handler(#[from] handler::HandlerError),
    #[error(transparent)]
    Store(#[from] store::StoreError),
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
