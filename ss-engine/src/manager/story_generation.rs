use std::collections::HashMap;
use std::sync::Arc;

use agents::architect::{Architect, ArchitectDraftContinueRequest, ArchitectDraftInitRequest};
use state::StateFieldSchema;
use store::{SchemaRecord, StoryDraftRecord, StoryDraftStatus, StoryRecord};
use story::{CommonVariableDefinition, NarrativeNode, StoryGraph, validate_common_variables};
use tracing::{debug, info};

use crate::logging::{
    json_for_log, summarize_architect_draft_chunk, summarize_architect_draft_init,
};
use crate::lorebook::{LorebookPromptSections, build_lorebook_prompt_sections};

use super::util::{
    apply_transition_patches, build_graph_summary, effective_planned_story_text,
    extract_outline_sections, merge_story_chunk, now_timestamp_ms, validate_schema_fields,
    validate_story_graph,
};
use super::{
    DEFAULT_ARCHITECT_CHUNK_NODE_COUNT, DEFAULT_ARCHITECT_CONTINUE_MAX_TOKENS,
    DEFAULT_ARCHITECT_INIT_MAX_TOKENS, DEFAULT_ARCHITECT_TEMPERATURE, EngineManager, ManagerError,
};

impl EngineManager {
    pub async fn generate_story_plan(
        &self,
        resource_id: &str,
        api_group_id: Option<String>,
        preset_id: Option<String>,
    ) -> Result<agents::planner::PlannerResponse, ManagerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;
        let story_resources = self.build_engine_story_resources(&resource).await?;
        let (api_group, preset, _) = self
            .resolve_api_group_and_preset(api_group_id.as_deref(), preset_id.as_deref())
            .await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let generation_configs = self.registry.build_story_generation_configs(
            &apis.planner,
            &apis.architect,
            &preset.agents.planner,
            &preset.agents.architect,
        )?;
        crate::generate_story_plan(&generation_configs, &story_resources)
            .await
            .map_err(ManagerError::from)
    }

    pub async fn generate_story(
        &self,
        resource_id: &str,
        display_name: Option<String>,
        api_group_id: Option<String>,
        preset_id: Option<String>,
        common_variables: Vec<CommonVariableDefinition>,
    ) -> Result<StoryRecord, ManagerError> {
        let mut draft = self
            .start_story_draft(
                resource_id,
                display_name,
                api_group_id,
                preset_id,
                common_variables,
            )
            .await?;

        while draft.status == StoryDraftStatus::Building {
            draft = self.continue_story_draft(&draft.draft_id).await?;
        }

        let story = self.finalize_story_draft(&draft.draft_id).await?;
        let _ = self.delete_story_draft(&draft.draft_id).await?;
        Ok(story)
    }

    pub async fn start_story_draft(
        &self,
        resource_id: &str,
        display_name: Option<String>,
        api_group_id: Option<String>,
        preset_id: Option<String>,
        common_variables: Vec<CommonVariableDefinition>,
    ) -> Result<StoryDraftRecord, ManagerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;

        let planned_story = effective_planned_story_text(&resource);
        let outline_sections = extract_outline_sections(&planned_story);
        if outline_sections.is_empty() {
            return Err(ManagerError::InvalidDraft(
                "planned_story did not contain any outline sections".to_owned(),
            ));
        }

        let story_resources = self.build_engine_story_resources(&resource).await?;
        let (api_group, preset, binding) = self
            .resolve_api_group_and_preset(api_group_id.as_deref(), preset_id.as_deref())
            .await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let generation_configs = self.registry.build_story_generation_configs(
            &apis.planner,
            &apis.architect,
            &preset.agents.planner,
            &preset.agents.architect,
        )?;
        let architect = self.build_architect_for_init(&generation_configs);
        let lorebook_sections = self.story_generation_lorebook_sections(
            &story_resources,
            &[
                story_resources.story_concept(),
                &planned_story,
                &outline_sections[0],
            ],
        );

        let init = architect
            .start_draft(ArchitectDraftInitRequest {
                story_concept: story_resources.story_concept(),
                planned_story: &planned_story,
                current_section: &outline_sections[0],
                section_index: 0,
                total_sections: outline_sections.len(),
                graph_summary: &[],
                recent_nodes: &[],
                target_node_count: DEFAULT_ARCHITECT_CHUNK_NODE_COUNT,
                world_state_schema: story_resources.world_state_schema_seed(),
                player_state_schema: story_resources.player_state_schema_seed(),
                available_characters: story_resources.character_cards(),
                lorebook_base: lorebook_sections.base.as_deref(),
                lorebook_matched: lorebook_sections.matched.as_deref(),
            })
            .await?;

        info!(
            resource_id = %resource.resource_id,
            summary = %json_for_log(&summarize_architect_draft_init(
                &init,
                0,
                outline_sections.len(),
            )),
            "architect generated draft init chunk"
        );
        debug!(
            resource_id = %resource.resource_id,
            payload = %json_for_log(&init),
            "architect draft init payload"
        );

        let now = now_timestamp_ms();
        let draft_id = format!("draft-{}", self.store.list_story_drafts().await?.len());
        let story_id_tag = format!("draft:{draft_id}");
        self.validate_story_common_variables_for_fields(
            &resource,
            &init.world_state_schema.fields,
            &init.player_state_schema.fields,
            &common_variables,
        )
        .await?;
        let world_schema = self
            .create_generated_schema(
                format!(
                    "{} world schema",
                    display_name
                        .as_deref()
                        .unwrap_or(resource.story_concept.as_str())
                ),
                vec![
                    "world".to_owned(),
                    "generated".to_owned(),
                    story_id_tag.clone(),
                ],
                init.world_state_schema.fields.clone(),
            )
            .await?;
        let player_schema = self
            .create_generated_schema(
                format!(
                    "{} player schema",
                    display_name
                        .as_deref()
                        .unwrap_or(resource.story_concept.as_str())
                ),
                vec!["player".to_owned(), "generated".to_owned(), story_id_tag],
                init.player_state_schema.fields.clone(),
            )
            .await?;

        let mut partial_graph = StoryGraph::new(init.start_node.clone(), init.nodes.clone());
        apply_transition_patches(&mut partial_graph, &init.transition_patches)?;
        validate_story_graph(&partial_graph)?;

        let draft = StoryDraftRecord {
            draft_id,
            display_name: display_name.unwrap_or_else(|| resource.story_concept.clone()),
            resource_id: resource.resource_id,
            api_group_id: binding.api_group_id,
            preset_id: binding.preset_id,
            planned_story,
            outline_sections,
            next_section_index: 1,
            partial_graph,
            world_schema_id: world_schema.schema_id,
            player_schema_id: player_schema.schema_id,
            introduction: init.introduction,
            common_variables,
            section_summaries: vec![init.section_summary],
            section_node_ids: vec![init.nodes.into_iter().map(|node| node.id).collect()],
            status: StoryDraftStatus::Building,
            final_story_id: None,
            created_at_ms: Some(now),
            updated_at_ms: Some(now),
        };

        let draft = self.refresh_draft_status(draft);
        self.store.save_story_draft(draft.clone()).await?;
        Ok(draft)
    }

    pub async fn continue_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<StoryDraftRecord, ManagerError> {
        let mut draft = self
            .store
            .get_story_draft(draft_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryDraft(draft_id.to_owned()))?;

        if draft.status != StoryDraftStatus::Building {
            return Err(ManagerError::InvalidDraft(format!(
                "story draft '{draft_id}' is not in building state"
            )));
        }

        let resource = self
            .store
            .get_story_resources(&draft.resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(draft.resource_id.clone()))?;
        let story_resources = self.build_engine_story_resources(&resource).await?;
        let api_group = self.resolve_api_group(&draft.api_group_id).await?;
        let preset = self.resolve_preset(&draft.preset_id).await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let generation_configs = self.registry.build_story_generation_configs(
            &apis.planner,
            &apis.architect,
            &preset.agents.planner,
            &preset.agents.architect,
        )?;
        let architect = self.build_architect_for_continue(&generation_configs);
        let world_schema = self.resolve_world_schema(&draft.world_schema_id).await?;
        let player_schema = self.resolve_player_schema(&draft.player_schema_id).await?;
        let graph_summary = build_graph_summary(&draft.partial_graph);
        let recent_nodes = self.recent_draft_nodes(&draft);
        let current_section = draft
            .outline_sections
            .get(draft.next_section_index)
            .ok_or_else(|| {
                ManagerError::InvalidDraft("story draft has no remaining section".to_owned())
            })?
            .clone();
        let section_summary_refs = draft
            .section_summaries
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let recent_node_texts = recent_nodes
            .iter()
            .flat_map(|node| [node.title.as_str(), node.scene.as_str(), node.goal.as_str()])
            .collect::<Vec<_>>();
        let mut lorebook_match_inputs =
            Vec::with_capacity(1 + section_summary_refs.len() + recent_node_texts.len());
        lorebook_match_inputs.push(current_section.as_str());
        lorebook_match_inputs.extend(section_summary_refs.iter().copied());
        lorebook_match_inputs.extend(recent_node_texts.iter().copied());
        let lorebook_sections =
            self.story_generation_lorebook_sections(&story_resources, &lorebook_match_inputs);

        let chunk = architect
            .continue_draft(ArchitectDraftContinueRequest {
                story_concept: story_resources.story_concept(),
                current_section: &current_section,
                section_index: draft.next_section_index,
                total_sections: draft.outline_sections.len(),
                section_summaries: &draft.section_summaries,
                graph_summary: &graph_summary,
                recent_nodes: &recent_nodes,
                target_node_count: DEFAULT_ARCHITECT_CHUNK_NODE_COUNT,
                world_state_schema: &world_schema,
                player_state_schema: &player_schema,
                available_characters: story_resources.character_cards(),
                lorebook_base: lorebook_sections.base.as_deref(),
                lorebook_matched: lorebook_sections.matched.as_deref(),
            })
            .await?;

        info!(
            draft_id = %draft.draft_id,
            resource_id = %draft.resource_id,
            summary = %json_for_log(&summarize_architect_draft_chunk(
                &chunk,
                draft.next_section_index,
                draft.outline_sections.len(),
            )),
            "architect generated draft continuation chunk"
        );
        debug!(
            draft_id = %draft.draft_id,
            resource_id = %draft.resource_id,
            payload = %json_for_log(&chunk),
            "architect draft continuation payload"
        );

        merge_story_chunk(
            &mut draft.partial_graph,
            &chunk.nodes,
            &chunk.transition_patches,
        )?;
        draft.section_summaries.push(chunk.section_summary);
        draft
            .section_node_ids
            .push(chunk.nodes.iter().map(|node| node.id.clone()).collect());
        draft.next_section_index += 1;
        draft.updated_at_ms = Some(now_timestamp_ms());
        draft = self.refresh_draft_status(draft);
        self.store.save_story_draft(draft.clone()).await?;
        Ok(draft)
    }

    pub async fn update_story_draft_graph(
        &self,
        draft_id: &str,
        partial_graph: StoryGraph,
    ) -> Result<StoryDraftRecord, ManagerError> {
        let mut draft = self
            .store
            .get_story_draft(draft_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryDraft(draft_id.to_owned()))?;

        if draft.status == StoryDraftStatus::Finalized {
            return Err(ManagerError::InvalidDraft(format!(
                "story draft '{draft_id}' is already finalized"
            )));
        }

        validate_story_graph(&partial_graph)?;
        draft.partial_graph = partial_graph;
        draft.updated_at_ms = Some(now_timestamp_ms());
        draft = self.refresh_draft_status(draft);
        self.store.save_story_draft(draft.clone()).await?;
        Ok(draft)
    }

    pub async fn finalize_story_draft(&self, draft_id: &str) -> Result<StoryRecord, ManagerError> {
        let mut draft = self
            .store
            .get_story_draft(draft_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryDraft(draft_id.to_owned()))?;

        if draft.status == StoryDraftStatus::Finalized {
            let story_id = draft.final_story_id.clone().ok_or_else(|| {
                ManagerError::InvalidDraft("finalized draft is missing final_story_id".to_owned())
            })?;
            return self
                .store
                .get_story(&story_id)
                .await?
                .ok_or(ManagerError::MissingStory(story_id));
        }

        draft = self.refresh_draft_status(draft);
        if draft.status != StoryDraftStatus::ReadyToFinalize {
            return Err(ManagerError::InvalidDraft(format!(
                "story draft '{draft_id}' is not ready to finalize"
            )));
        }

        validate_story_graph(&draft.partial_graph)?;
        let resource = self
            .store
            .get_story_resources(&draft.resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(draft.resource_id.clone()))?;
        let now = now_timestamp_ms();
        let story = StoryRecord {
            story_id: format!("story-{}", self.store.list_stories().await?.len()),
            display_name: draft.display_name.clone(),
            resource_id: draft.resource_id.clone(),
            graph: draft.partial_graph.clone(),
            world_schema_id: draft.world_schema_id.clone(),
            player_schema_id: draft.player_schema_id.clone(),
            introduction: draft.introduction.clone(),
            common_variables: draft.common_variables.clone(),
            created_at_ms: Some(now),
            updated_at_ms: Some(now),
        };
        self.store.save_story(story.clone()).await?;

        draft.status = StoryDraftStatus::Finalized;
        draft.final_story_id = Some(story.story_id.clone());
        draft.updated_at_ms = Some(now);
        if resource.resource_id == draft.resource_id {
            self.store.save_story_draft(draft).await?;
        }
        Ok(story)
    }

    pub async fn delete_story_draft(
        &self,
        draft_id: &str,
    ) -> Result<StoryDraftRecord, ManagerError> {
        let draft = self
            .store
            .delete_story_draft(draft_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryDraft(draft_id.to_owned()))?;

        if draft.final_story_id.is_none() {
            let _ = self.store.delete_schema(&draft.world_schema_id).await?;
            let _ = self.store.delete_schema(&draft.player_schema_id).await?;
        }

        Ok(draft)
    }

    pub(super) fn story_generation_lorebook_sections<'a>(
        &self,
        resources: &'a crate::StoryResources,
        extra_inputs: &[&'a str],
    ) -> LorebookPromptSections {
        let mut match_inputs = Vec::with_capacity(extra_inputs.len().saturating_add(2));
        match_inputs.push(resources.story_concept());
        if let Some(planned_story) = resources.planned_story() {
            match_inputs.push(planned_story);
        }
        match_inputs.extend(
            extra_inputs
                .iter()
                .copied()
                .filter(|text| !text.trim().is_empty()),
        );

        build_lorebook_prompt_sections(resources.lorebook_entries(), &match_inputs)
    }

    fn build_architect_for_init(
        &self,
        generation_configs: &crate::engine::StoryGenerationAgentConfigs,
    ) -> Architect {
        Architect::new_with_options(
            Arc::clone(&generation_configs.architect.client),
            generation_configs.architect.model.clone(),
            Some(
                generation_configs
                    .architect
                    .temperature
                    .unwrap_or(DEFAULT_ARCHITECT_TEMPERATURE),
            ),
            Some(
                generation_configs
                    .architect
                    .max_tokens
                    .unwrap_or(DEFAULT_ARCHITECT_INIT_MAX_TOKENS),
            ),
        )
        .with_prompt_profiles(generation_configs.architect.prompt_profiles.clone())
    }

    fn build_architect_for_continue(
        &self,
        generation_configs: &crate::engine::StoryGenerationAgentConfigs,
    ) -> Architect {
        Architect::new_with_options(
            Arc::clone(&generation_configs.architect.client),
            generation_configs.architect.model.clone(),
            Some(
                generation_configs
                    .architect
                    .temperature
                    .unwrap_or(DEFAULT_ARCHITECT_TEMPERATURE),
            ),
            Some(
                generation_configs
                    .architect
                    .max_tokens
                    .unwrap_or(DEFAULT_ARCHITECT_CONTINUE_MAX_TOKENS),
            ),
        )
        .with_prompt_profiles(generation_configs.architect.prompt_profiles.clone())
    }

    async fn create_generated_schema(
        &self,
        display_name: String,
        tags: Vec<String>,
        fields: HashMap<String, StateFieldSchema>,
    ) -> Result<SchemaRecord, ManagerError> {
        validate_schema_fields(&fields)?;
        let mut next_index = self.store.list_schemas().await?.len();
        loop {
            let schema_id = format!("schema-generated-{next_index}");
            if self.store.get_schema(&schema_id).await?.is_none() {
                let record = SchemaRecord {
                    schema_id,
                    display_name: display_name.clone(),
                    tags: tags.clone(),
                    fields: fields.clone(),
                };
                self.store.save_schema(record.clone()).await?;
                return Ok(record);
            }
            next_index += 1;
        }
    }

    pub(super) fn recent_draft_nodes(&self, draft: &StoryDraftRecord) -> Vec<NarrativeNode> {
        draft
            .section_node_ids
            .last()
            .into_iter()
            .flatten()
            .filter_map(|node_id| draft.partial_graph.get_node(node_id).cloned())
            .collect()
    }

    fn refresh_draft_status(&self, mut draft: StoryDraftRecord) -> StoryDraftRecord {
        draft.status = if draft.final_story_id.is_some() {
            StoryDraftStatus::Finalized
        } else if draft.next_section_index >= draft.outline_sections.len() {
            StoryDraftStatus::ReadyToFinalize
        } else {
            StoryDraftStatus::Building
        };
        draft
    }

    async fn validate_story_common_variables_for_fields(
        &self,
        resource: &store::StoryResourcesRecord,
        world_fields: &HashMap<String, StateFieldSchema>,
        player_fields: &HashMap<String, StateFieldSchema>,
        common_variables: &[CommonVariableDefinition],
    ) -> Result<(), ManagerError> {
        let mut character_fields = HashMap::new();

        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| ManagerError::MissingCharacter(character_id.clone()))?;
            let schema = self
                .store
                .get_schema(&character.content.schema_id)
                .await?
                .ok_or_else(|| ManagerError::MissingSchema(character.content.schema_id.clone()))?;
            character_fields.insert(character_id.clone(), schema.fields);
        }

        validate_common_variables(
            common_variables,
            &resource.character_ids,
            world_fields,
            player_fields,
            &character_fields,
        )
        .map_err(ManagerError::InvalidCommonVariable)
    }
}
