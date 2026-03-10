mod common;

use std::error::Error;

use common::{
    build_client_from_env, build_story_resources, print_startup_banner,
    print_story_generation_result, resolve_language_from_args, run_interactive_loop,
    seed_runtime_state,
};
use ss_engine::{Engine, RuntimeState, generate_story_graph};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("engine_story_from_resources_smoke failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let language = resolve_language_from_args()?;
    let (client, model) = build_client_from_env()?;
    let resources = build_story_resources(language)?;
    let architect_response = generate_story_graph(&client, model.clone(), &resources).await?;

    let mut runtime_state =
        RuntimeState::from_story_resources(&resources, architect_response.graph.clone())?;
    seed_runtime_state(&mut runtime_state);

    let mut engine = Engine::new(&client, model.clone(), runtime_state)?;
    print_story_generation_result(
        language,
        &architect_response.graph,
        &architect_response.world_state_schema,
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
