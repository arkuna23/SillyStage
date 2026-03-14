use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agents::actor::CharacterCard;
use async_stream::stream;
use futures_core::Stream;
use futures_util::StreamExt;
use state::{PlayerStateSchema, WorldStateSchema};
use store::{
    AgentApiIdOverrides, AgentApiIds, CharacterCardRecord, LlmApiRecord, RuntimeSnapshot,
    SchemaRecord, SessionConfigMode, SessionEngineConfig, SessionRecord, Store, StoreError,
    StoryRecord, StoryResourcesRecord,
};

use crate::{
    Engine, EngineError, EngineEvent, LlmApiRegistry, RegistryError, RuntimeError, RuntimeState,
    StoryResources, generate_story_graph, generate_story_plan,
};

pub type ManagedTurnStream<'a> =
    Pin<Box<dyn Stream<Item = Result<EngineEvent, ManagerError>> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct ResolvedSessionConfig {
    pub config: SessionEngineConfig,
    pub effective_api_ids: AgentApiIds,
}

pub struct EngineManager {
    store: Arc<dyn Store>,
    registry: LlmApiRegistry,
}

impl EngineManager {
    pub async fn new(
        store: Arc<dyn Store>,
        registry: LlmApiRegistry,
        initial_global_config: AgentApiIds,
    ) -> Result<Self, ManagerError> {
        validate_api_ids(&registry, &initial_global_config)?;

        if store.get_global_config().await?.is_none() {
            store.set_global_config(initial_global_config).await?;
        }

        Ok(Self { store, registry })
    }

    pub fn store(&self) -> &Arc<dyn Store> {
        &self.store
    }

    pub fn upsert_llm_api_record(&self, record: &LlmApiRecord) -> Result<(), ManagerError> {
        self.registry.upsert_record(record)?;
        Ok(())
    }

    pub fn remove_llm_api_record(&self, api_id: &str) {
        self.registry.remove(api_id);
    }

    pub async fn get_global_config(&self) -> Result<AgentApiIds, ManagerError> {
        self.store
            .get_global_config()
            .await?
            .ok_or(ManagerError::MissingGlobalConfig)
    }

    pub async fn update_global_config(
        &self,
        overrides: AgentApiIdOverrides,
    ) -> Result<AgentApiIds, ManagerError> {
        let current = self.get_global_config().await?;
        let updated = current.apply_overrides(&overrides);
        validate_api_ids(&self.registry, &updated)?;
        self.store.set_global_config(updated.clone()).await?;
        Ok(updated)
    }

    pub async fn generate_story_plan(
        &self,
        resource_id: &str,
        planner_api_id: Option<String>,
    ) -> Result<agents::planner::PlannerResponse, ManagerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;
        let api_ids = self
            .get_global_config()
            .await?
            .apply_overrides(&AgentApiIdOverrides {
                planner_api_id,
                ..AgentApiIdOverrides::default()
            });
        validate_api_ids(&self.registry, &api_ids)?;

