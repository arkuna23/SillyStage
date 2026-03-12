mod common;

use ss_engine::{AgentApiIds, LlmApiRegistry, RegistryError};

use common::QueuedMockLlm;

fn sample_api_ids() -> AgentApiIds {
    AgentApiIds {
        planner_api_id: "planner".to_owned(),
        architect_api_id: "architect".to_owned(),
        director_api_id: "director".to_owned(),
        actor_api_id: "actor".to_owned(),
        narrator_api_id: "narrator".to_owned(),
        keeper_api_id: "keeper".to_owned(),
    }
}

#[test]
fn registry_builds_story_generation_and_runtime_configs() {
    let llm = QueuedMockLlm::new(vec![], vec![]);
    let api_ids = sample_api_ids();
    let registry = LlmApiRegistry::new()
        .register(&api_ids.planner_api_id, &llm, "planner-model")
        .register(&api_ids.architect_api_id, &llm, "architect-model")
        .register(&api_ids.director_api_id, &llm, "director-model")
        .register(&api_ids.actor_api_id, &llm, "actor-model")
        .register(&api_ids.narrator_api_id, &llm, "narrator-model")
        .register(&api_ids.keeper_api_id, &llm, "keeper-model");

    let generation = registry
        .build_story_generation_configs(&api_ids)
        .expect("generation config should resolve");
    let runtime = registry
        .build_runtime_configs(&api_ids)
        .expect("runtime config should resolve");

    assert_eq!(generation.planner.model, "planner-model");
    assert_eq!(generation.architect.model, "architect-model");
    assert_eq!(runtime.director.model, "director-model");
    assert_eq!(runtime.actor.model, "actor-model");
    assert_eq!(runtime.narrator.model, "narrator-model");
    assert_eq!(runtime.keeper.model, "keeper-model");
}

#[test]
fn registry_reports_unknown_api_ids() {
    let llm = QueuedMockLlm::new(vec![], vec![]);
    let api_ids = sample_api_ids();
    let registry = LlmApiRegistry::new().register("planner", &llm, "planner-model");

    let error = registry
        .build_story_generation_configs(&api_ids)
        .err()
        .expect("missing architect api should fail");

    assert!(matches!(error, RegistryError::UnknownApiId(api_id) if api_id == "architect"));
}
