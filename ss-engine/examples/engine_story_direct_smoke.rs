mod common;

use std::error::Error;

use common::{
    build_client_from_env, build_direct_story_bundle, localized_player_description,
    print_direct_story_summary, print_startup_banner, resolve_smoke_options, run_interactive_loop,
    seed_runtime_state, shared_runtime_agent_configs,
};
use ss_engine::{Engine, RuntimeState};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("engine_story_direct_smoke failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let options = resolve_smoke_options(false)?;
    let language = options.language;
    let (client, model) = build_client_from_env()?;
    let runtime_agent_configs = shared_runtime_agent_configs(&client, model.clone());
    let bundle = build_direct_story_bundle(language);
    let introduction = bundle.introduction.clone();
    let story_graph = bundle.story_graph.clone();

    let mut runtime_state = RuntimeState::from_story_graph(
        bundle.story_id,
        bundle.story_graph,
        bundle.character_cards,
        localized_player_description(language),
        bundle.player_state_schema,
    )?;
    seed_runtime_state(&mut runtime_state);

    let mut engine = Engine::new(runtime_agent_configs, runtime_state)?;
    print_direct_story_summary(language, &story_graph)?;
    print_startup_banner(
        language,
        "direct",
        &model,
        engine.runtime_state(),
        &introduction,
        engine.runtime_state().character_cards(),
    )?;
    run_interactive_loop(&mut engine, language).await
}
