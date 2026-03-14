mod common;

use std::sync::Arc;

use ss_engine::{AgentApiIds, LlmApiRegistry, RegistryError};
use store::{LlmApiRecord, LlmProvider};

use common::QueuedMockLlm;

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner".to_owned(),
        architect_api_id: "architect".to_owned(),
        director_api_id: "director".to_owned(),
        actor_api_id: "actor".to_owned(),
        narrator_api_id: "narrator".to_owned(),
        keeper_api_id: "keeper".to_owned(),
        replyer_api_id: "replyer".to_owned(),
    }
}

#[test]
fn registry_builds_story_generation_and_runtime_configs() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let api_ids = sample_api_ids();
    let llm_api: Arc<dyn llm::LlmApi> = llm.clone();
    let registry = LlmApiRegistry::new()
        .register(
            &api_ids.planner_api_id,
            Arc::clone(&llm_api),
            "planner-model",
        )
        .register(
            &api_ids.architect_api_id,
            Arc::clone(&llm_api),
            "architect-model",
        )
        .register(
            &api_ids.director_api_id,
            Arc::clone(&llm_api),
            "director-model",
        )
        .register(&api_ids.actor_api_id, Arc::clone(&llm_api), "actor-model")
        .register(
            &api_ids.narrator_api_id,
            Arc::clone(&llm_api),
            "narrator-model",
        )
        .register(&api_ids.keeper_api_id, Arc::clone(&llm_api), "keeper-model")
        .register(&api_ids.replyer_api_id, llm_api, "replyer-model");

    let generation = registry
        .build_story_generation_configs(&api_ids)
        .expect("generation config should resolve");
    let runtime = registry
        .build_runtime_configs(&api_ids)
        .expect("runtime config should resolve");
    let replyer = registry
        .build_replyer_config(&api_ids)
        .expect("replyer config should resolve");

    assert_eq!(generation.planner.model, "planner-model");
    assert_eq!(generation.architect.model, "architect-model");
    assert_eq!(generation.planner.temperature, None);
    assert_eq!(generation.planner.max_tokens, None);
    assert_eq!(generation.architect.max_tokens, None);
    assert_eq!(runtime.director.model, "director-model");
    assert_eq!(runtime.actor.model, "actor-model");
    assert_eq!(runtime.narrator.model, "narrator-model");
    assert_eq!(runtime.keeper.model, "keeper-model");
    assert_eq!(replyer.model, "replyer-model");
}

#[test]
fn registry_reports_unknown_api_ids() {
    let llm = Arc::new(QueuedMockLlm::new(vec![], vec![]));
    let api_ids = sample_api_ids();
    let registry = LlmApiRegistry::new().register("planner", llm, "planner-model");

    let error = registry
        .build_story_generation_configs(&api_ids)
        .err()
        .expect("missing architect api should fail");

    assert!(matches!(error, RegistryError::UnknownApiId(api_id) if api_id == "architect"));
}

#[test]
fn registry_can_upsert_and_remove_records() {
    let registry = LlmApiRegistry::new();
    let record = LlmApiRecord {
        api_id: "default".to_owned(),
        provider: LlmProvider::OpenAi,
        base_url: "https://api.openai.example/v1".to_owned(),
        api_key: "sk-secret".to_owned(),
        model: "gpt-4.1-mini".to_owned(),
        temperature: Some(0.3),
        max_tokens: Some(512),
    };

    registry
        .upsert_record(&record)
        .expect("record should build into client");
    let resolved = registry.resolve("default").expect("api should resolve");
    assert_eq!(resolved.model, "gpt-4.1-mini");
    assert_eq!(resolved.temperature, Some(0.3));
    assert_eq!(resolved.max_tokens, Some(512));

    registry.remove("default");
    let error = registry
        .resolve("default")
        .err()
        .expect("removed api should no longer resolve");
    assert!(matches!(error, RegistryError::UnknownApiId(api_id) if api_id == "default"));
}
