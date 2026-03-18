use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
use store::{ApiRecord, SessionBindingConfig, Store};

use crate::{EngineEvent, LlmApiRegistry};

mod error;
mod replies;
mod resolve;
mod session;
mod session_characters;
mod story_generation;
mod util;

pub use error::ManagerError;

const DEFAULT_ARCHITECT_CHUNK_NODE_COUNT: usize = 4;
const DEFAULT_ARCHITECT_INIT_MAX_TOKENS: u32 = 8_192;
const DEFAULT_ARCHITECT_CONTINUE_MAX_TOKENS: u32 = 4_096;
const DEFAULT_ARCHITECT_TEMPERATURE: f32 = 0.0;
const DEFAULT_REPLY_HISTORY_LIMIT: usize = 8;

pub type ManagedTurnStream<'a> =
    Pin<Box<dyn Stream<Item = Result<EngineEvent, ManagerError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct ResolvedSessionConfig {
    pub binding: SessionBindingConfig,
}

#[derive(Debug, Clone)]
pub struct SessionCharacterUpdate {
    pub display_name: String,
    pub personality: String,
    pub style: String,
    pub system_prompt: String,
}

pub struct EngineManager {
    store: Arc<dyn Store>,
    registry: LlmApiRegistry,
}

#[derive(Debug, Clone)]
struct ResolvedApiGroup {
    planner: ApiRecord,
    architect: ApiRecord,
    director: ApiRecord,
    actor: ApiRecord,
    narrator: ApiRecord,
    keeper: ApiRecord,
    replyer: ApiRecord,
}

impl EngineManager {
    pub async fn new(
        store: Arc<dyn Store>,
        registry: LlmApiRegistry,
    ) -> Result<Self, ManagerError> {
        Ok(Self { store, registry })
    }

    pub fn store(&self) -> &Arc<dyn Store> {
        &self.store
    }
}
