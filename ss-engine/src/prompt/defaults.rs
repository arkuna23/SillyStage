use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PromptModuleId,
};

use super::templates::templates_for_agent;
use super::types::{BuiltInEntryTemplate, PromptAgentKind};

pub fn default_agent_preset_config(agent: PromptAgentKind) -> AgentPresetConfig {
    AgentPresetConfig {
        temperature: None,
        max_tokens: None,
        extra: None,
        modules: module_order()
            .iter()
            .map(|module_id| AgentPromptModuleConfig {
                module_id: *module_id,
                entries: templates_for_agent(agent)
                    .iter()
                    .filter(|template| template.module_id == *module_id)
                    .map(config_entry_from_template)
                    .collect(),
            })
            .collect(),
    }
}

pub(super) fn module_order() -> [PromptModuleId; 5] {
    [
        PromptModuleId::Role,
        PromptModuleId::Task,
        PromptModuleId::StaticContext,
        PromptModuleId::DynamicContext,
        PromptModuleId::Output,
    ]
}

pub(super) fn config_entry_from_template(
    template: &BuiltInEntryTemplate,
) -> AgentPromptModuleEntryConfig {
    AgentPromptModuleEntryConfig {
        entry_id: template.entry_id.to_owned(),
        display_name: template.display_name.to_owned(),
        kind: template.kind,
        enabled: true,
        order: template.order,
        required: template.required,
        text: template.text.map(str::to_owned),
        context_key: template.context_key.map(str::to_owned),
    }
}

pub(super) fn fallback_display_name(display_name: &str, fallback: &str) -> String {
    let trimmed = display_name.trim();
    if trimmed.is_empty() {
        fallback.trim().to_owned()
    } else {
        trimmed.to_owned()
    }
}
