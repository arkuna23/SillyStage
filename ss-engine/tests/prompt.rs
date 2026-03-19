use ss_engine::{
    PromptAgentKind, compact_agent_preset_config, default_agent_preset_config,
    normalize_agent_preset_config,
};
use store::{AgentPromptModuleEntryConfig, PromptEntryKind, PromptModuleId};

#[test]
fn compact_agent_preset_config_drops_default_built_ins() {
    let compact = compact_agent_preset_config(
        PromptAgentKind::Planner,
        default_agent_preset_config(PromptAgentKind::Planner),
    )
    .expect("default planner config should compact");

    assert!(compact.modules.is_empty());
    assert_eq!(compact.temperature, None);
    assert_eq!(compact.max_tokens, None);
    assert_eq!(compact.extra, None);
}

#[test]
fn compact_agent_preset_config_preserves_overrides_and_custom_entries() {
    let mut config = default_agent_preset_config(PromptAgentKind::Narrator);
    let static_context_module = config
        .modules
        .iter_mut()
        .find(|module| module.module_id == PromptModuleId::StaticContext)
        .expect("static context module should exist");

    let built_in = static_context_module
        .entries
        .iter_mut()
        .find(|entry| entry.entry_id == "narrator_lorebook_base")
        .expect("built-in entry should exist");
    built_in.display_name = "Narrator Lorebook Base".to_owned();
    built_in.enabled = false;
    built_in.order = 15;

    let task_module = config
        .modules
        .iter_mut()
        .find(|module| module.module_id == PromptModuleId::Task)
        .expect("task module should exist");
    task_module.entries.push(AgentPromptModuleEntryConfig {
        entry_id: "narrator-tone".to_owned(),
        display_name: "Narrator Tone".to_owned(),
        kind: PromptEntryKind::CustomText,
        enabled: true,
        order: 95,
        required: false,
        text: Some("Keep the narration dry.".to_owned()),
        context_key: None,
    });

    let normalized = normalize_agent_preset_config(PromptAgentKind::Narrator, config)
        .expect("config should normalize");
    let compact = compact_agent_preset_config(PromptAgentKind::Narrator, normalized.clone())
        .expect("config should compact");

    assert_eq!(compact.modules.len(), 2);
    let compact_static_context = compact
        .modules
        .iter()
        .find(|module| module.module_id == PromptModuleId::StaticContext)
        .expect("static context module should be stored");
    let compact_built_in = compact_static_context
        .entries
        .iter()
        .find(|entry| entry.entry_id == "narrator_lorebook_base")
        .expect("built-in override should be stored");
    assert_eq!(compact_built_in.kind, PromptEntryKind::BuiltInContextRef);
    assert_eq!(compact_built_in.text, None);
    assert_eq!(compact_built_in.context_key, None);
    assert!(!compact_built_in.enabled);
    assert_eq!(compact_built_in.order, 15);

    let expanded = normalize_agent_preset_config(PromptAgentKind::Narrator, compact)
        .expect("compact config should expand");
    assert_eq!(expanded, normalized);
}
