use std::collections::{BTreeMap, HashMap};

use agents::actor::CharacterCard;
use agents::replyer::ReplyHistoryMessage;
use serde::Serialize;
use serde_json::Value;
use state::{ActorMemoryEntry, ActorMemoryKind, StateFieldSchema};
use store::{AgentPresetConfig, PromptModuleId, StoryDraftRecord};
use story::{Condition, ConditionOperator, ConditionScope, NarrativeNode, Transition};

use crate::lorebook::build_lorebook_prompt_sections;
use crate::prompt::{
    ArchitectPromptMode, PromptAgentKind, PromptPreview, PromptPreviewActorPurpose,
    PromptPreviewKeeperPhase, PromptPreviewNarratorPurpose, RuntimePromptPreviewOptions,
    compile_architect_prompt_module, compile_architect_prompt_preview_profile,
    compile_prompt_module, compile_prompt_preview_profile, render_module_preview,
    render_profile_preview,
};
use crate::registry::RegistryError;
use crate::{RuntimeError, RuntimeState, StoryResources};
use crate::history::{
    resolve_actor_private_memory_limit, resolve_actor_shared_history_limit,
    resolve_director_shared_history_limit, resolve_narrator_shared_history_limit,
    resolve_replyer_session_history_limit, resolve_runtime_shared_memory_limit,
};

use super::util::{build_graph_summary, build_reply_history, truncate_text};
use super::{DEFAULT_ARCHITECT_CHUNK_NODE_COUNT, EngineManager, ManagerError};

impl EngineManager {
    pub async fn preview_prompt_template(
        &self,
        preset_id: &str,
        agent: PromptAgentKind,
        module_id: Option<&PromptModuleId>,
        architect_mode: Option<ArchitectPromptMode>,
    ) -> Result<PromptPreview, ManagerError> {
        let preset = self.resolve_preset(preset_id).await?;
        render_preview_from_config(
            agent_preset_config(&preset.agents, agent),
            agent,
            module_id,
            architect_mode,
            true,
            &HashMap::new(),
        )
    }

    pub async fn preview_prompt_runtime_for_resource(
        &self,
        preset_id: &str,
        agent: PromptAgentKind,
        module_id: Option<&PromptModuleId>,
        architect_mode: Option<ArchitectPromptMode>,
        resource_id: &str,
    ) -> Result<PromptPreview, ManagerError> {
        let preset = self.resolve_preset(preset_id).await?;
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;
        let story_resources = self.build_engine_story_resources(&resource).await?;
        let context = match agent {
            PromptAgentKind::Planner => planner_context(&story_resources),
            PromptAgentKind::Architect => {
                let mode = architect_mode.unwrap_or(ArchitectPromptMode::Graph);
                if mode != ArchitectPromptMode::Graph {
                    return Err(ManagerError::InvalidPromptPreview(
                        "resource runtime preview supports architect graph mode only".to_owned(),
                    ));
                }
                architect_graph_context(&story_resources)
            }
            _ => {
                return Err(ManagerError::InvalidPromptPreview(
                    "resource runtime preview supports planner or architect graph only".to_owned(),
                ));
            }
        };

        render_preview_from_config(
            agent_preset_config(&preset.agents, agent),
            agent,
            module_id,
            architect_mode,
            false,
            &context,
        )
    }

    pub async fn preview_prompt_runtime_for_draft(
        &self,
        preset_id: &str,
        module_id: Option<&PromptModuleId>,
        architect_mode: ArchitectPromptMode,
        draft_id: &str,
    ) -> Result<PromptPreview, ManagerError> {
        let preset = self.resolve_preset(preset_id).await?;
        let draft = self
            .store
            .get_story_draft(draft_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryDraft(draft_id.to_owned()))?;
        let resource = self
            .store
            .get_story_resources(&draft.resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(draft.resource_id.clone()))?;
        let story_resources = self.build_engine_story_resources(&resource).await?;
        let context = match architect_mode {
            ArchitectPromptMode::Graph => {
                return Err(ManagerError::InvalidPromptPreview(
                    "draft runtime preview supports architect draft modes only".to_owned(),
                ));
            }
            ArchitectPromptMode::DraftInit => {
                architect_draft_init_context(self, &story_resources, &draft).await?
            }
            ArchitectPromptMode::DraftContinue => {
                architect_draft_continue_context(self, &story_resources, &draft).await?
            }
        };

        render_preview_from_config(
            &preset.agents.architect,
            PromptAgentKind::Architect,
            module_id,
            Some(architect_mode),
            false,
            &context,
        )
    }

