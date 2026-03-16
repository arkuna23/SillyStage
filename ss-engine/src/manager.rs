use std::collections::HashSet;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use agents::actor::ActorSegmentKind;
use agents::actor::CharacterCard;
use agents::architect::{
    Architect, ArchitectDraftContinueRequest, ArchitectDraftInitRequest, ArchitectError,
    GraphSummaryNode, NodeTransitionPatch,
};
use agents::replyer::{
    ReplyHistoryKind, ReplyHistoryMessage, ReplyOption, Replyer, ReplyerError, ReplyerRequest,
};
use async_stream::stream;
use futures_core::Stream;
use futures_util::StreamExt;
use state::{PlayerStateSchema, StateFieldSchema, WorldStateSchema};
use store::{
    ApiGroupRecord, ApiRecord, CharacterCardRecord, PresetRecord, RuntimeSnapshot, SchemaRecord,
    SessionBindingConfig, SessionCharacterRecord, SessionMessageKind, SessionMessageRecord,
    SessionRecord, Store, StoreError, StoryDraftRecord, StoryDraftStatus, StoryRecord,
    StoryResourcesRecord,
};
use story::{NarrativeNode, StoryGraph, validate_graph_state_conventions};
use tracing::{debug, info};

use crate::logging::{
    json_for_log, summarize_architect_draft_chunk, summarize_architect_draft_init,
    summarize_reply_options,
};
use crate::{
    Engine, EngineError, EngineEvent, EngineTurnResult, ExecutedBeat, LlmApiRegistry,
    RegistryError, RuntimeApiRecords, RuntimeError, RuntimeState, StoryResources,
    generate_story_plan,
};

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

    pub async fn get_global_config(&self) -> Result<Option<SessionBindingConfig>, ManagerError> {
        self.resolve_first_available_binding().await
    }

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
        generate_story_plan(&generation_configs, &story_resources)
            .await
            .map_err(ManagerError::from)
    }

    pub async fn generate_story(
        &self,
        resource_id: &str,
        display_name: Option<String>,
        api_group_id: Option<String>,
        preset_id: Option<String>,
    ) -> Result<StoryRecord, ManagerError> {
        let mut draft = self
            .start_story_draft(resource_id, display_name, api_group_id, preset_id)
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

    pub async fn start_session_from_story(
        &self,
        story_id: &str,
        display_name: Option<String>,
        player_profile_id: Option<String>,
        api_group_id: Option<String>,
        preset_id: Option<String>,
    ) -> Result<SessionRecord, ManagerError> {
        let story = self
            .store
            .get_story(story_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStory(story_id.to_owned()))?;
        let (_api_group, _preset, binding) = self
            .resolve_api_group_and_preset(api_group_id.as_deref(), preset_id.as_deref())
            .await?;

        let (player_name, player_description) = self
            .resolve_player_identity(player_profile_id.as_deref())
            .await?;
        let runtime_state = self
            .build_runtime_state_from_story(&story, player_name, player_description)
            .await?;
        let session_id = format!("session-{}", self.store.list_sessions().await?.len());
        let now = now_timestamp_ms();
        let session = SessionRecord {
            session_id,
            display_name: display_name.unwrap_or_else(|| story.display_name.clone()),
            story_id: story.story_id,
            player_profile_id,
            player_schema_id: story.player_schema_id,
            binding,
            snapshot: runtime_state.snapshot(),
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

    pub async fn suggest_replies(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ReplyOption>, ManagerError> {
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
        let api_group = self
            .resolve_api_group(&session.binding.api_group_id)
            .await?;
        let preset = self.resolve_preset(&session.binding.preset_id).await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let replyer_config = self
            .registry
            .build_replyer_config(&apis.replyer, &preset.agents.replyer)?;
        let history = self.load_reply_history(session_id).await?;
        let current_node = runtime_state.current_node()?;
        let replyer = Replyer::new_with_options(
            Arc::clone(&replyer_config.client),
            replyer_config.model.clone(),
            replyer_config.temperature,
            replyer_config.max_tokens,
        )?;
        let response = replyer
            .suggest(ReplyerRequest {
                current_node,
                character_cards: runtime_state.character_cards(),
                current_cast_ids: runtime_state.world_state().active_characters(),
                player_name: runtime_state.player_name(),
                player_description: runtime_state.player_description(),
                player_state_schema: runtime_state.player_state_schema(),
                world_state: runtime_state.world_state(),
                history: &history,
                limit,
            })
            .await?;

        info!(
            session_id = %session_id,
            summary = %json_for_log(&summarize_reply_options(&response.replies)),
            "replyer generated suggested replies"
        );
        debug!(
            session_id = %session_id,
            payload = %json_for_log(&response),
            "replyer response payload"
        );

        Ok(response.replies)
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

        let (_player_name, player_description) = self
            .resolve_player_identity(player_profile_id.as_deref())
            .await?;
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
        Ok(ResolvedSessionConfig {
            binding: session.binding,
        })
    }

    pub async fn update_session_config(
        &self,
        session_id: &str,
        api_group_id: Option<String>,
        preset_id: Option<String>,
    ) -> Result<ResolvedSessionConfig, ManagerError> {
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let binding = self
            .resolve_api_group_and_preset(
                api_group_id
                    .as_deref()
                    .or(Some(session.binding.api_group_id.as_str())),
                preset_id
                    .as_deref()
                    .or(Some(session.binding.preset_id.as_str())),
            )
            .await?
            .2;
        session.binding = binding.clone();
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session).await?;

        Ok(ResolvedSessionConfig { binding })
    }

    pub async fn get_session_character(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<SessionCharacterRecord, ManagerError> {
        self.store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let character = self
            .store
            .get_session_character(session_character_id)
            .await?
            .ok_or_else(|| {
                ManagerError::MissingSessionCharacter(session_character_id.to_owned())
            })?;
        ensure_session_character_belongs(session_id, &character)?;
        Ok(character)
    }

    pub async fn list_session_characters(
        &self,
        session_id: &str,
    ) -> Result<Vec<SessionCharacterRecord>, ManagerError> {
        self.store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        let mut characters = self.store.list_session_characters(session_id).await?;
        characters.sort_by(|left, right| {
            left.created_at_ms
                .cmp(&right.created_at_ms)
                .then_with(|| left.session_character_id.cmp(&right.session_character_id))
        });
        Ok(characters)
    }

    pub async fn update_session_character(
        &self,
        session_id: &str,
        session_character_id: &str,
        update: SessionCharacterUpdate,
    ) -> Result<SessionCharacterRecord, ManagerError> {
        let mut character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        character.display_name = update.display_name;
        character.personality = update.personality;
        character.style = update.style;
        character.system_prompt = update.system_prompt;
        character.updated_at_ms = now_timestamp_ms();
        self.store.save_session_character(character.clone()).await?;
        Ok(character)
    }

    pub async fn delete_session_character(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<SessionCharacterRecord, ManagerError> {
        let character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session
            .snapshot
            .world_state
            .remove_active_character(session_character_id);
        session
            .snapshot
            .world_state
            .character_state
            .remove(session_character_id);
        session
            .snapshot
            .world_state
            .actor_private_memory
            .remove(session_character_id);
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store
            .delete_session_character(session_character_id)
            .await?
            .ok_or_else(|| {
                ManagerError::MissingSessionCharacter(session_character_id.to_owned())
            })?;
        self.store.save_session(session).await?;
        Ok(character)
    }

    pub async fn enter_session_character_scene(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<(SessionRecord, SessionCharacterRecord), ManagerError> {
        let character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session
            .snapshot
            .world_state
            .add_active_character(session_character_id.to_owned());
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;
        Ok((session, character))
    }

    pub async fn leave_session_character_scene(
        &self,
        session_id: &str,
        session_character_id: &str,
    ) -> Result<(SessionRecord, SessionCharacterRecord), ManagerError> {
        let character = self
            .get_session_character(session_id, session_character_id)
            .await?;
        let mut session = self
            .store
            .get_session(session_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSession(session_id.to_owned()))?;
        session
            .snapshot
            .world_state
            .remove_active_character(session_character_id);
        session.updated_at_ms = Some(now_timestamp_ms());
        self.store.save_session(session.clone()).await?;
        Ok((session, character))
    }

    pub async fn run_turn_stream(
        &self,
        session_id: &str,
        player_input: String,
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
        let api_group = self
            .resolve_api_group(&session.binding.api_group_id)
            .await?;
        let preset = self.resolve_preset(&session.binding.preset_id).await?;
        let apis = self.resolve_api_group_bindings(&api_group).await?;
        let runtime_configs = self.registry.build_runtime_configs(
            RuntimeApiRecords {
                director: &apis.director,
                actor: &apis.actor,
                narrator: &apis.narrator,
                keeper: &apis.keeper,
            },
            &preset,
        )?;
        let mut engine = Engine::new(runtime_configs, runtime_state)?;
        let store = Arc::clone(&self.store);
        let session_record = session.clone();

        let stream = stream! {
            let mut updated_session = session_record.clone();
            let mut engine_stream = match engine.run_turn_stream(&player_input).await {
                Ok(stream) => stream,
                Err(error) => {
                    yield Err(ManagerError::Engine(error));
                    return;
                }
            };

            while let Some(event) = engine_stream.next().await {
                match &event {
                    EngineEvent::SessionCharacterCreated { character, snapshot } => {
                        let mut record = character.clone();
                        record.session_id = session_record.session_id.clone();
                        if let Err(error) = store.save_session_character(record).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                    }
                    EngineEvent::SessionCharacterEnteredScene { snapshot, .. }
                    | EngineEvent::SessionCharacterLeftScene { snapshot, .. } => {
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                    }
                    EngineEvent::TurnCompleted { result } => {
                        updated_session.snapshot = result.snapshot.clone();
                        let recorded_at_ms = now_timestamp_ms();
                        updated_session.updated_at_ms = Some(recorded_at_ms);
                        if let Err(error) = store.save_session(updated_session.clone()).await {
                            yield Err(ManagerError::Store(error));
                            return;
                        }
                        let messages = build_session_messages(
                            &session_record.session_id,
                            &session_record,
                            result,
                            recorded_at_ms,
                            match store.list_session_messages(&session_record.session_id).await {
                                Ok(existing) => next_session_message_sequence(&existing),
                                Err(error) => {
                                    yield Err(ManagerError::Store(error));
                                    return;
                                }
                            },
                        );
                        for message in messages {
                            if let Err(error) = store.save_session_message(message).await {
                                yield Err(ManagerError::Store(error));
                                return;
                            }
                        }
                    }
                    EngineEvent::TurnFailed { snapshot, .. } => {
                        updated_session.snapshot = (*snapshot.clone()).clone();
                        updated_session.updated_at_ms = Some(now_timestamp_ms());
                        if let Err(error) = store.save_session(updated_session.clone()).await {
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
            cards.push(self.resolve_character_card(&character, None).await?);
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

        if let Some(planned_story) = non_empty_planned_story(resource.planned_story.as_deref()) {
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
        player_name: Option<String>,
        player_description: String,
    ) -> Result<RuntimeState, ManagerError> {
        let characters = self.load_story_characters(&story.resource_id).await?;
        let mut resolved_characters = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved_characters.push(
                self.resolve_character_card(character, player_name.as_deref())
                    .await?,
            );
        }

        let mut runtime_state = RuntimeState::from_story_graph(
            &story.story_id,
            story.graph.clone(),
            resolved_characters,
            player_description,
            self.resolve_player_schema(&story.player_schema_id).await?,
        )
        .map_err(ManagerError::from)?;
        runtime_state.set_player_name(player_name);
        Ok(runtime_state)
    }

    async fn build_runtime_state_from_session(
        &self,
        story: &StoryRecord,
        session: &SessionRecord,
    ) -> Result<RuntimeState, ManagerError> {
        let (player_name, _player_description) = self
            .resolve_player_identity(session.player_profile_id.as_deref())
            .await?;
        let characters = self.load_story_characters(&story.resource_id).await?;
        let session_characters = self
            .store
            .list_session_characters(&session.session_id)
            .await?;
        let mut resolved_characters =
            Vec::with_capacity(characters.len().saturating_add(session_characters.len()));
        for character in &characters {
            resolved_characters.push(
                self.resolve_character_card(character, player_name.as_deref())
                    .await?,
            );
        }
        for character in &session_characters {
            resolved_characters.push(
                self.resolve_session_character_card(character, player_name.as_deref()),
            );
        }

        let mut runtime_state = RuntimeState::from_snapshot(
            &story.story_id,
            story::runtime_graph::RuntimeStoryGraph::from_story_graph(story.graph.clone())
                .map_err(RuntimeError::GraphBuild)?,
            resolved_characters,
            self.resolve_player_schema(&session.player_schema_id)
                .await?,
            session.snapshot.clone(),
        )
        .map_err(ManagerError::from)?;
        for character in &session_characters {
            runtime_state.register_existing_session_character(&character.session_character_id)?;
        }
        runtime_state.set_player_name(player_name);
        Ok(runtime_state)
    }

    async fn resolve_player_identity(
        &self,
        player_profile_id: Option<&str>,
    ) -> Result<(Option<String>, String), ManagerError> {
        match player_profile_id {
            Some(player_profile_id) => {
                let profile = self
                    .store
                    .get_player_profile(player_profile_id)
                    .await?
                    .ok_or_else(|| {
                        ManagerError::MissingPlayerProfile(player_profile_id.to_owned())
                    })?;
                Ok((Some(profile.display_name), profile.description))
            }
            None => Ok((None, String::new())),
        }
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
        player_name: Option<&str>,
    ) -> Result<CharacterCard, ManagerError> {
        let schema = self
            .resolve_schema_record(&record.content.schema_id)
            .await?;
        Ok(CharacterCard {
            id: record.content.id.clone(),
            name: record.content.name.clone(),
            personality: record.content.personality.clone(),
            style: record.content.style.clone(),
            state_schema: schema.fields,
            system_prompt: record.content.system_prompt.clone(),
        }
        .rendered_with_player_name(player_name))
    }

    fn resolve_session_character_card(
        &self,
        record: &SessionCharacterRecord,
        player_name: Option<&str>,
    ) -> CharacterCard {
        CharacterCard {
            id: record.session_character_id.clone(),
            name: record.display_name.clone(),
            personality: record.personality.clone(),
            style: record.style.clone(),
            state_schema: Default::default(),
            system_prompt: record.system_prompt.clone(),
        }
        .rendered_with_player_name(player_name)
    }

    async fn load_reply_history(
        &self,
        session_id: &str,
    ) -> Result<Vec<ReplyHistoryMessage>, ManagerError> {
        let mut messages = self.store.list_session_messages(session_id).await?;
        messages.sort_by_key(|message| message.sequence);
        let start = messages.len().saturating_sub(DEFAULT_REPLY_HISTORY_LIMIT);
        Ok(messages
            .into_iter()
            .skip(start)
            .map(|message| ReplyHistoryMessage {
                kind: match message.kind {
                    SessionMessageKind::PlayerInput => ReplyHistoryKind::PlayerInput,
                    SessionMessageKind::Narration => ReplyHistoryKind::Narration,
                    SessionMessageKind::Dialogue => ReplyHistoryKind::Dialogue,
                    SessionMessageKind::Action => ReplyHistoryKind::Action,
                },
                turn_index: message.turn_index,
                speaker_id: message.speaker_id,
                speaker_name: message.speaker_name,
                text: message.text,
            })
            .collect())
    }

    async fn resolve_first_available_binding(
        &self,
    ) -> Result<Option<SessionBindingConfig>, ManagerError> {
        let mut api_groups = self.store.list_api_groups().await?;
        let mut presets = self.store.list_presets().await?;
        api_groups.sort_by(|left, right| left.api_group_id.cmp(&right.api_group_id));
        presets.sort_by(|left, right| left.preset_id.cmp(&right.preset_id));

        match (api_groups.first(), presets.first()) {
            (Some(api_group), Some(preset)) => Ok(Some(SessionBindingConfig {
                api_group_id: api_group.api_group_id.clone(),
                preset_id: preset.preset_id.clone(),
            })),
            _ => Ok(None),
        }
    }

    async fn resolve_api_group(&self, api_group_id: &str) -> Result<ApiGroupRecord, ManagerError> {
        self.store
            .get_api_group(api_group_id)
            .await?
            .ok_or_else(|| ManagerError::MissingApiGroup(api_group_id.to_owned()))
    }

    async fn resolve_api(&self, api_id: &str) -> Result<ApiRecord, ManagerError> {
        self.store
            .get_api(api_id)
            .await?
            .ok_or_else(|| ManagerError::MissingApi(api_id.to_owned()))
    }

    async fn resolve_api_group_bindings(
        &self,
        api_group: &ApiGroupRecord,
    ) -> Result<ResolvedApiGroup, ManagerError> {
        Ok(ResolvedApiGroup {
            planner: self.resolve_api(&api_group.agents.planner_api_id).await?,
            architect: self.resolve_api(&api_group.agents.architect_api_id).await?,
            director: self.resolve_api(&api_group.agents.director_api_id).await?,
            actor: self.resolve_api(&api_group.agents.actor_api_id).await?,
            narrator: self.resolve_api(&api_group.agents.narrator_api_id).await?,
            keeper: self.resolve_api(&api_group.agents.keeper_api_id).await?,
            replyer: self.resolve_api(&api_group.agents.replyer_api_id).await?,
        })
    }

    async fn resolve_preset(&self, preset_id: &str) -> Result<PresetRecord, ManagerError> {
        self.store
            .get_preset(preset_id)
            .await?
            .ok_or_else(|| ManagerError::MissingPreset(preset_id.to_owned()))
    }

    async fn resolve_api_group_and_preset(
        &self,
        api_group_id: Option<&str>,
        preset_id: Option<&str>,
    ) -> Result<(ApiGroupRecord, PresetRecord, SessionBindingConfig), ManagerError> {
        let binding = match (api_group_id, preset_id) {
            (Some(api_group_id), Some(preset_id)) => SessionBindingConfig {
                api_group_id: api_group_id.to_owned(),
                preset_id: preset_id.to_owned(),
            },
            (Some(api_group_id), None) => {
                let mut presets = self.store.list_presets().await?;
                presets.sort_by(|left, right| left.preset_id.cmp(&right.preset_id));
                let preset = presets
                    .first()
                    .ok_or(ManagerError::LlmConfigNotInitialized)?;
                SessionBindingConfig {
                    api_group_id: api_group_id.to_owned(),
                    preset_id: preset.preset_id.clone(),
                }
            }
            (None, Some(preset_id)) => {
                let mut api_groups = self.store.list_api_groups().await?;
                api_groups.sort_by(|left, right| left.api_group_id.cmp(&right.api_group_id));
                let api_group = api_groups
                    .first()
                    .ok_or(ManagerError::LlmConfigNotInitialized)?;
                SessionBindingConfig {
                    api_group_id: api_group.api_group_id.clone(),
                    preset_id: preset_id.to_owned(),
                }
            }
            (None, None) => self
                .resolve_first_available_binding()
                .await?
                .ok_or(ManagerError::LlmConfigNotInitialized)?,
        };

        let api_group = self.resolve_api_group(&binding.api_group_id).await?;
        let preset = self.resolve_preset(&binding.preset_id).await?;
        Ok((api_group, preset, binding))
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
    }

    async fn create_generated_schema(
        &self,
        display_name: String,
        tags: Vec<String>,
        fields: std::collections::HashMap<String, StateFieldSchema>,
    ) -> Result<SchemaRecord, ManagerError> {
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

    fn recent_draft_nodes(&self, draft: &StoryDraftRecord) -> Vec<NarrativeNode> {
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
}

fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_millis()
        .min(u128::from(u64::MAX)) as u64
}

fn non_empty_planned_story(planned_story: Option<&str>) -> Option<String> {
    planned_story
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn effective_planned_story_text(resource: &StoryResourcesRecord) -> String {
    non_empty_planned_story(resource.planned_story.as_deref())
        .unwrap_or_else(|| resource.story_concept.clone())
}

fn extract_outline_sections(planned_story: &str) -> Vec<String> {
    let mut sections = Vec::new();
    let mut in_suggested_beats = false;

    for raw_line in planned_story.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(opening) = line.strip_prefix("Opening Situation:") {
            let opening = opening.trim();
            if !opening.is_empty() {
                sections.push(opening.to_owned());
            }
            in_suggested_beats = false;
            continue;
        }

        if line == "Suggested Beats:" {
            in_suggested_beats = true;
            continue;
        }

        if matches!(
            line,
            "Title:" | "Core Conflict:" | "Character Roles:" | "State Hints:"
        ) {
            in_suggested_beats = false;
            continue;
        }

        if in_suggested_beats {
            let beat = line
                .trim_start_matches(|c: char| {
                    c == '-' || c == '*' || c.is_ascii_digit() || c == '.' || c == ')'
                })
                .trim();
            if !beat.is_empty() {
                sections.push(beat.to_owned());
            }
        }
    }

    if sections.is_empty() {
        planned_story
            .split("\n\n")
            .map(str::trim)
            .filter(|section| !section.is_empty() && !section.ends_with(':'))
            .map(ToOwned::to_owned)
            .collect()
    } else {
        sections
    }
}

fn build_graph_summary(graph: &StoryGraph) -> Vec<GraphSummaryNode> {
    graph
        .nodes
        .iter()
        .map(|node| GraphSummaryNode {
            id: node.id.clone(),
            title: node.title.clone(),
            scene_summary: truncate_text(&node.scene, 200),
            goal: truncate_text(&node.goal, 120),
            characters: node.characters.clone(),
            transition_targets: node
                .transitions
                .iter()
                .map(|transition| transition.to.clone())
                .collect(),
        })
        .collect()
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (idx, ch) in text.chars().enumerate() {
        if idx >= max_chars {
            out.push_str("...");
            break;
        }
        out.push(ch);
    }
    out
}

fn merge_story_chunk(
    graph: &mut StoryGraph,
    nodes: &[NarrativeNode],
    transition_patches: &[NodeTransitionPatch],
) -> Result<(), ManagerError> {
    for node in nodes {
        if graph.has_node(&node.id) {
            return Err(ManagerError::InvalidDraft(format!(
                "architect draft returned duplicate node id '{}'",
                node.id
            )));
        }
        graph.nodes.push(node.clone());
    }

    apply_transition_patches(graph, transition_patches)?;
    validate_story_graph(graph)?;
    Ok(())
}

fn apply_transition_patches(
    graph: &mut StoryGraph,
    transition_patches: &[NodeTransitionPatch],
) -> Result<(), ManagerError> {
    for patch in transition_patches {
        let node = graph.get_node_mut(&patch.node_id).ok_or_else(|| {
            ManagerError::InvalidDraft(format!(
                "architect draft attempted to patch missing node '{}'",
                patch.node_id
            ))
        })?;
        node.transitions.extend(patch.add_transitions.clone());
    }
    Ok(())
}

fn validate_story_graph(graph: &StoryGraph) -> Result<(), ManagerError> {
    if graph.is_empty() {
        return Err(ManagerError::InvalidDraft(
            "story graph must contain at least one node".to_owned(),
        ));
    }

    let mut node_ids = HashSet::new();
    for node in &graph.nodes {
        if !node_ids.insert(node.id.clone()) {
            return Err(ManagerError::InvalidDraft(format!(
                "story graph contains duplicate node id '{}'",
                node.id
            )));
        }
    }

    if !node_ids.contains(graph.start_node()) {
        return Err(ManagerError::InvalidDraft(format!(
            "story graph start node '{}' does not exist",
            graph.start_node()
        )));
    }

    for node in &graph.nodes {
        for transition in &node.transitions {
            if !node_ids.contains(&transition.to) {
                return Err(ManagerError::InvalidDraft(format!(
                    "transition from '{}' points to missing node '{}'",
                    node.id, transition.to
                )));
            }
        }
    }

    validate_graph_state_conventions(graph)
        .map_err(|error| ManagerError::InvalidDraft(error.to_string()))?;

    Ok(())
}

fn ensure_session_character_belongs(
    session_id: &str,
    character: &SessionCharacterRecord,
) -> Result<(), ManagerError> {
    if character.session_id == session_id {
        return Ok(());
    }

    Err(ManagerError::MissingSessionCharacter(
        character.session_character_id.clone(),
    ))
}

fn next_session_message_sequence(existing: &[SessionMessageRecord]) -> u64 {
    existing
        .iter()
        .map(|message| message.sequence)
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(0)
}

fn build_session_messages(
    session_id: &str,
    session: &SessionRecord,
    result: &EngineTurnResult,
    recorded_at_ms: u64,
    starting_sequence: u64,
) -> Vec<SessionMessageRecord> {
    let mut next_sequence = starting_sequence;
    let mut messages = vec![SessionMessageRecord {
        message_id: format!("{}-message-{}", session_id, next_sequence),
        session_id: session.session_id.clone(),
        kind: SessionMessageKind::PlayerInput,
        sequence: next_sequence,
        turn_index: result.turn_index,
        recorded_at_ms,
        created_at_ms: recorded_at_ms,
        updated_at_ms: recorded_at_ms,
        speaker_id: "player".to_owned(),
        speaker_name: "Player".to_owned(),
        text: result.player_input.clone(),
    }];
    next_sequence = next_sequence.saturating_add(1);

    for beat in &result.completed_beats {
        match beat {
            ExecutedBeat::Narrator { response, .. } => {
                let text = response.text.trim();
                if !text.is_empty() {
                    messages.push(SessionMessageRecord {
                        message_id: format!("{}-message-{}", session_id, next_sequence),
                        session_id: session.session_id.clone(),
                        kind: SessionMessageKind::Narration,
                        sequence: next_sequence,
                        turn_index: result.turn_index,
                        recorded_at_ms,
                        created_at_ms: recorded_at_ms,
                        updated_at_ms: recorded_at_ms,
                        speaker_id: "narrator".to_owned(),
                        speaker_name: "Narrator".to_owned(),
                        text: text.to_owned(),
                    });
                    next_sequence = next_sequence.saturating_add(1);
                }
            }
            ExecutedBeat::Actor { response, .. } => {
                for segment in &response.segments {
                    let kind = match segment.kind {
                        ActorSegmentKind::Dialogue => Some(SessionMessageKind::Dialogue),
                        ActorSegmentKind::Action => Some(SessionMessageKind::Action),
                        ActorSegmentKind::Thought => None,
                    };

                    let Some(kind) = kind else {
                        continue;
                    };

                    let text = segment.text.trim();
                    if text.is_empty() {
                        continue;
                    }

                    messages.push(SessionMessageRecord {
                        message_id: format!("{}-message-{}", session_id, next_sequence),
                        session_id: session.session_id.clone(),
                        kind,
                        sequence: next_sequence,
                        turn_index: result.turn_index,
                        recorded_at_ms,
                        created_at_ms: recorded_at_ms,
                        updated_at_ms: recorded_at_ms,
                        speaker_id: response.speaker_id.clone(),
                        speaker_name: response.speaker_name.clone(),
                        text: text.to_owned(),
                    });
                    next_sequence = next_sequence.saturating_add(1);
                }
            }
        }
    }

    messages
}

#[derive(Debug, thiserror::Error)]
pub enum ManagerError {
    #[error("llm engine config is not initialized")]
    LlmConfigNotInitialized,
    #[error("api '{0}' not found")]
    MissingApi(String),
    #[error("api group '{0}' not found")]
    MissingApiGroup(String),
    #[error("preset '{0}' not found")]
    MissingPreset(String),
    #[error("schema '{0}' not found")]
    MissingSchema(String),
    #[error("character '{0}' not found")]
    MissingCharacter(String),
    #[error("player profile '{0}' not found")]
    MissingPlayerProfile(String),
    #[error("story resources '{0}' not found")]
    MissingStoryResources(String),
    #[error("story draft '{0}' not found")]
    MissingStoryDraft(String),
    #[error("story '{0}' not found")]
    MissingStory(String),
    #[error("session '{0}' not found")]
    MissingSession(String),
    #[error("session character '{0}' not found")]
    MissingSessionCharacter(String),
    #[error("character_ids cannot be empty")]
    EmptyCharacterIds,
    #[error("invalid story draft: {0}")]
    InvalidDraft(String),
    #[error(transparent)]
    Architect(#[from] ArchitectError),
    #[error(transparent)]
    Replyer(#[from] ReplyerError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
    #[error(transparent)]
    Registry(#[from] RegistryError),
    #[error(transparent)]
    Store(#[from] StoreError),
}
