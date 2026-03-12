use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use agents::actor::CharacterCard;
use async_stream::stream;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use engine::{
    AgentApiIdOverrides, AgentApiIds, Engine, EngineEvent, LlmApiRegistry, RuntimeState,
    SessionConfigMode, SessionEngineConfig, StoryResources, generate_story_graph,
    generate_story_plan,
};
use futures_core::Stream;
use futures_util::StreamExt;
use protocol::{
    CharacterArchive, CharacterCardUploadedPayload, ConfigGetGlobalParams,
    ConfigUpdateGlobalParams, CreateStoryResourcesParams, GenerateStoryParams,
    GenerateStoryPlanParams, JsonRpcRequestMessage, JsonRpcResponseMessage, RequestParams,
    ResponseResult, RunTurnParams, ServerEventMessage, SessionConfigPayload,
    SessionGetConfigParams, SessionStartedPayload, SessionUpdateConfigParams,
    StartSessionFromStoryParams, StoryGeneratedPayload, StoryPlannedPayload, StoryResourcesPayload,
    StreamEventBody, TurnCompletedPayload, TurnStreamAcceptedPayload,
    UpdatePlayerDescriptionParams, UpdateStoryResourcesParams, UploadChunkAcceptedPayload,
    UploadChunkParams, UploadCompleteParams, UploadInitParams, UploadInitializedPayload,
    UploadTargetKind,
};

