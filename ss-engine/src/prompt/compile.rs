use std::collections::{HashMap, HashSet};

use agents::{ArchitectPromptProfiles, PromptEntry, PromptEntryValue, PromptProfile};
use store::{AgentPresetConfig, AgentPromptModuleEntryConfig, PromptEntryKind, PromptModuleId};

use super::contracts::{
    ARCHITECT_DRAFT_CONTINUE_OUTPUT_CONTRACT, ARCHITECT_DRAFT_INIT_OUTPUT_CONTRACT,
    ARCHITECT_GRAPH_OUTPUT_CONTRACT,
};
use super::defaults::fallback_display_name;
use super::normalize::normalize_agent_preset_config;
use super::types::{PromptAgentKind, PromptConfigError};

pub fn compile_prompt_profile(
    agent: PromptAgentKind,
    config: &AgentPresetConfig,
) -> Result<PromptProfile, PromptConfigError> {
    let normalized = normalize_agent_preset_config(agent, config.clone())?;
    Ok(match agent {
        PromptAgentKind::Planner => compile_profile(
            &normalized,
            &[],
            &[],
            &[],
            &["story_concept", "lorebook_base", "available_characters"],
            &["lorebook_matched"],
        ),
        PromptAgentKind::Director => compile_profile(
            &normalized,
            &[],
            &[],
            &[],
            &[
                "lorebook_base",
                "player",
                "current_cast",
                "current_node",
                "player_state_schema",
                "transitioned_this_turn",
            ],
            &["lorebook_matched", "world_state", "shared_history"],
        ),
        PromptAgentKind::Actor => compile_profile(
            &normalized,
            &[],
            &[],
            &[],
            &["lorebook_base", "player", "current_cast", "current_node"],
            &[
                "lorebook_matched",
                "world_state",
                "shared_history",
                "private_memory",
                "actor_purpose",
            ],
        ),
        PromptAgentKind::Narrator => compile_profile(
            &normalized,
            &[],
            &[],
            &[],
            &[
                "lorebook_base",
                "player",
                "previous_node",
                "current_node",
                "previous_cast",
                "current_cast",
                "player_state_schema",
                "narrator_purpose",
            ],
            &["lorebook_matched", "world_state", "shared_history"],
        ),
        PromptAgentKind::Keeper => compile_profile(
            &normalized,
            &[],
            &[],
            &[],
            &[
                "lorebook_base",
                "player",
                "previous_node",
                "current_node",
                "previous_cast",
                "current_cast",
                "player_state_schema",
                "keeper_phase",
                "node_change",
                "progression_hints",
            ],
            &[
                "lorebook_matched",
                "world_state",
                "player_input",
                "completed_beats",
            ],
        ),
        PromptAgentKind::Replyer => compile_profile(
            &normalized,
            &[],
            &[],
            &[],
            &[
                "lorebook_base",
                "player",
                "reply_limit",
                "current_cast",
                "current_node",
                "player_state_schema",
            ],
            &["world_state", "session_history", "lorebook_matched"],
        ),
        PromptAgentKind::Architect => compile_profile(
            &normalized,
            &[],
            &[],
            &[ARCHITECT_GRAPH_OUTPUT_CONTRACT],
            &[
                "story_concept",
                "lorebook_base",
                "planned_story",
                "available_characters",
                "world_state_schema_seed",
                "player_state_schema_seed",
            ],
            &["lorebook_matched"],
        ),
    })
}

pub fn compile_architect_prompt_profiles(
    config: &AgentPresetConfig,
) -> Result<ArchitectPromptProfiles, PromptConfigError> {
    let normalized = normalize_agent_preset_config(PromptAgentKind::Architect, config.clone())?;

    Ok(ArchitectPromptProfiles {
        graph: compile_profile(
            &normalized,
            &[],
            &["Generate a full story graph from the concept and the optional planned story."],
            &[ARCHITECT_GRAPH_OUTPUT_CONTRACT],
            &[
                "story_concept",
                "lorebook_base",
                "planned_story",
                "available_characters",
                "world_state_schema_seed",
                "player_state_schema_seed",
            ],
            &["lorebook_matched"],
        ),
        draft_init: compile_profile(
            &normalized,
            &[],
            &["Generate only the first draft chunk for the current outline section."],
            &[ARCHITECT_DRAFT_INIT_OUTPUT_CONTRACT],
            &[
                "story_concept",
                "lorebook_base",
                "planned_story",
                "available_characters",
                "world_state_schema_seed",
                "player_state_schema_seed",
            ],
            &[
                "current_section",
                "section_index",
                "total_sections",
                "target_node_count",
                "graph_summary",
                "recent_section_detail",
                "lorebook_matched",
            ],
        ),
        draft_continue: compile_profile(
            &normalized,
            &[],
            &["Continue the draft for the current section without resetting earlier results."],
            &[ARCHITECT_DRAFT_CONTINUE_OUTPUT_CONTRACT],
            &[
                "story_concept",
                "lorebook_base",
                "available_characters",
                "world_state_schema",
                "player_state_schema",
                "section_summaries",
            ],
            &[
                "current_section",
                "section_index",
                "total_sections",
                "target_node_count",
                "graph_summary",
                "recent_section_detail",
                "lorebook_matched",
            ],
        ),
        repair_system_prompt: architect_repair_system_prompt().to_owned(),
    })
}