    pub async fn preview_prompt_runtime_for_session(
        &self,
        preset_id: &str,
        agent: PromptAgentKind,
        module_id: Option<&PromptModuleId>,
        session_id: &str,
        options: RuntimePromptPreviewOptions,
    ) -> Result<PromptPreview, ManagerError> {
        let preset = self.resolve_preset(preset_id).await?;
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
        let shared_memory_limit = resolve_runtime_shared_memory_limit(
            &preset.agents.director,
            &preset.agents.actor,
            &preset.agents.narrator,
        );

        let context = match agent {
            PromptAgentKind::Director => {
                director_context(
                    &runtime_state,
                    &preset.agents.director,
                    &options,
                    shared_memory_limit,
                )?
            }
            PromptAgentKind::Actor => actor_context(
                &runtime_state,
                &preset.agents.actor,
                &options,
                shared_memory_limit,
            )?,
            PromptAgentKind::Narrator => {
                narrator_context(
                    &runtime_state,
                    &preset.agents.narrator,
                    &options,
                    shared_memory_limit,
                )?
            }
            PromptAgentKind::Keeper => keeper_context(&runtime_state, &options, shared_memory_limit)?,
            PromptAgentKind::Replyer => {
                let history = self
                    .load_reply_history_for_preview(
                        session_id,
                        resolve_replyer_session_history_limit(&preset.agents.replyer),
                    )
                    .await?;
                replyer_context(&runtime_state, &history, &options)?
            }
            PromptAgentKind::Planner | PromptAgentKind::Architect => {
                return Err(ManagerError::InvalidPromptPreview(
                    "session runtime preview supports runtime agents only".to_owned(),
                ));
            }
        };

        render_preview_from_config(
            agent_preset_config(&preset.agents, agent),
            agent,
            module_id,
            None,
            false,
            &context,
        )
    }

    async fn load_reply_history_for_preview(
        &self,
        session_id: &str,
        history_limit: usize,
    ) -> Result<Vec<ReplyHistoryMessage>, ManagerError> {
        let mut messages = self.store.list_session_messages(session_id).await?;
        messages.sort_by_key(|message| message.sequence);
        Ok(build_reply_history(messages, history_limit))
    }
}

fn render_preview_from_config(
    config: &AgentPresetConfig,
    agent: PromptAgentKind,
    module_id: Option<&PromptModuleId>,
    architect_mode: Option<ArchitectPromptMode>,
    include_placeholders: bool,
    context: &HashMap<String, String>,
) -> Result<PromptPreview, ManagerError> {
    match module_id {
        Some(module_id) => {
            let compiled = if agent == PromptAgentKind::Architect {
                compile_architect_prompt_module(
                    config,
                    architect_mode.unwrap_or(ArchitectPromptMode::Graph),
                    module_id,
                )
            } else {
                compile_prompt_module(agent, config, module_id)
            }
            .map_err(prompt_compile_error)?
            .ok_or_else(|| {
                ManagerError::InvalidPromptPreview(format!(
                    "module '{}' is not available for {:?}",
                    module_id.as_str(),
                    agent
                ))
            })?;

            Ok(render_module_preview(
                compiled.message_role,
                compiled.module.as_ref(),
                include_placeholders,
                |key| context.get(key).cloned(),
            ))
        }
        None => {
            let profile = if agent == PromptAgentKind::Architect {
                compile_architect_prompt_preview_profile(
                    config,
                    architect_mode.unwrap_or(ArchitectPromptMode::Graph),
                )
                .map_err(prompt_compile_error)?
            } else {
                compile_prompt_preview_profile(agent, config).map_err(prompt_compile_error)?
            };

            Ok(render_profile_preview(
                &profile,
                include_placeholders,
                |key| context.get(key).cloned(),
            ))
        }
    }
}

fn prompt_compile_error(error: crate::prompt::PromptConfigError) -> ManagerError {
    ManagerError::Registry(RegistryError::PromptConfig(error.to_string()))
}

fn agent_preset_config(
    agents: &store::PresetAgentConfigs,
    agent: PromptAgentKind,
) -> &AgentPresetConfig {
    match agent {
        PromptAgentKind::Planner => &agents.planner,
        PromptAgentKind::Architect => &agents.architect,
        PromptAgentKind::Director => &agents.director,
        PromptAgentKind::Actor => &agents.actor,
        PromptAgentKind::Narrator => &agents.narrator,
        PromptAgentKind::Keeper => &agents.keeper,
        PromptAgentKind::Replyer => &agents.replyer,
    }
}

fn planner_context(resources: &StoryResources) -> HashMap<String, String> {
    let lorebook_sections = build_lorebook_prompt_sections(
        resources.lorebook_entries(),
        &[
            resources.story_concept(),
            resources.planned_story().unwrap_or(""),
        ],
    );
    let mut context = HashMap::new();
    context.insert(
        "story_concept".to_owned(),
        resources.story_concept().to_owned(),
    );
    context.insert(
        "available_characters".to_owned(),
        render_character_summaries_from_cards(resources.character_cards(), None),
    );
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    context
}

fn architect_graph_context(resources: &StoryResources) -> HashMap<String, String> {
    let lorebook_sections = build_lorebook_prompt_sections(
        resources.lorebook_entries(),
        &[
            resources.story_concept(),
            resources.planned_story().unwrap_or(""),
        ],
    );
    let mut context = HashMap::new();
    context.insert(
        "story_concept".to_owned(),
        resources.story_concept().to_owned(),
    );
    context.insert(
        "planned_story".to_owned(),
        resources.planned_story().unwrap_or("null").to_owned(),
    );
    context.insert(
        "available_characters".to_owned(),
        render_architect_character_summaries(resources.character_cards()),
    );
    context.insert(
        "world_state_schema_seed".to_owned(),
        render_compact_schema_text(
            resources
                .world_state_schema_seed()
                .map(|schema| &schema.fields),
        ),
    );
    context.insert(
        "player_state_schema_seed".to_owned(),
        render_compact_schema_text(
            resources
                .player_state_schema_seed()
                .map(|schema| &schema.fields),
        ),
    );
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    context
}

