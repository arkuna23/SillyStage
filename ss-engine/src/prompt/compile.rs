use std::collections::HashSet;

use agents::{ArchitectPromptProfiles, PromptModule, PromptModuleEntry, PromptProfile};
use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PromptEntryKind,
    PromptMessageRole, PromptModuleId,
};

use super::contracts::{
    ARCHITECT_DRAFT_CONTINUE_OUTPUT_CONTRACT, ARCHITECT_DRAFT_INIT_OUTPUT_CONTRACT,
    ARCHITECT_GRAPH_OUTPUT_CONTRACT,
};
use super::defaults::fallback_display_name;
use super::normalize::normalize_agent_preset_config;
use super::types::{
    ArchitectPromptMode, CompiledPromptPreviewEntry, CompiledPromptPreviewEntryValue,
    CompiledPromptPreviewModule, CompiledPromptPreviewProfile, PromptAgentKind, PromptConfigError,
    PromptPreviewEntrySource,
};

const PLANNER_ALLOWED_CONTEXT_KEYS: [&str; 4] = [
    "story_concept",
    "lorebook_base",
    "available_characters",
    "lorebook_matched",
];
const DIRECTOR_ALLOWED_CONTEXT_KEYS: [&str; 9] = [
    "lorebook_base",
    "player",
    "current_cast",
    "current_node",
    "player_state_schema",
    "transitioned_this_turn",
    "lorebook_matched",
    "world_state",
    "shared_history",
];
const ACTOR_ALLOWED_CONTEXT_KEYS: [&str; 9] = [
    "lorebook_base",
    "player",
    "current_cast",
    "current_node",
    "lorebook_matched",
    "world_state",
    "shared_history",
    "private_memory",
    "actor_purpose",
];
const NARRATOR_ALLOWED_CONTEXT_KEYS: [&str; 11] = [
    "lorebook_base",
    "player",
    "previous_node",
    "current_node",
    "previous_cast",
    "current_cast",
    "player_state_schema",
    "narrator_purpose",
    "lorebook_matched",
    "world_state",
    "shared_history",
];
const KEEPER_ALLOWED_CONTEXT_KEYS: [&str; 14] = [
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
    "lorebook_matched",
    "world_state",
    "player_input",
    "completed_beats",
];
const REPLYER_ALLOWED_CONTEXT_KEYS: [&str; 9] = [
    "lorebook_base",
    "player",
    "reply_limit",
    "current_cast",
    "current_node",
    "player_state_schema",
    "world_state",
    "session_history",
    "lorebook_matched",
];
const ARCHITECT_GRAPH_ALLOWED_CONTEXT_KEYS: [&str; 7] = [
    "story_concept",
    "lorebook_base",
    "planned_story",
    "available_characters",
    "world_state_schema_seed",
    "player_state_schema_seed",
    "lorebook_matched",
];
const ARCHITECT_DRAFT_INIT_ALLOWED_CONTEXT_KEYS: [&str; 13] = [
    "story_concept",
    "lorebook_base",
    "planned_story",
    "available_characters",
    "world_state_schema_seed",
    "player_state_schema_seed",
    "current_section",
    "section_index",
    "total_sections",
    "target_node_count",
    "graph_summary",
    "recent_section_detail",
    "lorebook_matched",
];
const ARCHITECT_DRAFT_CONTINUE_ALLOWED_CONTEXT_KEYS: [&str; 13] = [
    "story_concept",
    "lorebook_base",
    "available_characters",
    "world_state_schema",
    "player_state_schema",
    "section_summaries",
    "current_section",
    "section_index",
    "total_sections",
    "target_node_count",
    "graph_summary",
    "recent_section_detail",
    "lorebook_matched",
];
const ARCHITECT_GRAPH_OUTPUT_EXTRA: [SyntheticEntryTemplate; 1] = [SyntheticEntryTemplate {
    entry_id: "__injected_architect_graph_output_contract",
    display_name: "Architect Graph Output Contract",
    text: ARCHITECT_GRAPH_OUTPUT_CONTRACT,
}];
const ARCHITECT_DRAFT_INIT_TASK_EXTRA: [SyntheticEntryTemplate; 1] = [SyntheticEntryTemplate {
    entry_id: "__injected_architect_draft_init_task",
    display_name: "Architect Draft Init Task",
    text: "Generate only the first draft chunk for the current outline section.",
}];
const ARCHITECT_DRAFT_INIT_OUTPUT_EXTRA: [SyntheticEntryTemplate; 1] = [SyntheticEntryTemplate {
    entry_id: "__injected_architect_draft_init_output_contract",
    display_name: "Architect Draft Init Output Contract",
    text: ARCHITECT_DRAFT_INIT_OUTPUT_CONTRACT,
}];
const ARCHITECT_DRAFT_CONTINUE_TASK_EXTRA: [SyntheticEntryTemplate; 1] = [SyntheticEntryTemplate {
    entry_id: "__injected_architect_draft_continue_task",
    display_name: "Architect Draft Continue Task",
    text: "Continue the draft for the current section without resetting earlier results.",
}];
const ARCHITECT_DRAFT_CONTINUE_OUTPUT_EXTRA: [SyntheticEntryTemplate; 1] =
    [SyntheticEntryTemplate {
        entry_id: "__injected_architect_draft_continue_output_contract",
        display_name: "Architect Draft Continue Output Contract",
        text: ARCHITECT_DRAFT_CONTINUE_OUTPUT_CONTRACT,
    }];

