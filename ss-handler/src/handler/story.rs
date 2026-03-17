use protocol::{
    CharacterCardSummaryPayload, ContinueStoryDraftParams, CreateStoryResourcesParams,
    DeleteStoryDraftParams, DeleteStoryParams, DeleteStoryResourcesParams,
    FinalizeStoryDraftParams, GenerateStoryParams, GenerateStoryPlanParams, GetStoryDraftParams,
    GetStoryParams, GetStoryResourcesParams, JsonRpcResponseMessage, ResponseResult,
    StartSessionFromStoryParams, StartStoryDraftParams, StoriesListedPayload, StoryDeletedPayload,
    StoryDetailPayload, StoryDraftDeletedPayload, StoryDraftDetailPayload, StoryDraftStatusPayload,
    StoryDraftSummaryPayload, StoryDraftsListedPayload, StoryGeneratedPayload, StoryPlannedPayload,
    StoryResourcesDeletedPayload, StoryResourcesListedPayload, StoryResourcesPayload,
    StorySummaryPayload, UpdateStoryDraftGraphParams, UpdateStoryGraphParams, UpdateStoryParams,
    UpdateStoryResourcesParams,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use store::{
    CharacterCardRecord, StoryDraftRecord, StoryDraftStatus, StoryRecord, StoryResourcesRecord,
};
use story::runtime_graph::{GraphBuildError, RuntimeStoryGraph};
use story::{
    CommonVariableDefinition, validate_common_variables, validate_graph_state_conventions,
};

use crate::error::HandlerError;

use super::Handler;
use super::config::build_session_config_payload;
use super::session::{build_session_started_payload, load_session_message_payloads};

impl Handler {
    pub(crate) async fn handle_story_resources_create(
        &self,
        request_id: &str,
        params: CreateStoryResourcesParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        if params.character_ids.is_empty() {
            return Err(HandlerError::EmptyCharacterIds);
        }

        self.ensure_characters_exist(&params.character_ids).await?;
        if let Some(player_schema_id_seed) = &params.player_schema_id_seed {
            self.ensure_schema_exists(player_schema_id_seed).await?;
        }
        if let Some(world_schema_id_seed) = &params.world_schema_id_seed {
            self.ensure_schema_exists(world_schema_id_seed).await?;
        }
        self.ensure_lorebooks_exist(&params.lorebook_ids).await?;

        let record = StoryResourcesRecord {
            resource_id: self.id_generator.next("resource"),
            story_concept: params.story_concept,
            character_ids: params.character_ids,
            player_schema_id_seed: params.player_schema_id_seed,
            world_schema_id_seed: params.world_schema_id_seed,
            lorebook_ids: params.lorebook_ids,
            planned_story: normalize_planned_story(params.planned_story),
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
        if let Some(player_schema_id_seed) = params.player_schema_id_seed {
            self.ensure_schema_exists(&player_schema_id_seed).await?;
            record.player_schema_id_seed = Some(player_schema_id_seed);
        }
        if let Some(world_schema_id_seed) = params.world_schema_id_seed {
            self.ensure_schema_exists(&world_schema_id_seed).await?;
            record.world_schema_id_seed = Some(world_schema_id_seed);
        }
        if let Some(lorebook_ids) = params.lorebook_ids {
            self.ensure_lorebooks_exist(&lorebook_ids).await?;
            record.lorebook_ids = lorebook_ids;
        }
        if let Some(planned_story) = params.planned_story {
            record.planned_story = normalize_planned_story(Some(planned_story));
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
        if self
            .store
            .list_story_drafts()
            .await?
            .into_iter()
            .any(|draft| draft.resource_id == params.resource_id)
        {
            return Err(HandlerError::StoryResourcesDraftInUse(params.resource_id));
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
            .generate_story_plan(&params.resource_id, params.api_group_id, params.preset_id)
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
                params.api_group_id,
                params.preset_id,
                params.common_variables.unwrap_or_default(),
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

    pub(crate) async fn handle_story_update(
        &self,
        request_id: &str,
        params: UpdateStoryParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut story = self
            .store
            .get_story(&params.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(params.story_id.clone()))?;

        if let Some(display_name) = params.display_name {
            story.display_name = display_name;
        }
        if let Some(common_variables) = params.common_variables {
            self.validate_story_common_variables(&story, &common_variables)
                .await?;
            story.common_variables = common_variables;
        }
        story.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_story(story.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Story(Box::new(story_detail_payload_from_record(&story))),
        ))
    }

    pub(crate) async fn handle_story_update_graph(
        &self,
        request_id: &str,
        params: UpdateStoryGraphParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut story = self
            .store
            .get_story(&params.story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(params.story_id.clone()))?;

        validate_story_graph(&params.graph)?;
        story.graph = params.graph;
        story.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_story(story.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Story(Box::new(story_detail_payload_from_record(&story))),
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
                params.player_profile_id,
                params.api_group_id,
                params.preset_id,
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
        let history =
            load_session_message_payloads(self.store.as_ref(), &session.session_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session.session_id.clone()),
            ResponseResult::SessionStarted(Box::new(build_session_started_payload(
                &session,
                history,
                character_summaries,
                config,
            ))),
        ))
    }

    pub(crate) async fn handle_story_draft_start(
        &self,
        request_id: &str,
        params: StartStoryDraftParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let draft = self
            .manager
            .start_story_draft(
                &params.resource_id,
                params.display_name,
                params.api_group_id,
                params.preset_id,
                params.common_variables.unwrap_or_default(),
            )
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDraft(Box::new(story_draft_detail_payload_from_record(&draft))),
        ))
    }

    pub(crate) async fn handle_story_draft_get(
        &self,
        request_id: &str,
        params: GetStoryDraftParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let draft = self
            .store
            .get_story_draft(&params.draft_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryDraft(params.draft_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDraft(Box::new(story_draft_detail_payload_from_record(&draft))),
        ))
    }

    pub(crate) async fn handle_story_draft_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let drafts = self
            .store
            .list_story_drafts()
            .await?
            .into_iter()
            .map(|draft| story_draft_summary_payload_from_record(&draft))
            .collect();

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDraftsListed(StoryDraftsListedPayload { drafts }),
        ))
    }

    pub(crate) async fn handle_story_draft_update_graph(
        &self,
        request_id: &str,
        params: UpdateStoryDraftGraphParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let draft = self
            .manager
            .update_story_draft_graph(&params.draft_id, params.partial_graph)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDraft(Box::new(story_draft_detail_payload_from_record(&draft))),
        ))
    }

    pub(crate) async fn handle_story_draft_continue(
        &self,
        request_id: &str,
        params: ContinueStoryDraftParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let draft = self.manager.continue_story_draft(&params.draft_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDraft(Box::new(story_draft_detail_payload_from_record(&draft))),
        ))
    }

    pub(crate) async fn handle_story_draft_finalize(
        &self,
        request_id: &str,
        params: FinalizeStoryDraftParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let story = self.manager.finalize_story_draft(&params.draft_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryGenerated(Box::new(story_generated_payload_from_record(&story))),
        ))
    }

    pub(crate) async fn handle_story_draft_delete(
        &self,
        request_id: &str,
        params: DeleteStoryDraftParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        self.manager.delete_story_draft(&params.draft_id).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::StoryDraftDeleted(StoryDraftDeletedPayload {
                draft_id: params.draft_id,
            }),
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

    pub(crate) async fn ensure_schema_exists(&self, schema_id: &str) -> Result<(), HandlerError> {
        if self.store.get_schema(schema_id).await?.is_none() {
            return Err(HandlerError::MissingSchema(schema_id.to_owned()));
        }
        Ok(())
    }

    pub(crate) async fn ensure_lorebooks_exist(
        &self,
        lorebook_ids: &[String],
    ) -> Result<(), HandlerError> {
        for lorebook_id in lorebook_ids {
            if self.store.get_lorebook(lorebook_id).await?.is_none() {
                return Err(HandlerError::MissingLorebook(lorebook_id.clone()));
            }
        }
        Ok(())
    }

    async fn validate_story_common_variables(
        &self,
        story: &StoryRecord,
        common_variables: &[CommonVariableDefinition],
    ) -> Result<(), HandlerError> {
        let resource = self
            .store
            .get_story_resources(&story.resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(story.resource_id.clone()))?;
        let world_schema = self
            .store
            .get_schema(&story.world_schema_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSchema(story.world_schema_id.clone()))?;
        let player_schema = self
            .store
            .get_schema(&story.player_schema_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSchema(story.player_schema_id.clone()))?;
        let mut character_fields = HashMap::new();

        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| HandlerError::MissingCharacter(character_id.clone()))?;
            let schema = self
                .store
                .get_schema(&character.content.schema_id)
                .await?
                .ok_or_else(|| HandlerError::MissingSchema(character.content.schema_id.clone()))?;
            character_fields.insert(character_id.clone(), schema.fields);
        }

        validate_common_variables(
            common_variables,
            &resource.character_ids,
            &world_schema.fields,
            &player_schema.fields,
            &character_fields,
        )
        .map_err(HandlerError::InvalidCommonVariable)
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
        cover_file_name: record.cover_file_name.clone(),
        cover_mime_type: record
            .cover_mime_type
            .as_deref()
            .and_then(protocol::CharacterCoverMimeType::parse),
    }
}