        let story_resources = self.build_engine_story_resources(&resource).await?;
        let generation_configs = self.registry.build_story_generation_configs(&api_ids)?;
        generate_story_plan(&generation_configs, &story_resources)
            .await
            .map_err(ManagerError::from)
    }

    pub async fn generate_story(
        &self,
        resource_id: &str,
        display_name: Option<String>,
        architect_api_id: Option<String>,
    ) -> Result<StoryRecord, ManagerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;
        let api_ids = self
            .get_global_config()
            .await?
            .apply_overrides(&AgentApiIdOverrides {
                architect_api_id,
                ..AgentApiIdOverrides::default()
            });
        validate_api_ids(&self.registry, &api_ids)?;

        let story_resources = self.build_engine_story_resources(&resource).await?;
        let generation_configs = self.registry.build_story_generation_configs(&api_ids)?;
        let story_id = format!("story-{}", self.store.list_stories().await?.len());
        let response = generate_story_graph(&generation_configs, &story_resources).await?;
        let world_schema = SchemaRecord {
            schema_id: format!("schema-{}", self.store.list_schemas().await?.len()),
            display_name: format!("{story_id} world schema"),
            tags: vec!["world".to_owned(), "generated".to_owned(), story_id.clone()],
            fields: response.world_state_schema.fields.clone(),
        };
        self.store.save_schema(world_schema.clone()).await?;

        let player_schema = SchemaRecord {
            schema_id: format!("schema-{}", self.store.list_schemas().await?.len()),
            display_name: format!("{story_id} player schema"),
            tags: vec![
                "player".to_owned(),
                "generated".to_owned(),
                story_id.clone(),
            ],
            fields: response.player_state_schema.fields.clone(),
        };
        self.store.save_schema(player_schema.clone()).await?;

        let now = now_timestamp_ms();
        let story = StoryRecord {
            story_id: story_id.clone(),
            display_name: display_name.unwrap_or_else(|| resource.story_concept.clone()),
            resource_id: resource.resource_id,
            graph: response.graph,
            world_schema_id: world_schema.schema_id,
            player_schema_id: player_schema.schema_id,
            introduction: response.introduction,
            created_at_ms: Some(now),
            updated_at_ms: Some(now),
        };

        self.store.save_story(story.clone()).await?;
        Ok(story)
    }

    pub async fn start_session_from_story(
        &self,
        story_id: &str,
        display_name: Option<String>,
        player_profile_id: Option<String>,
        config_mode: SessionConfigMode,
        session_api_ids: Option<AgentApiIds>,
    ) -> Result<SessionRecord, ManagerError> {
        let story = self
            .store
            .get_story(story_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStory(story_id.to_owned()))?;
        let global_config = self.get_global_config().await?;
        let session_config = match config_mode {
            SessionConfigMode::UseGlobal => SessionEngineConfig::use_global(),
            SessionConfigMode::UseSession => SessionEngineConfig::use_session(
                session_api_ids.unwrap_or_else(|| global_config.clone()),
            ),
        };
        let effective = effective_session_api_ids(&session_config, &global_config);
        validate_api_ids(&self.registry, &effective)?;

        let player_description = match &player_profile_id {
            Some(player_profile_id) => {
                self.store
                    .get_player_profile(player_profile_id)
                    .await?
                    .ok_or_else(|| ManagerError::MissingPlayerProfile(player_profile_id.clone()))?
                    .description
            }
            None => String::new(),
        };
        let runtime_state = self
            .build_runtime_state_from_story(&story, player_description)
            .await?;
        let session_id = format!("session-{}", self.store.list_sessions().await?.len());
        let now = now_timestamp_ms();
        let session = SessionRecord {
            session_id,
            display_name: display_name.unwrap_or_else(|| story.display_name.clone()),
            story_id: story.story_id,
            player_profile_id,
            player_schema_id: story.player_schema_id,
            snapshot: runtime_state.snapshot(),
            config: session_config,
            created_at_ms: Some(now),
            updated_at_ms: Some(now),
        };

        self.store.save_session(session.clone()).await?;
        Ok(session)
    }

    pub async fn get_runtime_snapshot(
        &self,
        session_id: &str,
    ) -> Result<RuntimeSnapshot, ManagerError> {
        self.store
            .get_session(session_id)
            .await?
            .map(|session| session.snapshot)
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))
    }

    pub async fn update_player_description(
        &self,
        session_id: &str,
        player_description: String,
    ) -> Result<RuntimeSnapshot, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session.player_profile_id = None;
        session.snapshot.player_description = player_description;
        session.updated_at_ms = Some(now_timestamp_ms());
        let snapshot = session.snapshot.clone();
        self.store.save_session(session).await?;
        Ok(snapshot)
    }

    pub async fn set_player_profile(
        &self,
        session_id: &str,
        player_profile_id: Option<String>,
    ) -> Result<SessionRecord, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;

        let player_description = match &player_profile_id {
            Some(player_profile_id) => {
                self.store
                    .get_player_profile(player_profile_id)
                    .await?
                    .ok_or_else(|| ManagerError::MissingPlayerProfile(player_profile_id.clone()))?
                    .description
            }
            None => String::new(),
        };

        session.player_profile_id = player_profile_id;
        session.snapshot.player_description = player_description;
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;
        Ok(session)
    }

    pub async fn get_resolved_session_config(
        &self,
        session_id: &str,
    ) -> Result<ResolvedSessionConfig, ManagerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let global = self.get_global_config().await?;
        Ok(ResolvedSessionConfig {
            effective_api_ids: effective_session_api_ids(&session.config, &global),
            config: session.config,
        })
    }

    pub async fn update_session_config(
        &self,
        session_id: &str,
        mode: SessionConfigMode,
        session_api_ids: Option<AgentApiIds>,
        api_overrides: Option<AgentApiIdOverrides>,
    ) -> Result<ResolvedSessionConfig, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let global = self.get_global_config().await?;
        let new_config = match mode {
            SessionConfigMode::UseGlobal => SessionEngineConfig::use_global(),
            SessionConfigMode::UseSession => {
                let base_api_ids = session_api_ids.unwrap_or_else(|| {
                    session
                        .config
                        .session_api_ids
                        .clone()
                        .unwrap_or_else(|| effective_session_api_ids(&session.config, &global))
                });
                let merged = api_overrides.unwrap_or_default();
                SessionEngineConfig::use_session(base_api_ids.apply_overrides(&merged))
            }
        };
        let effective = effective_session_api_ids(&new_config, &global);
        validate_api_ids(&self.registry, &effective)?;

        session.config = new_config.clone();
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session).await?;

        Ok(ResolvedSessionConfig {
            config: new_config,
            effective_api_ids: effective,
        })
    }

    pub async fn run_turn_stream(
        &self,
        session_id: &str,
        player_input: String,
        api_overrides: Option<AgentApiIdOverrides>,
    ) -> Result<ManagedTurnStream<'static>, ManagerError> {
        let session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let story = self
            .store
            .get_story(&session.story_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStory(session.story_id.clone()))?;
        let runtime_state = self
            .build_runtime_state_from_session(&story, &session)
            .await?;
        let global = self.get_global_config().await?;
        let effective_api_ids = effective_session_api_ids(&session.config, &global)
            .apply_overrides(&api_overrides.unwrap_or_default());
        validate_api_ids(&self.registry, &effective_api_ids)?;
        let runtime_configs = self.registry.build_runtime_configs(&effective_api_ids)?;
        let mut engine = Engine::new(runtime_configs, runtime_state)?;
        let store = Arc::clone(&self.store);
        let session_record = session.clone();

        let stream = stream! {
            let mut engine_stream = match engine.run_turn_stream(&player_input).await {
                Ok(stream) => stream,
                Err(error) => {
                    yield Err(ManagerError::Engine(error));
                    return;
                }
            };

            while let Some(event) = engine_stream.next().await {
                match &event {
                    EngineEvent::TurnCompleted { result } => {
                        let mut updated_session = session_record.clone();
                        updated_session.snapshot = result.snapshot.clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                        if let Err(error) = store.save_session(updated_session).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                    }
                    EngineEvent::TurnFailed { snapshot, .. } => {
                        let mut updated_session = session_record.clone();
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                        if let Err(error) = store.save_session(updated_session).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                    }
                    _ => {}
                }

                yield Ok(event);
            }
        };

        Ok(Box::pin(stream))
    }

    async fn build_engine_story_resources(
        &self,
        resource: &StoryResourcesRecord,
    ) -> Result<StoryResources, ManagerError> {
        if resource.character_ids.is_empty() {
            return Err(ManagerError::EmptyCharacterIds);
        }

        let mut cards = Vec::with_capacity(resource.character_ids.len());
        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| ManagerError::MissingCharacter(character_id.clone()))?;
            cards.push(self.resolve_character_card(&character).await?);
        }

        let player_state_schema_seed = match &resource.player_schema_id_seed {
            Some(schema_id) => Some(self.resolve_player_schema(schema_id).await?),
            None => None,
        };

        let mut story_resources = StoryResources::new(
            resource.resource_id.clone(),
            resource.story_concept.clone(),
            cards,
            player_state_schema_seed,
        )?;

        if let Some(planned_story) = &resource.planned_story {
            story_resources = story_resources.with_planned_story(planned_story.clone());
        }
        if let Some(world_schema_id_seed) = &resource.world_schema_id_seed {
            story_resources = story_resources.with_world_state_schema_seed(
                self.resolve_world_schema(world_schema_id_seed).await?,
            );
        }

        Ok(story_resources)
    }

    async fn build_runtime_state_from_story(
        &self,
        story: &StoryRecord,
        player_description: String,
    ) -> Result<RuntimeState, ManagerError> {
        let characters = self.load_story_characters(&story.resource_id).await?;
        let mut resolved_characters = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved_characters.push(self.resolve_character_card(character).await?);
        }

        RuntimeState::from_story_graph(
            &story.story_id,
            story.graph.clone(),
            resolved_characters,
            player_description,
            self.resolve_player_schema(&story.player_schema_id).await?,
        )
        .map_err(ManagerError::from)
    }

    async fn build_runtime_state_from_session(
        &self,
        story: &StoryRecord,
        session: &SessionRecord,
    ) -> Result<RuntimeState, ManagerError> {
        let characters = self.load_story_characters(&story.resource_id).await?;
        let mut resolved_characters = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved_characters.push(self.resolve_character_card(character).await?);
        }

        RuntimeState::from_snapshot(
            &story.story_id,
            story::runtime_graph::RuntimeStoryGraph::from_story_graph(story.graph.clone())
                .map_err(RuntimeError::GraphBuild)?,
            resolved_characters,
            self.resolve_player_schema(&session.player_schema_id)
                .await?,
            session.snapshot.clone(),
        )
        .map_err(ManagerError::from)
    }

    async fn load_story_characters(
        &self,
        resource_id: &str,
    ) -> Result<Vec<CharacterCardRecord>, ManagerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;
        let mut characters = Vec::with_capacity(resource.character_ids.len());
        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| ManagerError::MissingCharacter(character_id.clone()))?;
            characters.push(character);
        }
        Ok(characters)
    }

    async fn resolve_schema_record(&self, schema_id: &str) -> Result<SchemaRecord, ManagerError> {
        self.store
            .get_schema(schema_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSchema(schema_id.to_owned()))
    }

    async fn resolve_world_schema(
        &self,
        schema_id: &str,
    ) -> Result<WorldStateSchema, ManagerError> {
        let schema = self.resolve_schema_record(schema_id).await?;
        Ok(WorldStateSchema {
            fields: schema.fields,
        })
    }

    async fn resolve_player_schema(
        &self,
        schema_id: &str,
    ) -> Result<PlayerStateSchema, ManagerError> {
        let schema = self.resolve_schema_record(schema_id).await?;
        Ok(PlayerStateSchema {
            fields: schema.fields,
        })
    }

    async fn resolve_character_card(
        &self,
        record: &CharacterCardRecord,
    ) -> Result<CharacterCard, ManagerError> {
        let schema = self
            .resolve_schema_record(&record.content.schema_id)
            .await?;
        Ok(CharacterCard {
            id: record.content.id.clone(),
            name: record.content.name.clone(),
            personality: record.content.personality.clone(),
            style: record.content.style.clone(),
            tendencies: record.content.tendencies.clone(),
            state_schema: schema.fields,
            system_prompt: record.content.system_prompt.clone(),
        })
    }
}

fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

fn validate_api_ids(registry: &LlmApiRegistry, api_ids: &AgentApiIds) -> Result<(), ManagerError> {
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

#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("global engine config is not initialized")]
    MissingGlobalConfig,
    #[error("schema '{0}' not found")]
    MissingSchema(String),
    #[error("character '{0}' not found")]
    MissingCharacter(String),
    #[error("player profile '{0}' not found")]
    MissingPlayerProfile(String),
    #[error("story resources '{0}' not found")]
    MissingStoryResources(String),
    #[error("story '{0}' not found")]
    MissingStory(String),
    #[error("session '{0}' not found")]
    MissingSession(String),
    #[error("character_ids cannot be empty")]
    EmptyCharacterIds,
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Store(#[from] StoreError),
}