#[derive(Clone, Copy)]
struct SyntheticEntryTemplate {
    entry_id: &'static str,
    display_name: &'static str,
    text: &'static str,
}

struct ProfileCompilationSpec<'a> {
    extra_role_entries: &'a [SyntheticEntryTemplate],
    extra_task_entries: &'a [SyntheticEntryTemplate],
    extra_output_entries: &'a [SyntheticEntryTemplate],
    allowed_context_keys: &'a [&'a str],
}

pub(crate) struct CompiledPromptModulePreview {
    pub(crate) message_role: PromptMessageRole,
    pub(crate) module: Option<CompiledPromptPreviewModule>,
}

pub fn compile_prompt_profile(
    agent: PromptAgentKind,
    config: &AgentPresetConfig,
) -> Result<PromptProfile, PromptConfigError> {
    let normalized = normalize_agent_preset_config(agent, config.clone())?;
    Ok(compile_profile_from_spec(
        &normalized,
        spec_for_agent(agent),
    ))
}

pub fn compile_architect_prompt_profiles(
    config: &AgentPresetConfig,
) -> Result<ArchitectPromptProfiles, PromptConfigError> {
    let normalized = normalize_agent_preset_config(PromptAgentKind::Architect, config.clone())?;

    Ok(ArchitectPromptProfiles {
        graph: compile_profile_from_spec(
            &normalized,
            spec_for_architect_mode(ArchitectPromptMode::Graph),
        ),
        draft_init: compile_profile_from_spec(
            &normalized,
            spec_for_architect_mode(ArchitectPromptMode::DraftInit),
        ),
        draft_continue: compile_profile_from_spec(
            &normalized,
            spec_for_architect_mode(ArchitectPromptMode::DraftContinue),
        ),
        repair_system_prompt: architect_repair_system_prompt().to_owned(),
    })
}

pub(crate) fn compile_prompt_preview_profile(
    agent: PromptAgentKind,
    config: &AgentPresetConfig,
) -> Result<CompiledPromptPreviewProfile, PromptConfigError> {
    let normalized = normalize_agent_preset_config(agent, config.clone())?;
    Ok(compile_preview_profile_from_spec(
        &normalized,
        spec_for_agent(agent),
    ))
}

pub(crate) fn compile_architect_prompt_preview_profile(
    config: &AgentPresetConfig,
    mode: ArchitectPromptMode,
) -> Result<CompiledPromptPreviewProfile, PromptConfigError> {
    let normalized = normalize_agent_preset_config(PromptAgentKind::Architect, config.clone())?;
    Ok(compile_preview_profile_from_spec(
        &normalized,
        spec_for_architect_mode(mode),
    ))
}

pub(crate) fn compile_prompt_module(
    agent: PromptAgentKind,
    config: &AgentPresetConfig,
    module_id: &PromptModuleId,
) -> Result<Option<CompiledPromptModulePreview>, PromptConfigError> {
    let normalized = normalize_agent_preset_config(agent, config.clone())?;
    Ok(compile_module_preview(
        &normalized,
        module_id,
        spec_for_agent(agent),
    ))
}

