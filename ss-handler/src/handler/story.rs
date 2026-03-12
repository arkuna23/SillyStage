use protocol::{
    CharacterCardSummaryPayload, CreateStoryResourcesParams, DeleteStoryParams,
    DeleteStoryResourcesParams, GenerateStoryParams, GenerateStoryPlanParams, GetStoryParams,
    GetStoryResourcesParams, JsonRpcResponseMessage, ResponseResult, SessionStartedPayload,
    StartSessionFromStoryParams, StoriesListedPayload, StoryDeletedPayload, StoryDetailPayload,
    StoryGeneratedPayload, StoryPlannedPayload, StoryResourcesDeletedPayload,
    StoryResourcesListedPayload, StoryResourcesPayload, StorySummaryPayload,
    UpdateStoryResourcesParams,
};
use store::{CharacterCardRecord, StoryRecord, StoryResourcesRecord};

use crate::error::HandlerError;

use super::Handler;
use super::config::build_session_config_payload;

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

        let record = StoryResourcesRecord {
            resource_id: self.id_generator.next("resource"),
            story_concept: params.story_concept,
            character_ids: params.character_ids,
            player_state_schema_seed: params.player_state_schema_seed,
            world_state_schema_seed: params.world_state_schema_seed,
            planned_story: params.planned_story,
        };

        self.store.save_story_resources(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResourcesCreated(Box::new(story_resources_payload_from_record(
                &record,
            ))),
        ))
    }

    pub(crate) async fn handle_story_resources_get(
        &self,
        request_id: &str,
        params: GetStoryResourcesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let record = self
            .store
            .get_story_resources(&params.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(params.resource_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResources(Box::new(story_resources_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_story_resources_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let resources = self
            .store
            .list_story_resources()
            .await?
            .into_iter()
            .map(|record| story_resources_payload_from_record(&record))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResourcesListed(StoryResourcesListedPayload { resources }),
        ))
    }

    pub(crate) async fn handle_story_resources_update(
        &self,
        request_id: &str,
        params: UpdateStoryResourcesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut record = self
            .store
            .get_story_resources(&params.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(params.resource_id.clone()))?;

        if let Some(story_concept) = params.story_concept {
            record.story_concept = story_concept;
        }
        if let Some(character_ids) = params.character_ids {
            if character_ids.is_empty() {
                return Err(HandlerError::EmptyCharacterIds);
            }
            self.ensure_characters_exist(&character_ids).await?;
            record.character_ids = character_ids;
        }
        if let Some(player_state_schema_seed) = params.player_state_schema_seed {
            record.player_state_schema_seed = player_state_schema_seed;
        }
        if let Some(world_state_schema_seed) = params.world_state_schema_seed {
            record.world_state_schema_seed = Some(world_state_schema_seed);
        }
        if let Some(planned_story) = params.planned_story {
            record.planned_story = Some(planned_story);
        }

        self.store.save_story_resources(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResourcesUpdated(Box::new(story_resources_payload_from_record(
                &record,
            ))),
        ))
    }

    pub(crate) async fn handle_story_resources_delete(
        &self,
        request_id: &str,
        params: DeleteStoryResourcesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if self
            .store
            .list_stories()
            .await?
            .into_iter()
            .any(|story| story.resource_id == params.resource_id)
        {
            return Err(HandlerError::StoryResourcesInUse(params.resource_id));
        }

        self.store
            .delete_story_resources(&params.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(params.resource_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryResourcesDeleted(StoryResourcesDeletedPayload {
                resource_id: params.resource_id,
            }),
        ))
    }

    pub(crate) async fn handle_story_generate_plan(
        &self,
        request_id: &str,
        params: GenerateStoryPlanParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let response = self
            .manager
            .generate_story_plan(&params.resource_id, params.planner_api_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryPlanned(StoryPlannedPayload {
                resource_id: params.resource_id,
                story_script: response.story_script,
            }),
        ))
    }

    pub(crate) async fn handle_story_generate(
        &self,
        request_id: &str,
        params: GenerateStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let story = self
            .manager
            .generate_story(
                &params.resource_id,
                params.display_name,
                params.architect_api_id,
            )
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryGenerated(Box::new(story_generated_payload_from_record(&story))),
        ))
    }

    pub(crate) async fn handle_story_get(
        &self,
        request_id: &str,
        params: GetStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let story = self
            .store
            .get_story(&params.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(params.story_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Story(Box::new(story_detail_payload_from_record(&story))),
        ))
    }

    pub(crate) async fn handle_story_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let stories = self
            .store
            .list_stories()
            .await?
            .into_iter()
            .map(|story| story_summary_payload_from_record(&story))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoriesListed(StoriesListedPayload { stories }),
        ))
    }

    pub(crate) async fn handle_story_delete(
        &self,
        request_id: &str,
        params: DeleteStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if self
            .store
            .list_sessions()
            .await?
            .into_iter()
            .any(|session| session.story_id == params.story_id)
        {
            return Err(HandlerError::StoryHasSessions(params.story_id));
        }

        self.store
            .delete_story(&params.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(params.story_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDeleted(StoryDeletedPayload {
                story_id: params.story_id,
            }),
        ))
    }

    pub(crate) async fn handle_story_start_session(
        &self,
        request_id: &str,
        params: StartSessionFromStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session = self
            .manager
            .start_session_from_story(
                &params.story_id,
                params.display_name,
                params.player_description,
                params.config_mode,
                params.session_api_ids,
            )
            .await?;
        let story = self
            .store
            .get_story(&session.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(session.story_id.clone()))?;
        let character_summaries = self
            .load_story_character_cards(&story.resource_id)
            .await?
            .into_iter()
            .map(|record| character_summary_payload_from_record(&record))
            .collect();
        let config = build_session_config_payload(
            self.manager
                .get_resolved_session_config(&session.session_id)
                .await?,
        );

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session.session_id),
            ResponseResult::SessionStarted(Box::new(SessionStartedPayload {
                story_id: story.story_id,
                display_name: session.display_name,
                snapshot: session.snapshot,
                character_summaries,
                config,
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
}

fn character_summary_payload_from_record(
    record: &CharacterCardRecord,
) -> CharacterCardSummaryPayload {
    CharacterCardSummaryPayload {
        character_id: record.character_id.clone(),
        name: record.content.name.clone(),
        personality: record.content.personality.clone(),
        style: record.content.style.clone(),
        tendencies: record.content.tendencies.clone(),
        cover_file_name: record.cover_file_name.clone(),
        cover_mime_type: serde_json::from_str(&format!("\"{}\"", record.cover_mime_type))
            .expect("stored cover mime type should deserialize"),
    }
}

fn story_resources_payload_from_record(record: &StoryResourcesRecord) -> StoryResourcesPayload {
    StoryResourcesPayload {
        resource_id: record.resource_id.clone(),
        story_concept: record.story_concept.clone(),
        character_ids: record.character_ids.clone(),
        player_state_schema_seed: record.player_state_schema_seed.clone(),
        world_state_schema_seed: record.world_state_schema_seed.clone(),
        planned_story: record.planned_story.clone(),
    }
}

fn story_generated_payload_from_record(record: &StoryRecord) -> StoryGeneratedPayload {
    StoryGeneratedPayload {
        resource_id: record.resource_id.clone(),
        story_id: record.story_id.clone(),
        display_name: record.display_name.clone(),
        graph: record.graph.clone(),
        world_state_schema: record.world_state_schema.clone(),
        player_state_schema: record.player_state_schema.clone(),
        introduction: record.introduction.clone(),
    }
}

fn story_summary_payload_from_record(record: &StoryRecord) -> StorySummaryPayload {
    StorySummaryPayload {
        story_id: record.story_id.clone(),
        display_name: record.display_name.clone(),
        resource_id: record.resource_id.clone(),
        introduction: record.introduction.clone(),
    }
}

fn story_detail_payload_from_record(record: &StoryRecord) -> StoryDetailPayload {
    StoryDetailPayload {
        story_id: record.story_id.clone(),
        display_name: record.display_name.clone(),
        resource_id: record.resource_id.clone(),
        graph: record.graph.clone(),
        world_state_schema: record.world_state_schema.clone(),
        player_state_schema: record.player_state_schema.clone(),
        introduction: record.introduction.clone(),
    }
}