use crate::error::HandlerError;
use crate::store::{
    CharacterCardRecord, HandlerStore, InMemoryHandlerStore, SessionRecord, StoryRecord,
    UploadRecord,
};

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
        validate_api_ids(&registry, &initial_global_config)?;

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
            RequestParams::ConfigGetGlobal(ConfigGetGlobalParams {}) => {
                self.handle_config_get_global(&request_id).await
            }
            RequestParams::ConfigUpdateGlobal(params) => {
                self.handle_config_update_global(&request_id, params).await
            }
            RequestParams::SessionGetConfig(SessionGetConfigParams {}) => {
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

    async fn handle_upload_init(
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

    async fn handle_upload_chunk(
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

    async fn handle_upload_complete(
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

    async fn handle_story_resources_create(
        &self,
        request_id: &str,
        params: CreateStoryResourcesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if params.character_ids.is_empty() {
            return Err(HandlerError::EmptyCharacterIds);
        }

        self.ensure_characters_exist(&params.character_ids).await?;

        let payload = StoryResourcesPayload {
            resource_id: self.id_generator.next("resource"),
            story_concept: params.story_concept,
            character_ids: params.character_ids,
            player_state_schema_seed: params.player_state_schema_seed,
            world_state_schema_seed: params.world_state_schema_seed,
            planned_story: params.planned_story,
        };

        self.store.save_story_resources(payload.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResourcesCreated(Box::new(payload)),
        ))
    }

    async fn handle_story_resources_update(
        &self,
        request_id: &str,
        params: UpdateStoryResourcesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut payload = self
            .store
            .get_story_resources(&params.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(params.resource_id.clone()))?;

        if let Some(story_concept) = params.story_concept {
            payload.story_concept = story_concept;
        }
        if let Some(character_ids) = params.character_ids {
            if character_ids.is_empty() {
                return Err(HandlerError::EmptyCharacterIds);
            }
            self.ensure_characters_exist(&character_ids).await?;
            payload.character_ids = character_ids;
        }
        if let Some(player_state_schema_seed) = params.player_state_schema_seed {
            payload.player_state_schema_seed = player_state_schema_seed;
        }
        if let Some(world_state_schema_seed) = params.world_state_schema_seed {
            payload.world_state_schema_seed = Some(world_state_schema_seed);
        }
        if let Some(planned_story) = params.planned_story {
            payload.planned_story = Some(planned_story);
        }

        self.store.save_story_resources(payload.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResourcesUpdated(Box::new(payload)),
        ))
    }

    async fn handle_story_generate_plan(
        &self,
        request_id: &str,
        params: GenerateStoryPlanParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let resource = self
            .store
            .get_story_resources(&params.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(params.resource_id.clone()))?;
        let api_ids = self
            .load_global_config()
            .await?
            .apply_overrides(&AgentApiIdOverrides {
                planner_api_id: params.planner_api_id,
                ..AgentApiIdOverrides::default()
            });
        validate_api_ids(&self.registry, &api_ids)?;

        let story_resources = self.build_engine_story_resources(&resource).await?;
        let generation_configs = self.registry.build_story_generation_configs(&api_ids)?;
        let response = generate_story_plan(&generation_configs, &story_resources).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryPlanned(StoryPlannedPayload {
                resource_id: resource.resource_id,
                story_script: response.story_script,
            }),
        ))
    }

    async fn handle_story_generate(
        &self,
        request_id: &str,
        params: GenerateStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let resource = self
            .store
            .get_story_resources(&params.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(params.resource_id.clone()))?;
        let api_ids = self
            .load_global_config()
            .await?
            .apply_overrides(&AgentApiIdOverrides {
                architect_api_id: params.architect_api_id,
                ..AgentApiIdOverrides::default()
            });
        validate_api_ids(&self.registry, &api_ids)?;

        let story_resources = self.build_engine_story_resources(&resource).await?;
        let generation_configs = self.registry.build_story_generation_configs(&api_ids)?;
        let response = generate_story_graph(&generation_configs, &story_resources).await?;
        let story_id = self.id_generator.next("story");

        let payload = StoryGeneratedPayload {
            resource_id: resource.resource_id.clone(),
            story_id: story_id.clone(),
            graph: response.graph,
            world_state_schema: response.world_state_schema,
            player_state_schema: response.player_state_schema,
            introduction: response.introduction,
        };

        self.store
            .save_story(StoryRecord {
                story_id,
                resource_id: resource.resource_id,
                generated: payload.clone(),
            })
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryGenerated(Box::new(payload)),
        ))
    }

    async fn handle_story_start_session(
        &self,
        request_id: &str,
        params: StartSessionFromStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let story = self
            .store
            .get_story(&params.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(params.story_id.clone()))?;
        let global_config = self.load_global_config().await?;
        let session_config = match params.config_mode {
            SessionConfigMode::UseGlobal => SessionEngineConfig::use_global(),
            SessionConfigMode::UseSession => SessionEngineConfig::use_session(
                params
                    .session_api_ids
                    .unwrap_or_else(|| global_config.clone()),
            ),
        };
        let effective_api_ids = effective_session_api_ids(&session_config, &global_config);
        validate_api_ids(&self.registry, &effective_api_ids)?;

        let characters = self
            .load_story_character_cards(&story.resource_id)
            .await?
            .into_iter()
            .map(|record| record.archive.content.into())
            .collect::<Vec<CharacterCard>>();
        let runtime_state = RuntimeState::from_story_graph(
            &story.story_id,
            story.generated.graph.clone(),
            characters,
            params.player_description,
            story.generated.player_state_schema.clone(),
        )?;
        let snapshot = runtime_state.snapshot();
        let config_payload = build_session_config_payload(&session_config, &global_config);
        let session_id = self.id_generator.next("session");

        self.store
            .save_session(SessionRecord {
                session_id: session_id.clone(),
                story_id: story.story_id,
                snapshot: snapshot.clone(),
                config: session_config,
            })
            .await?;

        let character_summaries = self
            .load_story_character_cards(&story.resource_id)
            .await?
            .into_iter()
            .map(|record| record.summary)
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionStarted(Box::new(SessionStartedPayload {
                snapshot,
                character_summaries,
                config: config_payload,
            })),
        ))
    }

    async fn handle_session_run_turn(
        &self,
        request_id: String,
        session_id: Option<String>,
        params: RunTurnParams,
    ) -> HandlerReply<'a> {
        let session_id = match session_id {
            Some(session_id) => session_id,
            None => {
                return HandlerReply::Unary(JsonRpcResponseMessage::err(
                    request_id,
                    None::<String>,
                    HandlerError::MissingSessionId.to_error_payload(),
                ));
            }
        };

        let setup = self
            .prepare_turn_stream_state(
                &session_id,
                params.player_input.clone(),
                params.api_overrides,
            )
            .await;

        let (mut engine, session_record, request_runtime_input) = match setup {
            Ok(value) => value,
            Err(error) => {
                return HandlerReply::Unary(JsonRpcResponseMessage::err(
                    request_id,
                    Some(session_id),
                    error.to_error_payload(),
                ));
            }
        };

        let ack = JsonRpcResponseMessage::ok(
            &request_id,
            Some(session_id.clone()),
            ResponseResult::TurnStreamAccepted(TurnStreamAcceptedPayload::default()),
        );
        let store = Arc::clone(&self.store);

        let events = stream! {
            let mut sequence = 0_u64;
            yield ServerEventMessage::started(
                request_id.clone(),
                Some(session_id.clone()),
                sequence,
            );
            sequence = sequence.saturating_add(1);

            let mut engine_stream = match engine.run_turn_stream(&request_runtime_input).await {
                Ok(stream) => stream,
                Err(error) => {
                    yield ServerEventMessage::failed(
                        request_id.clone(),
                        Some(session_id.clone()),
                        sequence,
                        HandlerError::Engine(error).to_error_payload(),
                    );
                    return;
                }
            };

            while let Some(event) = engine_stream.next().await {
                match event {
                    EngineEvent::TurnStarted { next_turn_index, player_input } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::TurnStarted {
                                next_turn_index,
                                player_input,
                            },
                        );
                    }
                    EngineEvent::PlayerInputRecorded { entry, snapshot } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::PlayerInputRecorded { entry, snapshot },
                        );
                    }
                    EngineEvent::KeeperApplied { phase, update, snapshot } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::KeeperApplied {
                                phase,
                                update,
                                snapshot,
                            },
                        );
                    }
                    EngineEvent::DirectorCompleted { result, snapshot } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::DirectorCompleted {
                                result,
                                snapshot,
                            },
                        );
                    }
                    EngineEvent::NarratorStarted { beat_index, purpose } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorStarted { beat_index, purpose },
                        );
                    }
                    EngineEvent::NarratorTextDelta { beat_index, purpose, delta } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorTextDelta {
                                beat_index,
                                purpose,
                                delta,
                            },
                        );
                    }
                    EngineEvent::NarratorCompleted { beat_index, purpose, response } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::NarratorCompleted {
                                beat_index,
                                purpose,
                                response,
                            },
                        );
                    }
                    EngineEvent::ActorStarted { beat_index, speaker_id, purpose } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorStarted {
                                beat_index,
                                speaker_id,
                                purpose,
                            },
                        );
                    }
                    EngineEvent::ActorThoughtDelta { beat_index, speaker_id, delta } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorThoughtDelta {
                                beat_index,
                                speaker_id,
                                delta,
                            },
                        );
                    }
                    EngineEvent::ActorActionComplete { beat_index, speaker_id, text } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorActionComplete {
                                beat_index,
                                speaker_id,
                                text,
                            },
                        );
                    }
                    EngineEvent::ActorDialogueDelta { beat_index, speaker_id, delta } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorDialogueDelta {
                                beat_index,
                                speaker_id,
                                delta,
                            },
                        );
                    }
                    EngineEvent::ActorCompleted { beat_index, speaker_id, purpose, response } => {
                        yield ServerEventMessage::event(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            StreamEventBody::ActorCompleted {
                                beat_index,
                                speaker_id,
                                purpose,
                                response,
                            },
                        );
                    }
                    EngineEvent::TurnCompleted { result } => {
                        let mut updated_session = session_record.clone();
                        updated_session.snapshot = result.snapshot.clone();

                        if let Err(error) = store.save_session(updated_session).await {
                            yield ServerEventMessage::failed(
                                request_id.clone(),
                                Some(session_id.clone()),
                                sequence,
                                HandlerError::Store(error).to_error_payload(),
                            );
                            return;
                        }

                        yield ServerEventMessage::completed(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            ResponseResult::TurnCompleted(Box::new(TurnCompletedPayload {
                                result: *result,
                            })),
                        );
                        return;
                    }
                    EngineEvent::TurnFailed { stage, error, snapshot } => {
                        let mut updated_session = session_record.clone();
                        updated_session.snapshot = (*snapshot).clone();

                        if let Err(store_error) = store.save_session(updated_session).await {
                            yield ServerEventMessage::failed(
                                request_id.clone(),
                                Some(session_id.clone()),
                                sequence,
                                HandlerError::Store(store_error).to_error_payload(),
                            );
                            return;
                        }

                        yield ServerEventMessage::failed(
                            request_id.clone(),
                            Some(session_id.clone()),
                            sequence,
                            HandlerError::Engine(engine::EngineError::TurnFailed {
                                stage,
                                message: error,
                            })
                            .to_error_payload(),
                        );
                        return;
                    }
                }

                sequence = sequence.saturating_add(1);
            }
        };

        HandlerReply::Stream {
            ack,
            events: Box::pin(events),
        }
    }

    async fn handle_session_update_player_description(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: UpdatePlayerDescriptionParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        session.snapshot.player_description = params.player_description;
        let snapshot = session.snapshot.clone();
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::PlayerDescriptionUpdated(Box::new(
                protocol::PlayerDescriptionUpdatedPayload { snapshot },
            )),
        ))
    }

    async fn handle_session_get_runtime_snapshot(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::RuntimeSnapshot(Box::new(protocol::RuntimeSnapshotPayload {
                snapshot: session.snapshot,
            })),
        ))
    }

    async fn handle_config_get_global(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_ids = self.load_global_config().await?;
        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::GlobalConfig(protocol::GlobalConfigPayload { api_ids }),
        ))
    }

    async fn handle_config_update_global(
        &self,
        request_id: &str,
        params: ConfigUpdateGlobalParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let current = self.load_global_config().await?;
        let updated = current.apply_overrides(&params.api_overrides);
        validate_api_ids(&self.registry, &updated)?;
        self.store.set_global_config(updated.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::GlobalConfig(protocol::GlobalConfigPayload { api_ids: updated }),
        ))
    }

    async fn handle_session_get_config(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let global = self.load_global_config().await?;
        let payload = build_session_config_payload(&session.config, &global);

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionConfig(payload),
        ))
    }

    async fn handle_session_update_config(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: SessionUpdateConfigParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let global = self.load_global_config().await?;
        let new_config = match params.mode {
            SessionConfigMode::UseGlobal => SessionEngineConfig::use_global(),
            SessionConfigMode::UseSession => {
                let base_api_ids = params.session_api_ids.unwrap_or_else(|| {
                    session
                        .config
                        .session_api_ids
                        .clone()
                        .unwrap_or_else(|| effective_session_api_ids(&session.config, &global))
                });
                let merged = params.api_overrides.unwrap_or_default();
                SessionEngineConfig::use_session(base_api_ids.apply_overrides(&merged))
            }
        };
        let effective = effective_session_api_ids(&new_config, &global);
        validate_api_ids(&self.registry, &effective)?;

        session.config = new_config.clone();
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionConfig(build_session_config_payload(&new_config, &global)),
        ))
    }

    async fn load_global_config(&self) -> Result<AgentApiIds, HandlerError> {
        self.store
            .get_global_config()
            .await?
            .ok_or(HandlerError::MissingGlobalConfig)
    }

    async fn ensure_characters_exist(&self, character_ids: &[String]) -> Result<(), HandlerError> {
        for character_id in character_ids {
            if self.store.get_character(character_id).await?.is_none() {
                return Err(HandlerError::MissingCharacter(character_id.clone()));
            }
        }
        Ok(())
    }

    async fn load_story_character_cards(
        &self,
        resource_id: &str,
    ) -> Result<Vec<CharacterCardRecord>, HandlerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(resource_id.to_owned()))?;

        let mut records = Vec::with_capacity(resource.character_ids.len());
        for character_id in &resource.character_ids {
            let record = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| HandlerError::MissingCharacter(character_id.clone()))?;
            records.push(record);
        }
        Ok(records)
    }

    async fn build_engine_story_resources(
        &self,
        resource: &StoryResourcesPayload,
    ) -> Result<StoryResources, HandlerError> {
        let mut cards = Vec::with_capacity(resource.character_ids.len());
        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| HandlerError::MissingCharacter(character_id.clone()))?;
            cards.push(CharacterCard::from(character.archive.content));
        }

        let mut story_resources = StoryResources::new(
            resource.resource_id.clone(),
            resource.story_concept.clone(),
            cards,
            resource.player_state_schema_seed.clone(),
        )?;

        if let Some(planned_story) = &resource.planned_story {
            story_resources = story_resources.with_planned_story(planned_story.clone());
        }
        if let Some(world_state_schema_seed) = &resource.world_state_schema_seed {
            story_resources =
                story_resources.with_world_state_schema_seed(world_state_schema_seed.clone());
        }

        Ok(story_resources)
    }

    async fn prepare_turn_stream_state(
        &self,
        session_id: &str,
        player_input: String,
        api_overrides: Option<AgentApiIdOverrides>,
    ) -> Result<(Engine<'a>, SessionRecord, String), HandlerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.to_owned()))?;
        let story = self
            .store
            .get_story(&session.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(session.story_id.clone()))?;
        let character_cards = self
            .load_story_character_cards(&story.resource_id)
            .await?
            .into_iter()
            .map(|record| CharacterCard::from(record.archive.content))
            .collect::<Vec<_>>();
        let runtime_state = RuntimeState::from_snapshot(
            &story.story_id,
            story::runtime_graph::RuntimeStoryGraph::from_story_graph(
                story.generated.graph.clone(),
            )
            .map_err(engine::RuntimeError::GraphBuild)?,
            character_cards,
            story.generated.player_state_schema.clone(),
            session.snapshot.clone(),
        )?;
        let global = self.load_global_config().await?;
        let effective = effective_session_api_ids(&session.config, &global)
            .apply_overrides(&api_overrides.unwrap_or_default());
        validate_api_ids(&self.registry, &effective)?;
        let runtime_configs = self.registry.build_runtime_configs(&effective)?;
        let engine = Engine::new(runtime_configs, runtime_state)?;

        Ok((engine, session, player_input))
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

fn validate_api_ids(
    registry: &LlmApiRegistry<'_>,
    api_ids: &AgentApiIds,
) -> Result<(), HandlerError> {
    registry.build_story_generation_configs(api_ids)?;
    registry.build_runtime_configs(api_ids)?;
    Ok(())
}

fn effective_session_api_ids(config: &SessionEngineConfig, global: &AgentApiIds) -> AgentApiIds {
    match config.mode {
        SessionConfigMode::UseGlobal => global.clone(),
        SessionConfigMode::UseSession => config
            .session_api_ids
            .clone()
            .unwrap_or_else(|| global.clone()),
    }
}

fn build_session_config_payload(
    config: &SessionEngineConfig,
    global: &AgentApiIds,
) -> SessionConfigPayload {
    SessionConfigPayload {
        mode: config.mode,
        session_api_ids: config.session_api_ids.clone(),
        effective_api_ids: effective_session_api_ids(config, global),
    }
}
