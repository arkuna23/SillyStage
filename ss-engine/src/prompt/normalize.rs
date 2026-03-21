use std::collections::{HashMap, HashSet};

use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PromptEntryKind,
    PromptMessageRole, PromptModuleId,
};

use super::defaults::{default_agent_preset_config, fallback_display_name};
use super::types::{PromptAgentKind, PromptConfigError};

pub fn normalize_agent_preset_config(
    agent: PromptAgentKind,
    incoming: AgentPresetConfig,
) -> Result<AgentPresetConfig, PromptConfigError> {
    let defaults = default_agent_preset_config(agent);
    let default_modules = defaults
        .modules
        .iter()
        .cloned()
        .map(|module| (module.module_id.clone(), module))
        .collect::<HashMap<_, _>>();
    let mut pending_built_in_entries = defaults
        .modules
        .iter()
        .flat_map(|module| {
            module
                .entries
                .iter()
                .cloned()
                .map(move |entry| (entry.entry_id.clone(), (module.module_id.clone(), entry)))
        })
        .collect::<HashMap<_, _>>();
    let mut modules = defaults
        .modules
        .iter()
        .map(|module| {
            (
                module.module_id.clone(),
                AgentPromptModuleConfig {
                    module_id: module.module_id.clone(),
                    display_name: module.display_name.clone(),
                    message_role: module.message_role,
                    order: module.order,
                    entries: Vec::new(),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let mut seen_module_ids = HashSet::new();
    let mut seen_entry_ids = HashSet::new();

    for incoming_module in incoming.modules {
        let module_id = normalize_module_id(incoming_module.module_id)?;
        if !seen_module_ids.insert(module_id.clone()) {
            return Err(PromptConfigError::DuplicateModuleId { agent, module_id });
        }

        let default_module = default_modules.get(&module_id);
        let fallback_name = default_module
            .map(|module| module.display_name.as_str())
            .unwrap_or_else(|| module_id.as_str());
        let module = modules
            .entry(module_id.clone())
            .or_insert_with(|| AgentPromptModuleConfig {
                module_id: module_id.clone(),
                display_name: module_id.as_str().to_owned(),
                message_role: PromptMessageRole::User,
                order: 1_000,
                entries: Vec::new(),
            });
        module.display_name = fallback_display_name(&incoming_module.display_name, fallback_name);
        module.message_role = incoming_module.message_role;
        module.order = incoming_module.order;

        for entry in incoming_module.entries {
            if is_removed_built_in_entry(agent, &entry) {
                continue;
            }
            let normalized_entry =
                normalize_entry(agent, &module_id, entry, &pending_built_in_entries)?;
            if !seen_entry_ids.insert(normalized_entry.entry_id.clone()) {
                return Err(PromptConfigError::DuplicateEntryId {
                    agent,
                    module_id: module_id.clone(),
                    entry_id: normalized_entry.entry_id,
                });
            }
            if normalized_entry.kind != PromptEntryKind::CustomText {
                pending_built_in_entries.remove(&normalized_entry.entry_id);
            }
            module.entries.push(normalized_entry);
        }
    }

    for (default_module_id, entry) in pending_built_in_entries.into_values() {
        if !seen_entry_ids.insert(entry.entry_id.clone()) {
            return Err(PromptConfigError::DuplicateEntryId {
                agent,
                module_id: default_module_id,
                entry_id: entry.entry_id,
            });
        }
        modules
            .get_mut(&default_module_id)
            .expect("default built-in module must exist")
            .entries
            .push(entry);
    }

    let mut modules = modules.into_values().collect::<Vec<_>>();
    for module in &mut modules {
        module.entries.sort_by(|left, right| {
            left.order
                .cmp(&right.order)
                .then_with(|| left.entry_id.cmp(&right.entry_id))
        });
    }
    modules.sort_by(|left, right| {
        left.order
            .cmp(&right.order)
            .then_with(|| left.module_id.as_str().cmp(right.module_id.as_str()))
    });

    Ok(AgentPresetConfig {
        temperature: incoming.temperature,
        max_tokens: incoming.max_tokens,
        director_shared_history_limit: incoming.director_shared_history_limit,
        actor_shared_history_limit: incoming.actor_shared_history_limit,
        actor_private_memory_limit: incoming.actor_private_memory_limit,
        narrator_shared_history_limit: incoming.narrator_shared_history_limit,
        replyer_session_history_limit: incoming.replyer_session_history_limit,
        extra: incoming.extra,
        modules,
    })
}

pub fn compact_agent_preset_config(
    agent: PromptAgentKind,
    incoming: AgentPresetConfig,
) -> Result<AgentPresetConfig, PromptConfigError> {
    let normalized = normalize_agent_preset_config(agent, incoming)?;
    let defaults = default_agent_preset_config(agent);
    let default_modules = defaults
        .modules
        .iter()
        .cloned()
        .map(|module| (module.module_id.clone(), module))
        .collect::<HashMap<_, _>>();
    let default_entries = defaults
        .modules
        .iter()
        .flat_map(|module| {
            module
                .entries
                .iter()
                .cloned()
                .map(move |entry| (entry.entry_id.clone(), (module.module_id.clone(), entry)))
        })
        .collect::<HashMap<_, _>>();

    let mut modules = Vec::new();
    for module in normalized.modules {
        let default_module = default_modules.get(&module.module_id);
        let mut entries = Vec::new();

        for entry in module.entries {
            match entry.kind {
                PromptEntryKind::CustomText => entries.push(entry),
                PromptEntryKind::BuiltInText | PromptEntryKind::BuiltInContextRef => {
                    let Some((default_module_id, default_entry)) =
                        default_entries.get(&entry.entry_id)
                    else {
                        return Err(PromptConfigError::UnknownBuiltInEntry {
                            agent,
                            module_id: module.module_id.clone(),
                            entry_id: entry.entry_id,
                        });
                    };
                    let moved = module.module_id != *default_module_id;
                    let changed = !built_in_entry_matches_default(&entry, default_entry);
                    if moved || changed {
                        entries.push(strip_built_in_entry(entry));
                    }
                }
            }
        }

        let is_custom_module = default_module.is_none();
        let module_metadata_changed = default_module.is_none_or(|default_module| {
            module.display_name != default_module.display_name
                || module.message_role != default_module.message_role
                || module.order != default_module.order
        });

        if is_custom_module || module_metadata_changed || !entries.is_empty() {
            modules.push(AgentPromptModuleConfig {
                module_id: module.module_id,
                display_name: module.display_name,
                message_role: module.message_role,
                order: module.order,
                entries,
            });
        }
    }

    Ok(AgentPresetConfig {
        temperature: normalized.temperature,
        max_tokens: normalized.max_tokens,
        director_shared_history_limit: normalized.director_shared_history_limit,
        actor_shared_history_limit: normalized.actor_shared_history_limit,
        actor_private_memory_limit: normalized.actor_private_memory_limit,
        narrator_shared_history_limit: normalized.narrator_shared_history_limit,
        replyer_session_history_limit: normalized.replyer_session_history_limit,
        extra: normalized.extra,
        modules,
    })
}

fn is_removed_built_in_entry(agent: PromptAgentKind, entry: &AgentPromptModuleEntryConfig) -> bool {
    if entry.kind == PromptEntryKind::CustomText {
        return false;
    }

    matches!(
        (agent, entry.entry_id.as_str()),
        (PromptAgentKind::Director, "director_player_input")
            | (PromptAgentKind::Keeper, "keeper_shared_history")
    )
}

fn normalize_module_id(module_id: PromptModuleId) -> Result<PromptModuleId, PromptConfigError> {
    match module_id {
        PromptModuleId::Custom(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(PromptConfigError::EmptyModuleId);
            }
            Ok(PromptModuleId::from_raw(trimmed.to_owned()))
        }
        module_id => Ok(module_id),
    }
}

fn normalize_entry(
    agent: PromptAgentKind,
    module_id: &PromptModuleId,
    entry: AgentPromptModuleEntryConfig,
    pending_built_in_entries: &HashMap<String, (PromptModuleId, AgentPromptModuleEntryConfig)>,
) -> Result<AgentPromptModuleEntryConfig, PromptConfigError> {
    let entry_id = entry.entry_id.trim().to_owned();
    if entry_id.is_empty() {
        return Err(PromptConfigError::EmptyEntryId);
    }

    match entry.kind {
        PromptEntryKind::CustomText => {
            if entry.text.as_deref().unwrap_or("").trim().is_empty() {
                return Err(PromptConfigError::EmptyCustomEntryText(entry_id));
            }
            Ok(AgentPromptModuleEntryConfig {
                entry_id: entry_id.clone(),
                display_name: fallback_display_name(&entry.display_name, &entry_id),
                kind: PromptEntryKind::CustomText,
                enabled: entry.enabled,
                order: entry.order,
                required: false,
                text: entry.text,
                context_key: None,
            })
        }
        PromptEntryKind::BuiltInText | PromptEntryKind::BuiltInContextRef => {
            let Some((_, default_entry)) = pending_built_in_entries.get(&entry_id) else {
                return Err(PromptConfigError::UnknownBuiltInEntry {
                    agent,
                    module_id: module_id.clone(),
                    entry_id,
                });
            };
            if default_entry.kind != entry.kind {
                return Err(PromptConfigError::UnknownBuiltInEntry {
                    agent,
                    module_id: module_id.clone(),
                    entry_id: entry.entry_id,
                });
            }

            Ok(AgentPromptModuleEntryConfig {
                entry_id,
                display_name: fallback_display_name(
                    &entry.display_name,
                    &default_entry.display_name,
                ),
                kind: default_entry.kind,
                enabled: if default_entry.required {
                    true
                } else {
                    entry.enabled
                },
                order: entry.order,
                required: default_entry.required,
                text: default_entry.text.clone(),
                context_key: default_entry.context_key.clone(),
            })
        }
    }
}

fn strip_built_in_entry(entry: AgentPromptModuleEntryConfig) -> AgentPromptModuleEntryConfig {
    AgentPromptModuleEntryConfig {
        entry_id: entry.entry_id,
        display_name: entry.display_name,
        kind: entry.kind,
        enabled: entry.enabled,
        order: entry.order,
        required: false,
        text: None,
        context_key: None,
    }
}

fn built_in_entry_matches_default(
    entry: &AgentPromptModuleEntryConfig,
    default_entry: &AgentPromptModuleEntryConfig,
) -> bool {
    entry.display_name == default_entry.display_name
        && entry.enabled == default_entry.enabled
        && entry.order == default_entry.order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_drops_removed_director_builtin_entry() {
        let mut config = default_agent_preset_config(PromptAgentKind::Director);
        let module = config
            .modules
            .iter_mut()
            .find(|module| module.module_id == PromptModuleId::DynamicContext)
            .expect("dynamic context module should exist");
        module.entries.push(AgentPromptModuleEntryConfig {
            entry_id: "director_player_input".to_owned(),
            display_name: "Player Input".to_owned(),
            kind: PromptEntryKind::BuiltInContextRef,
            enabled: true,
            order: 40,
            required: true,
            text: None,
            context_key: Some("player_input".to_owned()),
        });

        let normalized = normalize_agent_preset_config(PromptAgentKind::Director, config)
            .expect("normalization should succeed");

        assert!(!normalized
            .modules
            .iter()
            .flat_map(|module| module.entries.iter())
            .any(|entry| entry.entry_id == "director_player_input"));
    }
}
