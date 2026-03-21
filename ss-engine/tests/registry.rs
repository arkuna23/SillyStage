mod common;

use std::sync::Arc;

use ss_engine::{LlmApiRegistry, RegistryError, RuntimeApiRecords};
use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, ApiRecord,
    LlmProvider, PresetAgentConfigs, PresetRecord, PromptEntryKind, PromptMessageRole,
    PromptModuleId,
};

use common::QueuedMockLlm;

fn sample_api_record(api_id: &str, model: &str) -> ApiRecord {
    ApiRecord {
        api_id: api_id.to_owned(),
        display_name: format!("API {api_id}"),
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: "sk-secret".to_owned(),
        model: model.to_owned(),
    }
}

fn sample_agent_preset_config(max_tokens: u32) -> AgentPresetConfig {
    AgentPresetConfig {
        temperature: Some(0.1),
        max_tokens: Some(max_tokens),
        director_shared_history_limit: None,
        actor_shared_history_limit: None,
        actor_private_memory_limit: None,
        narrator_shared_history_limit: None,
        replyer_session_history_limit: None,
        extra: None,
        modules: vec![AgentPromptModuleConfig {
            module_id: PromptModuleId::Task,
            display_name: "Task".to_owned(),
            message_role: PromptMessageRole::System,
            order: 20,
            entries: vec![AgentPromptModuleEntryConfig {
                entry_id: format!("entry-{max_tokens}"),
                display_name: format!("Prompt {max_tokens}"),
                kind: PromptEntryKind::CustomText,
                enabled: true,
                order: 10,
                required: false,
                text: Some(format!(
                    "Keep replies under {max_tokens} tokens when practical."
                )),
                context_key: None,
            }],
        }],
    }
}

fn sample_preset() -> PresetRecord {
    PresetRecord {
        preset_id: "preset-default".to_owned(),
        display_name: "Default Preset".to_owned(),
        agents: PresetAgentConfigs {
            planner: sample_agent_preset_config(512),
            architect: sample_agent_preset_config(8_192),
            director: sample_agent_preset_config(512),
            actor: sample_agent_preset_config(512),
            narrator: sample_agent_preset_config(512),
            keeper: sample_agent_preset_config(512),
            replyer: sample_agent_preset_config(256),
        },
    }
}

#[test]
fn registry_builds_story_generation_and_runtime_configs_for_group() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let llm_api: Arc<dyn llm::LlmApi> = llm.clone();
    let planner_api = sample_api_record("api-planner", "planner-model");
    let architect_api = sample_api_record("api-architect", "architect-model");
    let director_api = sample_api_record("api-director", "director-model");
    let actor_api = sample_api_record("api-actor", "actor-model");
    let narrator_api = sample_api_record("api-narrator", "narrator-model");
    let keeper_api = sample_api_record("api-keeper", "keeper-model");
    let replyer_api = sample_api_record("api-replyer", "replyer-model");
    let mut preset = sample_preset();
    preset.agents.director.director_shared_history_limit = Some(10);
    preset.agents.actor.actor_shared_history_limit = Some(12);
    preset.agents.actor.actor_private_memory_limit = Some(4);
    preset.agents.narrator.narrator_shared_history_limit = Some(6);
    preset.agents.replyer.replyer_session_history_limit = Some(5);
    let registry = LlmApiRegistry::new()
        .register(
            "api-planner",
            Arc::clone(&llm_api),
            "planner-override-model",
        )
        .register(
            "api-architect",
            Arc::clone(&llm_api),
            "architect-override-model",
        )
        .register(
            "api-director",
            Arc::clone(&llm_api),
            "director-override-model",
        )
        .register("api-actor", Arc::clone(&llm_api), "actor-override-model")
        .register(
            "api-narrator",
            Arc::clone(&llm_api),
            "narrator-override-model",
        )
        .register("api-keeper", Arc::clone(&llm_api), "keeper-override-model")
        .register("api-replyer", llm_api, "replyer-override-model");

    let generation = registry
        .build_story_generation_configs(
            &planner_api,
            &architect_api,
            &preset.agents.planner,
            &preset.agents.architect,
        )
        .expect("generation config should resolve");
    let runtime = registry
        .build_runtime_configs(
            RuntimeApiRecords {
                director: &director_api,
                actor: &actor_api,
                narrator: &narrator_api,
                keeper: &keeper_api,
            },
            &preset,
        )
        .expect("runtime config should resolve");
    let replyer = registry
        .build_replyer_config(&replyer_api, &preset.agents.replyer)
        .expect("replyer config should resolve");

    assert_eq!(generation.planner.model, "planner-override-model");
    assert_eq!(generation.architect.model, "architect-override-model");
    assert_eq!(generation.planner.temperature, Some(0.1));
    assert_eq!(generation.planner.max_tokens, Some(512));
    assert_eq!(generation.architect.max_tokens, Some(8_192));
    assert!(
        generation
            .planner
            .prompt_profile
            .system_prompt
            .contains("Keep replies under 512 tokens when practical.")
    );
    assert_eq!(runtime.director.model, "director-override-model");
    assert_eq!(runtime.actor.model, "actor-override-model");
    assert_eq!(runtime.narrator.model, "narrator-override-model");
    assert_eq!(runtime.keeper.model, "keeper-override-model");
    assert!(
        runtime
            .director
            .prompt_profile
            .system_prompt
            .contains("Keep replies under 512 tokens when practical.")
    );
    assert_eq!(replyer.model, "replyer-override-model");
    assert!(
        replyer
            .prompt_profile
            .system_prompt
            .contains("Keep replies under 256 tokens when practical.")
    );
    assert_eq!(runtime.director.shared_history_limit, Some(10));
    assert_eq!(runtime.actor.shared_history_limit, Some(12));
    assert_eq!(runtime.actor.private_memory_limit, Some(4));
    assert_eq!(runtime.narrator.shared_history_limit, Some(6));
    assert_eq!(runtime.shared_memory_limit, 12);
    assert_eq!(replyer.session_history_limit, Some(5));
}

#[test]
fn registry_reports_unknown_override_api_ids() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let registry = LlmApiRegistry::new().register("api-planner", llm, "planner-model");

    let error = registry
        .resolve("api-architect")
        .err()
        .expect("missing architect override should fail");

    assert!(matches!(error, RegistryError::UnknownApiId(api_id) if api_id == "api-architect"));
}

#[test]
fn registry_falls_back_to_group_records_when_override_is_missing() {
    let planner_api = sample_api_record("api-planner", "planner-model");
    let architect_api = sample_api_record("api-architect", "architect-model");
    let preset = sample_preset();
    let registry = LlmApiRegistry::new();

    let generation = registry
        .build_story_generation_configs(
            &planner_api,
            &architect_api,
            &preset.agents.planner,
            &preset.agents.architect,
        )
        .expect("group config should build without overrides");

    assert_eq!(generation.planner.model, "planner-model");
    assert_eq!(generation.planner.temperature, Some(0.1));
    assert_eq!(generation.planner.max_tokens, Some(512));
    assert!(
        generation
            .planner
            .prompt_profile
            .system_prompt
            .contains("Keep replies under 512 tokens when practical.")
    );
}