pub(crate) fn compile_architect_prompt_module(
    config: &AgentPresetConfig,
    mode: ArchitectPromptMode,
    module_id: &PromptModuleId,
) -> Result<Option<CompiledPromptModulePreview>, PromptConfigError> {
    let normalized = normalize_agent_preset_config(PromptAgentKind::Architect, config.clone())?;
    Ok(compile_module_preview(
        &normalized,
        module_id,
        spec_for_architect_mode(mode),
    ))
}

fn spec_for_agent(agent: PromptAgentKind) -> ProfileCompilationSpec<'static> {
    match agent {
        PromptAgentKind::Planner => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &[],
            allowed_context_keys: &PLANNER_ALLOWED_CONTEXT_KEYS,
        },
        PromptAgentKind::Director => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &[],
            allowed_context_keys: &DIRECTOR_ALLOWED_CONTEXT_KEYS,
        },
        PromptAgentKind::Actor => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &[],
            allowed_context_keys: &ACTOR_ALLOWED_CONTEXT_KEYS,
        },
        PromptAgentKind::Narrator => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &[],
            allowed_context_keys: &NARRATOR_ALLOWED_CONTEXT_KEYS,
        },
        PromptAgentKind::Keeper => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &[],
            allowed_context_keys: &KEEPER_ALLOWED_CONTEXT_KEYS,
        },
        PromptAgentKind::Replyer => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &[],
            allowed_context_keys: &REPLYER_ALLOWED_CONTEXT_KEYS,
        },
        PromptAgentKind::Architect => spec_for_architect_mode(ArchitectPromptMode::Graph),
    }
}

fn spec_for_architect_mode(mode: ArchitectPromptMode) -> ProfileCompilationSpec<'static> {
    match mode {
        ArchitectPromptMode::Graph => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &[],
            extra_output_entries: &ARCHITECT_GRAPH_OUTPUT_EXTRA,
            allowed_context_keys: &ARCHITECT_GRAPH_ALLOWED_CONTEXT_KEYS,
        },
        ArchitectPromptMode::DraftInit => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &ARCHITECT_DRAFT_INIT_TASK_EXTRA,
            extra_output_entries: &ARCHITECT_DRAFT_INIT_OUTPUT_EXTRA,
            allowed_context_keys: &ARCHITECT_DRAFT_INIT_ALLOWED_CONTEXT_KEYS,
        },
        ArchitectPromptMode::DraftContinue => ProfileCompilationSpec {
            extra_role_entries: &[],
            extra_task_entries: &ARCHITECT_DRAFT_CONTINUE_TASK_EXTRA,
            extra_output_entries: &ARCHITECT_DRAFT_CONTINUE_OUTPUT_EXTRA,
            allowed_context_keys: &ARCHITECT_DRAFT_CONTINUE_ALLOWED_CONTEXT_KEYS,
        },
    }
}

fn compile_profile_from_spec(
    config: &AgentPresetConfig,
    spec: ProfileCompilationSpec<'_>,
) -> PromptProfile {
    compile_profile(
        config,
        spec.extra_role_entries,
        spec.extra_task_entries,
        spec.extra_output_entries,
        spec.allowed_context_keys,
    )
}

fn compile_preview_profile_from_spec(
    config: &AgentPresetConfig,
    spec: ProfileCompilationSpec<'_>,
) -> CompiledPromptPreviewProfile {
    compile_preview_profile(
        config,
        spec.extra_role_entries,
        spec.extra_task_entries,
        spec.extra_output_entries,
        spec.allowed_context_keys,
    )
}

fn compile_module_preview(
    config: &AgentPresetConfig,
    module_id: &PromptModuleId,
    spec: ProfileCompilationSpec<'_>,
) -> Option<CompiledPromptModulePreview> {
    let allowed = spec
        .allowed_context_keys
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let module = config
        .modules
        .iter()
        .find(|module| &module.module_id == module_id)?;
    Some(CompiledPromptModulePreview {
        message_role: module.message_role,
        module: compile_preview_module(
            module,
            &allowed,
            extra_entries_for_module(
                &module.module_id,
                spec.extra_role_entries,
                spec.extra_task_entries,
                spec.extra_output_entries,
            ),
        ),
    })
}

