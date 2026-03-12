mod common;

use std::error::Error;

use common::{
    build_client_from_env, build_story_resources, localized_player_description,
    print_planned_story, print_startup_banner, print_story_generation_result,
    resolve_smoke_options, run_interactive_loop, seed_runtime_state,
    shared_generation_agent_configs, shared_runtime_agent_configs,
};
use ss_engine::{Engine, RuntimeState, generate_story_graph, generate_story_plan};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("engine_story_from_resources_smoke failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let options = resolve_smoke_options(true)?;
    let language = options.language;
    let (client, model) = build_client_from_env()?;
    let generation_agent_configs = shared_generation_agent_configs(client.clone(), model.clone());
    let runtime_agent_configs = shared_runtime_agent_configs(client, model.clone());
    let mut resources = build_story_resources(language)?;

    if options.use_planner {
        let planner_response = generate_story_plan(&generation_agent_configs, &resources).await?;
        print_planned_story(language, &planner_response.story_script)?;
        resources = resources.with_planned_story(planner_response.story_script);
    }

    let architect_response = generate_story_graph(&generation_agent_configs, &resources).await?;

    let mut runtime_state = RuntimeState::from_story_resources(
        &resources,
        architect_response.graph.clone(),
        localized_player_description(language),
        architect_response.player_state_schema.clone(),
    )?;
    seed_runtime_state(&mut runtime_state);

    let mut engine = Engine::new(runtime_agent_configs, runtime_state)?;
    print_story_generation_result(
        language,
        &architect_response.graph,
        &architect_response.world_state_schema,
        &architect_response.player_state_schema,
    )?;
    print_startup_banner(
        language,
        "from_resources",
        &model,
        engine.runtime_state(),
        &architect_response.introduction,
        resources.character_cards(),
    )?;
    run_interactive_loop(&mut engine, language).await
}
