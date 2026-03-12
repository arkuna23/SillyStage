mod config;
mod session;
mod story;
mod upload;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use engine::{AgentApiIds, LlmApiRegistry};
use futures_core::Stream;
use protocol::{JsonRpcRequestMessage, JsonRpcResponseMessage, RequestParams, ServerEventMessage};

use crate::error::HandlerError;
use crate::store::{HandlerStore, InMemoryHandlerStore};

pub type HandlerEventStream<'a> = Pin<Box<dyn Stream<Item = ServerEventMessage> + Send + 'a>>;

pub enum HandlerReply<'a> {
    Unary(JsonRpcResponseMessage),
    Stream {
        ack: JsonRpcResponseMessage,
        events: HandlerEventStream<'a>,
    },
}

pub struct Handler<'a> {
    store: Arc<dyn HandlerStore>,
    registry: LlmApiRegistry<'a>,
    id_generator: IdGenerator,
}

impl<'a> Handler<'a> {
    pub async fn new(
        store: Arc<dyn HandlerStore>,
        registry: LlmApiRegistry<'a>,
        initial_global_config: AgentApiIds,
    ) -> Result<Self, HandlerError> {
        config::validate_api_ids(&registry, &initial_global_config)?;

        if store.get_global_config().await?.is_none() {
            store.set_global_config(initial_global_config).await?;
        }

        Ok(Self {
            store,
            registry,
            id_generator: IdGenerator::default(),
        })
    }

    pub async fn with_in_memory_store(
        registry: LlmApiRegistry<'a>,
        initial_global_config: AgentApiIds,
    ) -> Result<Self, HandlerError> {
        Self::new(
            Arc::new(InMemoryHandlerStore::new()),
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
            RequestParams::StoryResourcesCreate(params) => {
                self.handle_story_resources_create(&request_id, params)
                    .await
            }
            RequestParams::StoryResourcesUpdate(params) => {
                self.handle_story_resources_update(&request_id, params)
                    .await
            }
            RequestParams::StoryGeneratePlan(params) => {
                self.handle_story_generate_plan(&request_id, params).await
            }
            RequestParams::StoryGenerate(params) => {
                self.handle_story_generate(&request_id, params).await
            }
            RequestParams::StoryStartSession(params) => {
                self.handle_story_start_session(&request_id, params).await
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