fn compile_profile(
    config: &AgentPresetConfig,
    extra_role_entries: &[SyntheticEntryTemplate],
    extra_task_entries: &[SyntheticEntryTemplate],
    extra_output_entries: &[SyntheticEntryTemplate],
    allowed_context_keys: &[&str],
) -> PromptProfile {
    let allowed = allowed_context_keys.iter().copied().collect::<HashSet<_>>();
    let mut system_modules = Vec::new();
    let mut user_modules = Vec::new();

    for module in &config.modules {
        let Some(compiled_module) = compile_module(
            module,
            &allowed,
            extra_entries_for_module(
                &module.module_id,
                extra_role_entries,
                extra_task_entries,
                extra_output_entries,
            ),
        ) else {
            continue;
        };

        match module.message_role {
            PromptMessageRole::System => system_modules.push(compiled_module),
            PromptMessageRole::User => user_modules.push(compiled_module),
        }
    }

    if system_modules.iter().any(module_has_context_refs) {
        PromptProfile {
            system_prompt: String::new(),
            system_modules,
            user_modules,
        }
    } else {
        let system_prompt = system_modules
            .iter()
            .map(render_static_module)
            .filter(|section| !section.is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");

        PromptProfile {
            system_prompt,
            system_modules: Vec::new(),
            user_modules,
        }
    }
}

fn compile_preview_profile(
    config: &AgentPresetConfig,
    extra_role_entries: &[SyntheticEntryTemplate],
    extra_task_entries: &[SyntheticEntryTemplate],
    extra_output_entries: &[SyntheticEntryTemplate],
    allowed_context_keys: &[&str],
) -> CompiledPromptPreviewProfile {
    let allowed = allowed_context_keys.iter().copied().collect::<HashSet<_>>();
    let mut system_modules = Vec::new();
    let mut user_modules = Vec::new();

    for module in &config.modules {
        let Some(compiled_module) = compile_preview_module(
            module,
            &allowed,
            extra_entries_for_module(
                &module.module_id,
                extra_role_entries,
                extra_task_entries,
                extra_output_entries,
            ),
        ) else {
            continue;
        };

        match module.message_role {
            PromptMessageRole::System => system_modules.push(compiled_module),
            PromptMessageRole::User => user_modules.push(compiled_module),
        }
    }

    CompiledPromptPreviewProfile {
        system_modules,
        user_modules,
    }
}

fn compile_module(
    module: &AgentPromptModuleConfig,
    allowed_context_keys: &HashSet<&str>,
    extra_entries: &[SyntheticEntryTemplate],
) -> Option<PromptModule> {
    let mut entries = module
        .entries
        .iter()
        .filter(|entry| entry.enabled)
        .filter_map(|entry| compile_entry(entry, allowed_context_keys))
        .collect::<Vec<_>>();
    entries.extend(
        extra_entries
            .iter()
            .map(|entry| entry.text.trim())
            .filter(|text| !text.is_empty())
            .map(|text| PromptModuleEntry::Text(text.to_owned())),
    );

    if entries.is_empty() {
        return None;
    }

    Some(PromptModule {
        title: fallback_display_name(&module.display_name, module.module_id.as_str()),
        entries,
    })
}

fn compile_preview_module(
    module: &AgentPromptModuleConfig,
    allowed_context_keys: &HashSet<&str>,
    extra_entries: &[SyntheticEntryTemplate],
) -> Option<CompiledPromptPreviewModule> {
    let mut entries = module
        .entries
        .iter()
        .filter(|entry| entry.enabled)
        .filter_map(|entry| compile_preview_entry(entry, allowed_context_keys))
        .collect::<Vec<_>>();

    let next_order_seed = entries.iter().map(|entry| entry.order).max().unwrap_or(0);
    entries.extend(
        extra_entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                let text = entry.text.trim();
                (!text.is_empty()).then(|| CompiledPromptPreviewEntry {
                    entry_id: entry.entry_id.to_owned(),
                    display_name: entry.display_name.to_owned(),
                    kind: PromptEntryKind::BuiltInText,
                    order: next_order_seed + ((index as i32) + 1) * 10,
                    source: PromptPreviewEntrySource::Synthetic,
                    value: CompiledPromptPreviewEntryValue::Text(text.to_owned()),
                })
            }),
    );

    if entries.is_empty() {
        return None;
    }

    Some(CompiledPromptPreviewModule {
        module_id: module.module_id.clone(),
        display_name: fallback_display_name(&module.display_name, module.module_id.as_str()),
        order: module.order,
        entries,
    })
}

