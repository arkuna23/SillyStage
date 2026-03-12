mod config;
mod session;
mod story;
mod upload;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use engine::{AgentApiIds, EngineManager, LlmApiRegistry};
use futures_core::Stream;
use protocol::{JsonRpcRequestMessage, JsonRpcResponseMessage, RequestParams, ServerEventMessage};
use store::{InMemoryStore, Store};

use crate::error::HandlerError;
use crate::store::UploadStore;

pub type HandlerEventStream<'a> = Pin<Box<dyn Stream<Item = ServerEventMessage> + Send + 'a>>;

pub enum HandlerReply<'a> {
    Unary(JsonRpcResponseMessage),
    Stream {
        ack: JsonRpcResponseMessage,
        events: HandlerEventStream<'a>,
    },
}

pub struct Handler<'a> {
    store: Arc<dyn Store>,
    manager: EngineManager<'a>,
    uploads: UploadStore,
    id_generator: IdGenerator,
}

impl<'a> Handler<'a> {
    pub async fn new(
        store: Arc<dyn Store>,
        registry: LlmApiRegistry<'a>,
        initial_global_config: AgentApiIds,
    ) -> Result<Self, HandlerError> {
        let manager =
            EngineManager::new(Arc::clone(&store), registry, initial_global_config).await?;

        Ok(Self {
            store,
            manager,
            uploads: UploadStore::new(),
            id_generator: IdGenerator::default(),
        })
    }

    pub async fn with_in_memory_store(
        registry: LlmApiRegistry<'a>,
        initial_global_config: AgentApiIds,
    ) -> Result<Self, HandlerError> {
        Self::new(
            Arc::new(InMemoryStore::new()),
            registry,
            initial_global_config,
        )
        .await
    }

    pub async fn handle(&self, request: JsonRpcRequestMessage) -> HandlerReply<'a> {
        let request_id = request.id.clone();
        let session_id = request.session_id.clone();

        let result = match request.params {
            RequestParams::UploadInit(params) => self.handle_upload_init(&request_id, params).await,
            RequestParams::UploadChunk(params) => {
                self.handle_upload_chunk(&request_id, params).await
            }
            RequestParams::UploadComplete(params) => {
                self.handle_upload_complete(&request_id, params).await
            }
            RequestParams::CharacterGet(params) => {
                self.handle_character_get(&request_id, params).await
            }
            RequestParams::CharacterList(_) => self.handle_character_list(&request_id).await,
            RequestParams::CharacterDelete(params) => {
                self.handle_character_delete(&request_id, params).await
            }
            RequestParams::StoryResourcesCreate(params) => {
                self.handle_story_resources_create(&request_id, params)
                    .await
            }
            RequestParams::StoryResourcesGet(params) => {
                self.handle_story_resources_get(&request_id, params).await
            }
            RequestParams::StoryResourcesList(_) => {
                self.handle_story_resources_list(&request_id).await
            }
            RequestParams::StoryResourcesUpdate(params) => {
                self.handle_story_resources_update(&request_id, params)
                    .await
            }
            RequestParams::StoryResourcesDelete(params) => {
                self.handle_story_resources_delete(&request_id, params)
                    .await
            }
            RequestParams::StoryGeneratePlan(params) => {
                self.handle_story_generate_plan(&request_id, params).await
            }
            RequestParams::StoryGenerate(params) => {
                self.handle_story_generate(&request_id, params).await
            }
            RequestParams::StoryGet(params) => self.handle_story_get(&request_id, params).await,
            RequestParams::StoryList(_) => self.handle_story_list(&request_id).await,
            RequestParams::StoryDelete(params) => {
                self.handle_story_delete(&request_id, params).await
            }
            RequestParams::StoryStartSession(params) => {
                self.handle_story_start_session(&request_id, params).await
            }
            RequestParams::SessionGet(_) => {
                self.handle_session_get(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionList(_) => self.handle_session_list(&request_id).await,
            RequestParams::SessionDelete(_) => {
                self.handle_session_delete(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionRunTurn(params) => {
                return self
                    .handle_session_run_turn(request_id, session_id, params)
                    .await;
            }
            RequestParams::SessionUpdatePlayerDescription(params) => {
                self.handle_session_update_player_description(
                    &request_id,
                    session_id.clone(),
                    params,
                )
                .await
            }
            RequestParams::SessionGetRuntimeSnapshot(_) => {
                self.handle_session_get_runtime_snapshot(&request_id, session_id.clone())
                    .await
            }
            RequestParams::ConfigGetGlobal(_) => self.handle_config_get_global(&request_id).await,
            RequestParams::ConfigUpdateGlobal(params) => {
                self.handle_config_update_global(&request_id, params).await
            }
            RequestParams::SessionGetConfig(_) => {
                self.handle_session_get_config(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionUpdateConfig(params) => {
                self.handle_session_update_config(&request_id, session_id.clone(), params)
                    .await
            }
        };

        match result {
            Ok(response) => HandlerReply::Unary(response),
            Err(error) => HandlerReply::Unary(JsonRpcResponseMessage::err(
                request_id,
                session_id,
                error.to_error_payload(),
            )),
        }
    }
}

#[derive(Default)]
struct IdGenerator {
    next: AtomicU64,
}

impl IdGenerator {
    fn next(&self, prefix: &str) -> String {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{id}")
    }
}

fn require_session_id(session_id: Option<String>) -> Result<String, HandlerError> {
    session_id.ok_or(HandlerError::MissingSessionId)
}
