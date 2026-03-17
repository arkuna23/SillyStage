pub mod actor;
pub mod architect;
pub mod director;
pub mod keeper;
pub mod narrator;
pub mod planner;
mod prompt;
pub mod replyer;
pub use prompt::SystemPromptEntry;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("builder error: {0}")]
    Builder(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