fn story_resources_payload_from_record(record: &StoryResourcesRecord) -> StoryResourcesPayload {
    StoryResourcesPayload {
        resource_id: record.resource_id.clone(),
        story_concept: record.story_concept.clone(),
        character_ids: record.character_ids.clone(),
        player_schema_id_seed: record.player_schema_id_seed.clone(),
        world_schema_id_seed: record.world_schema_id_seed.clone(),
        lorebook_ids: record.lorebook_ids.clone(),
        planned_story: record.planned_story.clone(),
    }
}

fn story_generated_payload_from_record(record: &StoryRecord) -> StoryGeneratedPayload {
    StoryGeneratedPayload {
        resource_id: record.resource_id.clone(),
        story_id: record.story_id.clone(),
        display_name: record.display_name.clone(),
        graph: record.graph.clone(),
        world_schema_id: record.world_schema_id.clone(),
        player_schema_id: record.player_schema_id.clone(),
        introduction: record.introduction.clone(),
        common_variables: record.common_variables.clone(),
    }
}

fn story_summary_payload_from_record(record: &StoryRecord) -> StorySummaryPayload {
    StorySummaryPayload {
        story_id: record.story_id.clone(),
        display_name: record.display_name.clone(),
        resource_id: record.resource_id.clone(),
        world_schema_id: record.world_schema_id.clone(),
        player_schema_id: record.player_schema_id.clone(),
        introduction: record.introduction.clone(),
        common_variables: record.common_variables.clone(),
    }
}