async fn architect_draft_init_context(
    manager: &EngineManager,
    resources: &StoryResources,
    draft: &StoryDraftRecord,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_section = draft.outline_sections.first().cloned().ok_or_else(|| {
        ManagerError::InvalidPromptPreview("story draft has no outline sections".to_owned())
    })?;
    let lorebook_sections = manager.story_generation_lorebook_sections(
        resources,
        &[
            resources.story_concept(),
            draft.planned_story.as_str(),
            current_section.as_str(),
        ],
    );
    let mut context = architect_graph_context(resources);
    context.insert("current_section".to_owned(), current_section);
    context.insert("section_index".to_owned(), "0".to_owned());
    context.insert(
        "total_sections".to_owned(),
        draft.outline_sections.len().to_string(),
    );
    context.insert(
        "target_node_count".to_owned(),
        DEFAULT_ARCHITECT_CHUNK_NODE_COUNT.to_string(),
    );
    context.insert("graph_summary".to_owned(), "[]".to_owned());
    context.insert("recent_section_detail".to_owned(), "[]".to_owned());
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

async fn architect_draft_continue_context(
    manager: &EngineManager,
    resources: &StoryResources,
    draft: &StoryDraftRecord,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_section = draft
        .outline_sections
        .get(draft.next_section_index)
        .cloned()
        .ok_or_else(|| {
            ManagerError::InvalidPromptPreview(
                "story draft has no remaining outline section".to_owned(),
            )
        })?;
    let world_schema = manager.resolve_world_schema(&draft.world_schema_id).await?;
    let player_schema = manager
        .resolve_player_schema(&draft.player_schema_id)
        .await?;
    let recent_nodes = manager.recent_draft_nodes(draft);
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
        manager.story_generation_lorebook_sections(resources, &lorebook_match_inputs);

    let mut context = HashMap::new();
    context.insert(
        "story_concept".to_owned(),
        resources.story_concept().to_owned(),
    );
    context.insert(
        "available_characters".to_owned(),
        render_architect_character_summaries(resources.character_cards()),
    );
    context.insert(
        "world_state_schema".to_owned(),
        render_compact_schema_text(Some(&world_schema.fields)),
    );
    context.insert(
        "player_state_schema".to_owned(),
        render_compact_schema_text(Some(&player_schema.fields)),
    );
    context.insert(
        "section_summaries".to_owned(),
        render_section_summaries(&draft.section_summaries),
    );
    context.insert("current_section".to_owned(), current_section);
    context.insert(
        "section_index".to_owned(),
        draft.next_section_index.to_string(),
    );
    context.insert(
        "total_sections".to_owned(),
        draft.outline_sections.len().to_string(),
    );
    context.insert(
        "target_node_count".to_owned(),
        DEFAULT_ARCHITECT_CHUNK_NODE_COUNT.to_string(),
    );
    context.insert(
        "graph_summary".to_owned(),
        compact_json(&build_graph_summary(&draft.partial_graph)),
    );
    context.insert(
        "recent_section_detail".to_owned(),
        compact_json(
            &recent_nodes
                .iter()
                .map(compact_recent_section_node)
                .collect::<Vec<_>>(),
        ),
    );
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

fn director_context(
    runtime_state: &RuntimeState,
    config: &AgentPresetConfig,
    options: &RuntimePromptPreviewOptions,
    shared_memory_limit: usize,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_node = runtime_state.current_node()?;
    let lorebook_sections = runtime_lorebook_sections(
        runtime_state,
        &current_node.id,
        options.player_input.as_deref().unwrap_or(""),
        shared_memory_limit,
    );
    let mut context = HashMap::new();
    context.insert(
        "player".to_owned(),
        render_player(
            runtime_state.player_name(),
            runtime_state.player_description(),
        ),
    );
    context.insert(
        "current_cast".to_owned(),
        render_runtime_character_summaries(
            runtime_state,
            runtime_state.world_state().active_characters(),
        )?,
    );
    context.insert("current_node".to_owned(), render_node(current_node));
    context.insert(
        "player_state_schema".to_owned(),
        render_state_schema_fields(&runtime_state.player_state_schema().fields),
    );
    context.insert("transitioned_this_turn".to_owned(), compact_json(&false));
    context.insert(
        "world_state".to_owned(),
        render_director_world_state(runtime_state.world_state()),
    );
    context.insert(
        "shared_history".to_owned(),
        render_actor_history(
            &runtime_state
                .world_state()
                .recent_actor_shared_history(resolve_director_shared_history_limit(config)),
        ),
    );
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

fn actor_context(
    runtime_state: &RuntimeState,
    config: &AgentPresetConfig,
    options: &RuntimePromptPreviewOptions,
    shared_memory_limit: usize,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_node = runtime_state.current_node()?;
    let character_id = options.character_id.as_deref().ok_or_else(|| {
        ManagerError::InvalidPromptPreview("actor runtime preview requires character_id".to_owned())
    })?;
    let character = runtime_state
        .character_card(character_id)
        .ok_or_else(|| RuntimeError::MissingCharacterCard(character_id.to_owned()))
        .map_err(ManagerError::Runtime)?;
    let lorebook_sections = runtime_lorebook_sections(
        runtime_state,
        &current_node.id,
        options.player_input.as_deref().unwrap_or(""),
        shared_memory_limit,
    );
    let mut context = HashMap::new();
    context.insert(
        "player".to_owned(),
        render_player(
            runtime_state.player_name(),
            runtime_state.player_description(),
        ),
    );
    context.insert(
        "actor_purpose".to_owned(),
        render_actor_purpose(
            options
                .actor_purpose
                .unwrap_or(PromptPreviewActorPurpose::ReactToPlayer),
        ),
    );
    context.insert(
        "current_cast".to_owned(),
        render_runtime_character_summaries(
            runtime_state,
            runtime_state.world_state().active_characters(),
        )?,
    );
    context.insert("current_node".to_owned(), render_node(current_node));
    context.insert(
        "world_state".to_owned(),
        render_actor_world_state(runtime_state.world_state()),
    );
    context.insert(
        "shared_history".to_owned(),
        render_actor_history(
            &runtime_state
                .world_state()
                .recent_actor_shared_history(resolve_actor_shared_history_limit(config)),
        ),
    );
    context.insert(
        "private_memory".to_owned(),
        render_actor_history(
            &runtime_state
                .world_state()
                .recent_actor_private_memory(
                    &character.id,
                    resolve_actor_private_memory_limit(config),
                ),
        ),
    );
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

fn narrator_context(
    runtime_state: &RuntimeState,
    config: &AgentPresetConfig,
    options: &RuntimePromptPreviewOptions,
    shared_memory_limit: usize,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_node = runtime_state.current_node()?;
    let purpose = options
        .narrator_purpose
        .unwrap_or(PromptPreviewNarratorPurpose::DescribeScene);
    let previous_node = previous_node(runtime_state, options.previous_node_id.as_deref())?;
    let lorebook_sections = runtime_lorebook_sections(
        runtime_state,
        &current_node.id,
        options.player_input.as_deref().unwrap_or(""),
        shared_memory_limit,
    );
    let mut context = HashMap::new();
    context.insert(
        "player".to_owned(),
        render_player(
            runtime_state.player_name(),
            runtime_state.player_description(),
        ),
    );
    context.insert(
        "narrator_purpose".to_owned(),
        render_narrator_purpose(purpose),
    );
    context.insert(
        "previous_node".to_owned(),
        previous_node
            .map(render_optional_node_from_ref)
            .unwrap_or_else(|| "null".to_owned()),
    );
    context.insert(
        "previous_cast".to_owned(),
        match previous_node {
            Some(node) => render_runtime_character_summaries(runtime_state, &node.characters)?,
            None => "null".to_owned(),
        },
    );
    context.insert("current_node".to_owned(), render_node(current_node));
    context.insert(
        "current_cast".to_owned(),
        render_runtime_character_summaries(
            runtime_state,
            runtime_state.world_state().active_characters(),
        )?,
    );
    context.insert(
        "player_state_schema".to_owned(),
        render_state_schema_fields(&runtime_state.player_state_schema().fields),
    );
    context.insert(
        "world_state".to_owned(),
        render_observable_world_state(runtime_state.world_state()),
    );
    context.insert(
        "shared_history".to_owned(),
        render_actor_history(
            &runtime_state
                .world_state()
                .recent_actor_shared_history(resolve_narrator_shared_history_limit(config)),
        ),
    );
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

fn keeper_context(
    runtime_state: &RuntimeState,
    options: &RuntimePromptPreviewOptions,
    shared_memory_limit: usize,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_node = runtime_state.current_node()?;
    let previous_node = previous_node(runtime_state, options.previous_node_id.as_deref())?;
    let lorebook_sections = runtime_lorebook_sections(
        runtime_state,
        &current_node.id,
        options.player_input.as_deref().unwrap_or(""),
        shared_memory_limit,
    );
    let mut context = HashMap::new();
    context.insert(
        "player".to_owned(),
        render_player(
            runtime_state.player_name(),
            runtime_state.player_description(),
        ),
    );
    context.insert(
        "keeper_phase".to_owned(),
        render_keeper_phase(
            options
                .keeper_phase
                .unwrap_or(PromptPreviewKeeperPhase::AfterPlayerInput),
        ),
    );
    context.insert(
        "previous_node".to_owned(),
        previous_node
            .map(render_keeper_node)
            .unwrap_or_else(|| "null".to_owned()),
    );
    context.insert(
        "node_change".to_owned(),
        render_keeper_node_change(previous_node, current_node),
    );
    context.insert(
        "previous_cast".to_owned(),
        match previous_node {
            Some(node) => render_runtime_character_summaries(runtime_state, &node.characters)?,
            None => "null".to_owned(),
        },
    );
    context.insert("current_node".to_owned(), render_keeper_node(current_node));
    context.insert(
        "progression_hints".to_owned(),
        render_keeper_progression_hints(current_node),
    );
    context.insert(
        "current_cast".to_owned(),
        render_runtime_character_summaries(
            runtime_state,
            runtime_state.world_state().active_characters(),
        )?,
    );
    context.insert(
        "player_state_schema".to_owned(),
        render_state_schema_fields(&runtime_state.player_state_schema().fields),
    );
    context.insert(
        "player_input".to_owned(),
        options.player_input.clone().unwrap_or_default(),
    );
    context.insert(
        "world_state".to_owned(),
        render_observable_world_state(runtime_state.world_state()),
    );
    context.insert("completed_beats".to_owned(), "- none".to_owned());
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

fn replyer_context(
    runtime_state: &RuntimeState,
    history: &[ReplyHistoryMessage],
    options: &RuntimePromptPreviewOptions,
) -> Result<HashMap<String, String>, ManagerError> {
    let current_node = runtime_state.current_node()?;
    let lorebook_sections = replyer_lorebook_sections(runtime_state, current_node, history);
    let mut context = HashMap::new();
    context.insert(
        "player".to_owned(),
        render_player(
            runtime_state.player_name(),
            runtime_state.player_description(),
        ),
    );
    context.insert(
        "reply_limit".to_owned(),
        options.reply_limit.unwrap_or(3).to_string(),
    );
    context.insert(
        "current_cast".to_owned(),
        render_runtime_character_summaries(
            runtime_state,
            runtime_state.world_state().active_characters(),
        )?,
    );
    context.insert("current_node".to_owned(), render_node(current_node));
    context.insert(
        "player_state_schema".to_owned(),
        render_state_schema_fields(&runtime_state.player_state_schema().fields),
    );
    context.insert(
        "world_state".to_owned(),
        render_observable_world_state(runtime_state.world_state()),
    );
    context.insert("session_history".to_owned(), render_reply_history(history));
    insert_optional(&mut context, "lorebook_base", lorebook_sections.base);
    insert_optional(&mut context, "lorebook_matched", lorebook_sections.matched);
    Ok(context)
}

fn previous_node<'a>(
    runtime_state: &'a RuntimeState,
    previous_node_id: Option<&str>,
) -> Result<Option<&'a NarrativeNode>, ManagerError> {
    let Some(previous_node_id) = previous_node_id else {
        return Ok(None);
    };

    let index = runtime_state
        .runtime_graph()
        .get_node_index(previous_node_id)
        .ok_or_else(|| {
            ManagerError::InvalidPromptPreview(format!(
                "previous_node_id '{}' was not found in runtime graph",
                previous_node_id
            ))
        })?;

    runtime_state
        .runtime_graph()
        .graph
        .node_weight(index)
        .map(Some)
        .ok_or_else(|| {
            ManagerError::InvalidPromptPreview(format!(
                "previous_node_id '{}' was not found in runtime graph",
                previous_node_id
            ))
        })
}

fn runtime_lorebook_sections(
    runtime_state: &RuntimeState,
    current_node_id: &str,
    player_input: &str,
    shared_memory_limit: usize,
) -> crate::lorebook::LorebookPromptSections {
    let node_texts = runtime_state
        .runtime_graph()
        .get_node_index(current_node_id)
        .and_then(|index| runtime_state.runtime_graph().graph.node_weight(index))
        .map(|node| vec![node.title.as_str(), node.scene.as_str(), node.goal.as_str()])
        .unwrap_or_default();
    let history_texts = runtime_state
        .world_state()
        .actor_shared_history()
        .iter()
        .rev()
        .take(shared_memory_limit)
        .map(|entry| entry.text.as_str())
        .collect::<Vec<_>>();
    let mut match_inputs = Vec::with_capacity(node_texts.len() + history_texts.len() + 1);
    match_inputs.extend(node_texts);
    match_inputs.extend(history_texts);
    match_inputs.push(player_input);
    build_lorebook_prompt_sections(runtime_state.lorebook_entries(), &match_inputs)
}

fn replyer_lorebook_sections(
    runtime_state: &RuntimeState,
    current_node: &NarrativeNode,
    history: &[ReplyHistoryMessage],
) -> crate::lorebook::LorebookPromptSections {
    let mut match_inputs = vec![
        current_node.title.as_str(),
        current_node.scene.as_str(),
        current_node.goal.as_str(),
    ];
    match_inputs.extend(
        history
            .iter()
            .map(|message| message.text.as_str())
            .filter(|text| !text.trim().is_empty()),
    );
    build_lorebook_prompt_sections(runtime_state.lorebook_entries(), &match_inputs)
}

fn insert_optional(map: &mut HashMap<String, String>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_owned(), value);
    }
}

fn render_character_summaries_from_cards(
    cards: &[CharacterCard],
    player_name: Option<&str>,
) -> String {
    render_list_lines(
        &cards
            .iter()
            .map(|card| {
                let summary = card.summary_with_template_values(player_name, None);
                format!(
                    "{} | {} | personality={} | style={} | state_schema={}",
                    summary.id,
                    summary.name,
                    normalize_inline_text(&summary.personality),
                    normalize_inline_text(&summary.style),
                    render_state_schema_fields(&summary.state_schema),
                )
            })
            .collect::<Vec<_>>(),
    )
}

fn render_runtime_character_summaries(
    runtime_state: &RuntimeState,
    character_ids: &[String],
) -> Result<String, ManagerError> {
    let lines = character_ids
        .iter()
        .map(|character_id| {
            let card = runtime_state
                .character_card(character_id)
                .ok_or_else(|| RuntimeError::MissingCharacterCard(character_id.clone()))
                .map_err(ManagerError::Runtime)?;
            let summary = card.summary_with_template_values(
                runtime_state.player_name(),
                runtime_state.world_state().character_states(character_id),
            );
            Ok(format!(
                "{} | {} | personality={} | style={} | state_schema={}",
                summary.id,
                summary.name,
                normalize_inline_text(&summary.personality),
                normalize_inline_text(&summary.style),
                render_state_schema_fields(&summary.state_schema),
            ))
        })
        .collect::<Result<Vec<_>, ManagerError>>()?;

    Ok(render_list_lines(&lines))
}

fn render_architect_character_summaries(cards: &[CharacterCard]) -> String {
    if cards.is_empty() {
        return "- none".to_owned();
    }

    cards
        .iter()
        .map(|card| {
            let summary = card.summary_with_template_values(None, None);
            let role_summary =
                truncate_text(&format!("{} | {}", summary.personality, summary.style), 180);
            format!(
                "- {} | {} | role={} | state_schema={}",
                summary.id,
                summary.name,
                role_summary,
                render_state_schema_fields_without_description(&summary.state_schema),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_player(name: Option<&str>, description: &str) -> String {
    render_sections(&[
        ("name", name.unwrap_or("User").to_owned()),
        ("description", description.to_owned()),
    ])
}

fn render_state_schema_fields(fields: &HashMap<String, StateFieldSchema>) -> String {
    if fields.is_empty() {
        return "none".to_owned();
    }

    fields
        .iter()
        .map(|(key, field)| (key.clone(), field))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(key, field)| {
            let mut line = format!("{key}:{}", compact_json(&field.value_type));
            if let Some(default) = &field.default {
                line.push_str(&format!(" default={}", compact_json(default)));
            }
            if let Some(enum_values) = &field.enum_values {
                line.push_str(&format!(" enum={}", compact_json(enum_values)));
            }
            if let Some(description) = &field.description {
                line.push_str(&format!(" desc={}", normalize_inline_text(description)));
            }
            line
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_state_schema_fields_without_description(
    fields: &HashMap<String, StateFieldSchema>,
) -> String {
    if fields.is_empty() {
        return "none".to_owned();
    }

    fields
        .iter()
        .map(|(key, field)| (key.clone(), field))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(key, field)| {
            let mut line = format!("{key}:{}", compact_json(&field.value_type));
            if let Some(default) = &field.default {
                line.push_str(&format!(" default={}", compact_json(default)));
            }
            if let Some(enum_values) = &field.enum_values {
                line.push_str(&format!(" enum={}", compact_json(enum_values)));
            }
            line
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_compact_schema_text(fields: Option<&HashMap<String, StateFieldSchema>>) -> String {
    fields
        .map(render_state_schema_fields_without_description)
        .unwrap_or_else(|| "null".to_owned())
}

fn render_section_summaries(section_summaries: &[String]) -> String {
    if section_summaries.is_empty() {
        return "- none".to_owned();
    }

    section_summaries
        .iter()
        .enumerate()
        .map(|(idx, summary)| format!("- [{}] {}", idx, summary))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_node(node: &NarrativeNode) -> String {
    let transitions = render_list_lines(
        &node
            .transitions
            .iter()
            .map(render_transition)
            .collect::<Vec<_>>(),
    );
    render_sections(&[
        ("id", node.id.clone()),
        ("title", node.title.clone()),
        ("scene", node.scene.clone()),
        ("goal", node.goal.clone()),
        ("characters", node.characters.join(", ")),
        ("transitions", transitions),
    ])
}

fn render_optional_node_from_ref(node: &NarrativeNode) -> String {
    render_node(node)
}

fn render_keeper_node(node: &NarrativeNode) -> String {
    let transitions = if node.transitions.is_empty() {
        "- none".to_owned()
    } else {
        node.transitions
            .iter()
            .map(render_keeper_transition)
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    render_sections(&[
        ("id", node.id.clone()),
        ("title", node.title.clone()),
        ("scene", node.scene.clone()),
        ("goal", node.goal.clone()),
        ("characters", node.characters.join(", ")),
        ("candidate_transitions", transitions),
    ])
}

fn render_transition(transition: &Transition) -> String {
    match &transition.condition {
        Some(condition) => format!(
            "to node={} when {}",
            transition.to,
            render_condition(condition)
        ),
        None => format!("to node={} when always", transition.to),
    }
}

fn render_condition(condition: &Condition) -> String {
    let left = match condition.scope {
        ConditionScope::Global => format!("global.{}", condition.key),
        ConditionScope::Player => format!("player.{}", condition.key),
        ConditionScope::Character => format!(
            "character[{}].{}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key
        ),
    };

    format!(
        "{left} {} {}",
        render_condition_operator(&condition.op),
        compact_json(&condition.value)
    )
}

fn render_keeper_transition(transition: &Transition) -> String {
    match &transition.condition {
        Some(condition) => {
            format!(
                "to node={} when {}",
                transition.to,
                render_keeper_condition(condition)
            )
        }
        None => format!("to node={} when always", transition.to),
    }
}

fn render_keeper_condition(condition: &Condition) -> String {
    let left = match condition.scope {
        ConditionScope::Global => format!("global.{}", condition.key),
        ConditionScope::Player => format!("player.{}", condition.key),
        ConditionScope::Character => format!(
            "character[{}].{}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key
        ),
    };

    format!(
        "{left} {} {}",
        render_condition_operator(&condition.op),
        compact_json(&condition.value)
    )
}

fn render_condition_operator(operator: &ConditionOperator) -> &'static str {
    match operator {
        ConditionOperator::Eq => "==",
        ConditionOperator::Ne => "!=",
        ConditionOperator::Gt => ">",
        ConditionOperator::Gte => ">=",
        ConditionOperator::Lt => "<",
        ConditionOperator::Lte => "<=",
        ConditionOperator::Contains => "contains",
    }
}

fn render_actor_history(entries: &[ActorMemoryEntry]) -> String {
    if entries.is_empty() {
        return "- none".to_owned();
    }

    entries
        .iter()
        .map(|entry| {
            format!(
                "- [{}|{}|{}] {}",
                entry.speaker_id,
                entry.speaker_name,
                actor_memory_kind_label(&entry.kind),
                normalize_inline_text(&entry.text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_actor_world_state(world_state: &state::WorldState) -> String {
    render_world_state_sections(
        world_state.current_node(),
        world_state.active_characters(),
        &world_state.custom,
        None,
        Some(&world_state.character_state),
    )
}

fn render_director_world_state(world_state: &state::WorldState) -> String {
    render_world_state_sections(
        world_state.current_node(),
        world_state.active_characters(),
        &world_state.custom,
        Some(world_state.player_states()),
        Some(&world_state.character_state),
    )
}

fn render_observable_world_state(world_state: &state::WorldState) -> String {
    render_sections(&[
        ("current_node", world_state.current_node().to_owned()),
        (
            "active_characters",
            if world_state.active_characters().is_empty() {
                "none".to_owned()
            } else {
                world_state.active_characters().join(", ")
            },
        ),
        ("world_state", render_sorted_map(&world_state.custom)),
        (
            "player_state",
            render_sorted_map(world_state.player_states()),
        ),
        (
            "character_state",
            render_character_state(&world_state.character_state),
        ),
    ])
}

fn render_world_state_sections(
    current_node: &str,
    active_characters: &[String],
    custom: &HashMap<String, Value>,
    player_state: Option<&HashMap<String, Value>>,
    character_state: Option<&HashMap<String, HashMap<String, Value>>>,
) -> String {
    let mut sections = vec![
        ("current_node", current_node.to_owned()),
        (
            "active_characters",
            if active_characters.is_empty() {
                "none".to_owned()
            } else {
                active_characters.join(", ")
            },
        ),
        ("world_state", render_sorted_map(custom)),
    ];

    if let Some(player_state) = player_state {
        sections.push(("player_state", render_sorted_map(player_state)));
    }
    if let Some(character_state) = character_state {
        sections.push(("character_state", render_character_state(character_state)));
    }

    render_sections(&sections)
}

fn render_sorted_map(map: &HashMap<String, Value>) -> String {
    if map.is_empty() {
        return "none".to_owned();
    }

    map.iter()
        .map(|(key, value)| (key.clone(), value))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(key, value)| format!("{key}={}", compact_json(value)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_character_state(map: &HashMap<String, HashMap<String, Value>>) -> String {
    if map.is_empty() {
        return "none".to_owned();
    }

    map.iter()
        .map(|(character, state)| (character.clone(), state))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(character, state)| format!("{character}: {}", render_sorted_map(state)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_reply_history(history: &[ReplyHistoryMessage]) -> String {
    if history.is_empty() {
        return "- none".to_owned();
    }

    history
        .iter()
        .map(|message| {
            format!(
                "- [turn:{}|{}|{}|{}] {}",
                message.turn_index,
                message.speaker_id,
                message.speaker_name,
                compact_json(&message.kind),
                normalize_inline_text(&message.text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_keeper_node_change(
    previous_node: Option<&NarrativeNode>,
    current_node: &NarrativeNode,
) -> String {
    let Some(previous_node) = previous_node else {
        return "null".to_owned();
    };

    let transitioned = previous_node.id != current_node.id;
    let matched_transition_lines = previous_node
        .transitions
        .iter()
        .filter(|transition| transition.to == current_node.id)
        .map(render_keeper_progression_hint_line)
        .collect::<Vec<_>>();

    render_sections(&[
        ("transitioned", compact_json(&transitioned)),
        ("from", previous_node.id.clone()),
        ("to", current_node.id.clone()),
        (
            "matched_transition_hints",
            if matched_transition_lines.is_empty() {
                "- none".to_owned()
            } else {
                matched_transition_lines
                    .into_iter()
                    .map(|line| format!("- {line}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
        ),
    ])
}

fn render_keeper_progression_hints(node: &NarrativeNode) -> String {
    if node.transitions.is_empty() {
        return "- none".to_owned();
    }

    node.transitions
        .iter()
        .map(render_keeper_progression_hint_line)
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_keeper_progression_hint_line(transition: &Transition) -> String {
    match &transition.condition {
        Some(condition) => {
            let (scope, key, character) = keeper_condition_tracking_hint(condition);
            match character {
                Some(character) => format!(
                    "target_node={} | condition={} | likely_state_scope={} | tracked_key={} | tracked_character={}",
                    transition.to,
                    render_keeper_condition(condition),
                    scope,
                    key,
                    character
                ),
                None => format!(
                    "target_node={} | condition={} | likely_state_scope={} | tracked_key={}",
                    transition.to,
                    render_keeper_condition(condition),
                    scope,
                    key
                ),
            }
        }
        None => format!(
            "target_node={} | condition=always | likely_state_scope=none | tracked_key=none",
            transition.to
        ),
    }
}

fn keeper_condition_tracking_hint(condition: &Condition) -> (&str, &str, Option<&str>) {
    match condition.scope {
        ConditionScope::Global => ("global", condition.key.as_str(), None),
        ConditionScope::Player => ("player", condition.key.as_str(), None),
        ConditionScope::Character => (
            "character",
            condition.key.as_str(),
            condition.character.as_deref(),
        ),
    }
}

fn render_actor_purpose(purpose: PromptPreviewActorPurpose) -> String {
    match purpose {
        PromptPreviewActorPurpose::AdvanceGoal => "\"AdvanceGoal\"".to_owned(),
        PromptPreviewActorPurpose::ReactToPlayer => "\"ReactToPlayer\"".to_owned(),
        PromptPreviewActorPurpose::CommentOnScene => "\"CommentOnScene\"".to_owned(),
    }
}

fn render_narrator_purpose(purpose: PromptPreviewNarratorPurpose) -> String {
    match purpose {
        PromptPreviewNarratorPurpose::DescribeTransition => "\"DescribeTransition\"".to_owned(),
        PromptPreviewNarratorPurpose::DescribeScene => "\"DescribeScene\"".to_owned(),
        PromptPreviewNarratorPurpose::DescribeResult => "\"DescribeResult\"".to_owned(),
    }
}

fn render_keeper_phase(phase: PromptPreviewKeeperPhase) -> String {
    match phase {
        PromptPreviewKeeperPhase::AfterPlayerInput => "\"AfterPlayerInput\"".to_owned(),
        PromptPreviewKeeperPhase::AfterTurnOutputs => "\"AfterTurnOutputs\"".to_owned(),
    }
}

fn render_sections(sections: &[(&str, String)]) -> String {
    sections
        .iter()
        .map(|(title, body)| format!("{title}:\n{}", body.trim_end()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_list_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return "- none".to_owned();
    }

    lines
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn compact_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}

fn actor_memory_kind_label(kind: &ActorMemoryKind) -> &'static str {
    match kind {
        ActorMemoryKind::PlayerInput => "player_input",
        ActorMemoryKind::Narration => "narration",
        ActorMemoryKind::Dialogue => "dialogue",
        ActorMemoryKind::Thought => "thought",
        ActorMemoryKind::Action => "action",
    }
}

#[derive(Debug, Clone, Serialize)]
struct RecentSectionDetailNodePreview {
    id: String,
    title: String,
    scene_summary: String,
    goal: String,
    characters: Vec<String>,
    transition_targets: Vec<String>,
    on_enter_update_keys: Vec<String>,
}

fn compact_recent_section_node(node: &NarrativeNode) -> RecentSectionDetailNodePreview {
    RecentSectionDetailNodePreview {
        id: node.id.clone(),
        title: node.title.clone(),
        scene_summary: truncate_text(&node.scene, 140),
        goal: truncate_text(&node.goal, 100),
        characters: node.characters.clone(),
        transition_targets: node
            .transitions
            .iter()
            .map(|transition| transition.to.clone())
            .collect(),
        on_enter_update_keys: node
            .on_enter_updates
            .iter()
            .map(|update| match update {
                state::StateOp::SetCurrentNode { node_id } => format!("current_node:{node_id}"),
                state::StateOp::SetActiveCharacters { .. } => "active_characters".to_owned(),
                state::StateOp::AddActiveCharacter { character } => format!("active+:{character}"),
                state::StateOp::RemoveActiveCharacter { character } => {
                    format!("active-:{character}")
                }
                state::StateOp::SetState { key, .. } => format!("world:{key}"),
                state::StateOp::RemoveState { key } => format!("world:{key}"),
                state::StateOp::SetPlayerState { key, .. } => format!("player:{key}"),
                state::StateOp::RemovePlayerState { key } => format!("player:{key}"),
                state::StateOp::SetCharacterState { character, key, .. } => {
                    format!("character:{character}:{key}")
                }
                state::StateOp::RemoveCharacterState { character, key } => {
                    format!("character:{character}:{key}")
                }
            })
            .collect(),
    }
}
