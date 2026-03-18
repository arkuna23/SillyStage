use std::collections::{HashMap, HashSet};

use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PromptEntryKind,
};

use super::defaults::{default_agent_preset_config, fallback_display_name, module_order};
use super::types::{PromptAgentKind, PromptConfigError};

pub fn normalize_agent_preset_config(
    agent: PromptAgentKind,
    incoming: AgentPresetConfig,
) -> Result<AgentPresetConfig, PromptConfigError> {
    let mut normalized = default_agent_preset_config(agent);
    normalized.temperature = incoming.temperature;
    normalized.max_tokens = incoming.max_tokens;
    normalized.extra = incoming.extra;

    let defaults_by_module = normalized
        .modules
        .iter()
        .map(|module| {
            (
                module.module_id,
                module
                    .entries
                    .iter()
                    .cloned()
                    .map(|entry| (entry.entry_id.clone(), entry))
                    .collect::<HashMap<_, _>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    let mut merged = normalized
        .modules
        .into_iter()
        .map(|module| (module.module_id, module.entries))
        .collect::<HashMap<_, _>>();

    for module in incoming.modules {
        let default_lookup = defaults_by_module
            .get(&module.module_id)
            .cloned()
            .unwrap_or_default();
        let entries = merged.entry(module.module_id).or_default();
        let mut seen = entries
            .iter()
            .map(|entry| entry.entry_id.clone())
            .collect::<HashSet<_>>();

        for entry in module.entries {
            let entry_id = entry.entry_id.trim().to_owned();
            if entry_id.is_empty() {
                return Err(PromptConfigError::EmptyEntryId);
            }

            match entry.kind {
                PromptEntryKind::CustomText => {
                    if entry.text.as_deref().unwrap_or("").trim().is_empty() {
                        return Err(PromptConfigError::EmptyCustomEntryText(entry_id));
                    }
                    if !seen.insert(entry_id.clone()) {
                        return Err(PromptConfigError::DuplicateEntryId {
                            agent,
                            module_id: module.module_id,
                            entry_id,
                        });
                    }
                    entries.push(AgentPromptModuleEntryConfig {
                        entry_id,
                        display_name: fallback_display_name(&entry.display_name, &entry.entry_id),
                        kind: PromptEntryKind::CustomText,
                        enabled: entry.enabled,
                        order: entry.order,
                        required: false,
                        text: entry.text,
                        context_key: None,
                    });
                }
                PromptEntryKind::BuiltInText | PromptEntryKind::BuiltInContextRef => {
                    let Some(default_entry) = default_lookup.get(&entry_id) else {
                        return Err(PromptConfigError::UnknownBuiltInEntry {
                            agent,
                            module_id: module.module_id,
                            entry_id,
                        });
                    };
                    if default_entry.kind != entry.kind {
                        return Err(PromptConfigError::UnknownBuiltInEntry {
                            agent,
                            module_id: module.module_id,
                            entry_id: entry.entry_id,
                        });
                    }

                    if let Some(existing) =
                        entries.iter_mut().find(|item| item.entry_id == entry_id)
                    {
                        existing.display_name =
                            fallback_display_name(&entry.display_name, &existing.display_name);
                        existing.enabled = if existing.required {
                            true
                        } else {
                            entry.enabled
                        };
                        existing.order = entry.order;
                    } else {
                        return Err(PromptConfigError::UnknownBuiltInEntry {
                            agent,
                            module_id: module.module_id,
                            entry_id,
                        });
                    }
                }
            }
        }
    }

    let modules = module_order()
        .iter()
        .map(|module_id| AgentPromptModuleConfig {
            module_id: *module_id,
            entries: {
                let mut entries = merged.remove(module_id).unwrap_or_default();
                entries.sort_by(|left, right| {
                    left.order
                        .cmp(&right.order)
                        .then_with(|| left.entry_id.cmp(&right.entry_id))
                });
                entries
            },
        })
        .collect();

    Ok(AgentPresetConfig {
        temperature: normalized.temperature,
        max_tokens: normalized.max_tokens,
        extra: normalized.extra,
        modules,
    })
}
