use agents::actor::CharacterCard;
use engine::{
    AgentApiIdOverrides, RuntimeState, SessionConfigMode, SessionEngineConfig, StoryResources,
    generate_story_graph, generate_story_plan,
};
use protocol::{
    CreateStoryResourcesParams, GenerateStoryParams, GenerateStoryPlanParams,
    JsonRpcResponseMessage, ResponseResult, SessionStartedPayload, StartSessionFromStoryParams,
    StoryGeneratedPayload, StoryPlannedPayload, StoryResourcesPayload, UpdateStoryResourcesParams,
};

use crate::error::HandlerError;
use crate::store::{CharacterCardRecord, SessionRecord, StoryRecord};

use super::Handler;
use super::config::{build_session_config_payload, effective_session_api_ids, validate_api_ids};

impl<'a> Handler<'a> {
    pub(crate) async fn handle_story_resources_create(
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

    pub(crate) async fn handle_story_resources_update(
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

    pub(crate) async fn handle_story_generate_plan(
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

    pub(crate) async fn handle_story_generate(
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

    pub(crate) async fn handle_story_start_session(
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

    async fn ensure_characters_exist(&self, character_ids: &[String]) -> Result<(), HandlerError> {
        for character_id in character_ids {
            if self.store.get_character(character_id).await?.is_none() {
                return Err(HandlerError::MissingCharacter(character_id.clone()));
            }
        }
        Ok(())
    }

    pub(crate) async fn load_story_character_cards(
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
}