fn compile_profile(
    config: &AgentPresetConfig,
    extra_role_texts: &[&str],
    extra_task_texts: &[&str],
    extra_output_texts: &[&str],
    allowed_static_keys: &[&str],
    allowed_dynamic_keys: &[&str],
) -> PromptProfile {
    let modules = module_map(config);
    let system_prompt = render_system_prompt(
        modules.get(&PromptModuleId::Role),
        modules.get(&PromptModuleId::Task),
        modules.get(&PromptModuleId::Output),
        extra_role_texts,
        extra_task_texts,
        extra_output_texts,
    );

    PromptProfile {
        system_prompt,
        stable_entries: compile_user_entries(
            modules.get(&PromptModuleId::StaticContext),
            allowed_static_keys,
        ),
        dynamic_entries: compile_user_entries(
            modules.get(&PromptModuleId::DynamicContext),
            allowed_dynamic_keys,
        ),
    }
}

fn render_system_prompt(
    role_entries: Option<&Vec<AgentPromptModuleEntryConfig>>,
    task_entries: Option<&Vec<AgentPromptModuleEntryConfig>>,
    output_entries: Option<&Vec<AgentPromptModuleEntryConfig>>,
    extra_role_texts: &[&str],
    extra_task_texts: &[&str],
    extra_output_texts: &[&str],
) -> String {
    let mut sections = Vec::new();

    let role_body = render_system_module_entries(role_entries, extra_role_texts);
    if !role_body.is_empty() {
        sections.push(("ROLE", role_body));
    }

    let task_body = render_system_module_entries(task_entries, extra_task_texts);
    if !task_body.is_empty() {
        sections.push(("TASK", task_body));
    }

    let output_body = render_system_module_entries(output_entries, extra_output_texts);
    if !output_body.is_empty() {
        sections.push(("OUTPUT", output_body));
    }

    sections
        .into_iter()
        .map(|(title, body)| format!("{title}:\n{body}"))
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_system_module_entries(
    entries: Option<&Vec<AgentPromptModuleEntryConfig>>,
    extra_texts: &[&str],
) -> String {
    let mut parts = entries
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter(|entry| entry.enabled)
        .filter_map(|entry| match entry.kind {
            PromptEntryKind::BuiltInText | PromptEntryKind::CustomText => {
                entry.text.as_deref().map(|text| {
                    format!(
                        "[{}] {}\n{}",
                        entry.entry_id,
                        fallback_display_name(&entry.display_name, &entry.entry_id),
                        text.trim()
                    )
                })
            }
            PromptEntryKind::BuiltInContextRef => None,
        })
        .collect::<Vec<_>>();

    parts.extend(
        extra_texts
            .iter()
            .map(|text| text.trim().to_owned())
            .filter(|text| !text.is_empty()),
    );

    parts.join("\n\n")
}

fn compile_user_entries(
    entries: Option<&Vec<AgentPromptModuleEntryConfig>>,
    allowed_keys: &[&str],
) -> Vec<PromptEntry> {
    let allowed = allowed_keys.iter().copied().collect::<HashSet<_>>();

    entries
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter(|entry| entry.enabled)
        .filter_map(|entry| match entry.kind {
            PromptEntryKind::BuiltInText | PromptEntryKind::CustomText => {
                entry.text.as_ref().map(|text| PromptEntry {
                    entry_id: entry.entry_id.clone(),
                    title: fallback_display_name(&entry.display_name, &entry.entry_id),
                    value: PromptEntryValue::Text(text.clone()),
                })
            }
            PromptEntryKind::BuiltInContextRef => entry.context_key.as_deref().and_then(|key| {
                if allowed.contains(key) {
                    Some(PromptEntry {
                        entry_id: entry.entry_id.clone(),
                        title: key.to_ascii_uppercase(),
                        value: PromptEntryValue::ContextRef(key.to_owned()),
                    })
                } else {
                    None
                }
            }),
        })
        .collect()
}

fn module_map(
    config: &AgentPresetConfig,
) -> HashMap<PromptModuleId, Vec<AgentPromptModuleEntryConfig>> {
    config
        .modules
        .iter()
        .map(|module| (module.module_id, module.entries.clone()))
        .collect()
}

fn architect_repair_system_prompt() -> &'static str {
    "ROLE:\nYou repair architect outputs that failed JSON decoding or validation.\n\nTASK:\nUse the raw failed output and the validation error to produce a corrected JSON payload that preserves as much intended content as possible.\nIf a schema field has enum_values, keep enum_values only for scalar types (bool, int, float, string).\nIf a schema field has enum_values, omit default or make default exactly match one enum_values item.\nUse only these StateOp type names: SetCurrentNode, SetActiveCharacters, AddActiveCharacter, RemoveActiveCharacter, SetState, RemoveState, SetPlayerState, RemovePlayerState, SetCharacterState, RemoveCharacterState.\nIf the raw output uses aliases such as SetGlobalState or SetWorldState, rewrite them to SetState.\nIf the raw output uses aliases such as RemoveGlobalState or RemoveWorldState, rewrite them to RemoveState.\nKeep every returned node id unique.\nFor draft_continue repairs, do not return any node whose id already exists in GRAPH_SUMMARY. Existing nodes may only be referenced in transition_patches or transition targets.\nFor draft_init repairs, transition and transition_patches targets must use returned node ids only.\nFor draft_continue repairs, transition and transition_patches targets must use GRAPH_SUMMARY node ids or returned node ids only.\nRemove links to future chunk nodes; later chunks can add them via transition_patches.\n\nOUTPUT:\nReturn valid JSON only. Do not wrap it in markdown."
}