fn story_detail_payload_from_record(record: &StoryRecord) -> StoryDetailPayload {
    StoryDetailPayload {
        story_id: record.story_id.clone(),
        display_name: record.display_name.clone(),
        resource_id: record.resource_id.clone(),
        graph: record.graph.clone(),
        world_schema_id: record.world_schema_id.clone(),
        player_schema_id: record.player_schema_id.clone(),
        introduction: record.introduction.clone(),
        common_variables: record.common_variables.clone(),
    }
}

fn story_draft_status_payload(status: StoryDraftStatus) -> StoryDraftStatusPayload {
    match status {
        StoryDraftStatus::Building => StoryDraftStatusPayload::Building,
        StoryDraftStatus::ReadyToFinalize => StoryDraftStatusPayload::ReadyToFinalize,
        StoryDraftStatus::Finalized => StoryDraftStatusPayload::Finalized,
    }
}

fn story_draft_summary_payload_from_record(record: &StoryDraftRecord) -> StoryDraftSummaryPayload {
    StoryDraftSummaryPayload {
        draft_id: record.draft_id.clone(),
        display_name: record.display_name.clone(),
        resource_id: record.resource_id.clone(),
        api_group_id: record.api_group_id.clone(),
        preset_id: record.preset_id.clone(),
        status: story_draft_status_payload(record.status),
        next_section_index: record.next_section_index,
        total_sections: record.outline_sections.len(),
        partial_node_count: record.partial_graph.nodes.len(),
        final_story_id: record.final_story_id.clone(),
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
    }
}

fn story_draft_detail_payload_from_record(record: &StoryDraftRecord) -> StoryDraftDetailPayload {
    StoryDraftDetailPayload {
        draft_id: record.draft_id.clone(),
        display_name: record.display_name.clone(),
        resource_id: record.resource_id.clone(),
        api_group_id: record.api_group_id.clone(),
        preset_id: record.preset_id.clone(),
        planned_story: record.planned_story.clone(),
        outline_sections: record.outline_sections.clone(),
        next_section_index: record.next_section_index,
        partial_graph: record.partial_graph.clone(),
        world_schema_id: record.world_schema_id.clone(),
        player_schema_id: record.player_schema_id.clone(),
        introduction: record.introduction.clone(),
        common_variables: record.common_variables.clone(),
        section_summaries: record.section_summaries.clone(),
        status: story_draft_status_payload(record.status),
        final_story_id: record.final_story_id.clone(),
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
    }
}

fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis() as u64
}

fn normalize_planned_story(planned_story: Option<String>) -> Option<String> {
    planned_story.filter(|value| !value.trim().is_empty())
}

fn validate_story_graph(graph: &story::StoryGraph) -> Result<(), HandlerError> {
    RuntimeStoryGraph::from_story_graph(graph.clone()).map_err(|error| {
        HandlerError::InvalidStoryGraph(match error {
            GraphBuildError::MissingStartNode(node_id) => {
                format!("start node '{node_id}' does not exist")
            }
            GraphBuildError::MissingTargetNode { from, to } => {
                format!("transition from '{from}' points to missing node '{to}'")
            }
            GraphBuildError::DuplicateNodeId(node_id) => {
                format!("story graph contains duplicate node id '{node_id}'")
            }
        })
    })?;
    validate_graph_state_conventions(graph)
        .map_err(|error| HandlerError::InvalidStoryGraph(error.to_string()))?;
    Ok(())
}
