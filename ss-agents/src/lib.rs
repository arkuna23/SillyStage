pub mod architect;
pub mod director;
pub mod actor;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("builder error: {0}")]
    Builder(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;
