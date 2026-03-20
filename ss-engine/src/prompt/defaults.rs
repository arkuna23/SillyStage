use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PromptMessageRole,
    PromptModuleId,
};

use super::templates::templates_for_agent;
use super::types::{BuiltInEntryTemplate, BuiltInModuleTemplate, PromptAgentKind};

pub fn default_agent_preset_config(agent: PromptAgentKind) -> AgentPresetConfig {
    AgentPresetConfig {
        temperature: None,
        max_tokens: None,
        extra: None,
        modules: built_in_module_templates()
            .iter()
            .map(|module_id| AgentPromptModuleConfig {
                module_id: module_id.module_id.clone(),
                display_name: module_id.display_name.to_owned(),
                message_role: module_id.message_role,
                order: module_id.order,
                entries: templates_for_agent(agent)
                    .iter()
                    .filter(|template| template.module_id == module_id.module_id)
                    .map(config_entry_from_template)
                    .collect(),
            })
            .collect(),
    }
}

pub(super) fn built_in_module_templates() -> [BuiltInModuleTemplate; 5] {
    [
        BuiltInModuleTemplate {
            module_id: PromptModuleId::Role,
            display_name: "Role",
            message_role: PromptMessageRole::System,
            order: 10,
        },
        BuiltInModuleTemplate {
            module_id: PromptModuleId::Task,
            display_name: "Task",
            message_role: PromptMessageRole::System,
            order: 20,
        },
        BuiltInModuleTemplate {
            module_id: PromptModuleId::StaticContext,
            display_name: "Static Context",
            message_role: PromptMessageRole::User,
            order: 30,
        },
        BuiltInModuleTemplate {
            module_id: PromptModuleId::DynamicContext,
            display_name: "Dynamic Context",
            message_role: PromptMessageRole::User,
            order: 40,
        },
        BuiltInModuleTemplate {
            module_id: PromptModuleId::Output,
            display_name: "Output",
            message_role: PromptMessageRole::System,
            order: 50,
        },
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