fn compile_entry(
    entry: &AgentPromptModuleEntryConfig,
    allowed_context_keys: &HashSet<&str>,
) -> Option<PromptModuleEntry> {
    match entry.kind {
        PromptEntryKind::BuiltInText | PromptEntryKind::CustomText => entry
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(|text| PromptModuleEntry::Text(text.to_owned())),
        PromptEntryKind::BuiltInContextRef => entry.context_key.as_deref().and_then(|key| {
            allowed_context_keys
                .contains(key)
                .then(|| PromptModuleEntry::ContextRef(key.to_owned()))
        }),
    }
}

fn compile_preview_entry(
    entry: &AgentPromptModuleEntryConfig,
    allowed_context_keys: &HashSet<&str>,
) -> Option<CompiledPromptPreviewEntry> {
    let value = match entry.kind {
        PromptEntryKind::BuiltInText | PromptEntryKind::CustomText => entry
            .text
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(|text| CompiledPromptPreviewEntryValue::Text(text.to_owned())),
        PromptEntryKind::BuiltInContextRef => entry.context_key.as_deref().and_then(|key| {
            allowed_context_keys
                .contains(key)
                .then(|| CompiledPromptPreviewEntryValue::ContextRef(key.to_owned()))
        }),
    }?;

    Some(CompiledPromptPreviewEntry {
        entry_id: entry.entry_id.clone(),
        display_name: fallback_display_name(&entry.display_name, &entry.entry_id),
        kind: entry.kind,
        order: entry.order,
        source: PromptPreviewEntrySource::Preset,
        value,
    })
}

fn extra_entries_for_module<'a>(
    module_id: &PromptModuleId,
    extra_role_entries: &'a [SyntheticEntryTemplate],
    extra_task_entries: &'a [SyntheticEntryTemplate],
    extra_output_entries: &'a [SyntheticEntryTemplate],
) -> &'a [SyntheticEntryTemplate] {
    match module_id {
        PromptModuleId::Role => extra_role_entries,
        PromptModuleId::Task => extra_task_entries,
        PromptModuleId::Output => extra_output_entries,
        PromptModuleId::StaticContext
        | PromptModuleId::DynamicContext
        | PromptModuleId::Custom(_) => &[],
    }
}

fn module_has_context_refs(module: &PromptModule) -> bool {
    module
        .entries
        .iter()
        .any(|entry| matches!(entry, PromptModuleEntry::ContextRef(_)))
}

fn render_static_module(module: &PromptModule) -> String {
    let body = module
        .entries
        .iter()
        .filter_map(|entry| match entry {
            PromptModuleEntry::Text(text) => {
                let body = text.trim();
                (!body.is_empty()).then(|| body.to_owned())
            }
            PromptModuleEntry::ContextRef(_) => None,
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    if body.is_empty() {
        String::new()
    } else {
        format!("{}:\n{}", module.title.trim(), body)
    }
}

fn architect_repair_system_prompt() -> &'static str {
    "ROLE:\nYou repair architect outputs that failed JSON decoding or validation.\n\nTASK:\nUse the raw failed output and the validation error to produce a corrected JSON payload that preserves as much intended content as possible.\nIf a schema field has enum_values, keep enum_values only for scalar types (bool, int, float, string).\nIf a schema field has enum_values, omit default or make default exactly match one enum_values item.\nUse only these StateOp type names: SetCurrentNode, SetActiveCharacters, AddActiveCharacter, RemoveActiveCharacter, SetState, RemoveState, SetPlayerState, RemovePlayerState, SetCharacterState, RemoveCharacterState.\nIf the raw output uses aliases such as SetGlobalState or SetWorldState, rewrite them to SetState.\nIf the raw output uses aliases such as RemoveGlobalState or RemoveWorldState, rewrite them to RemoveState.\nKeep every returned node id unique.\nFor draft_continue repairs, do not return any node whose id already exists in GRAPH_SUMMARY. Existing nodes may only be referenced in transition_patches or transition targets.\nFor draft_init repairs, transition and transition_patches targets must use returned node ids only.\nFor draft_continue repairs, transition and transition_patches targets must use GRAPH_SUMMARY node ids or returned node ids only.\nRemove links to future chunk nodes; later chunks can add them via transition_patches.\n\nOUTPUT:\nReturn valid JSON only. Do not wrap it in markdown."
}
