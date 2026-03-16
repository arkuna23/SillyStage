use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use dotenvy::dotenv;
use llm::{OpenAiClient, OpenAiConfig};
use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::architect::{Architect, ArchitectRequest};
use state::schema::{PlayerStateSchema, StateFieldSchema, StateValueType, WorldStateSchema};
use story::runtime_graph::RuntimeStoryGraph;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("architect smoke failed: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let base_url = require_env("LLM_API_BASE")?;
    let api_key = require_env("LLM_API_KEY")?;
    let model = require_env("LLM_API_MODEL")?;

    let client = OpenAiClient::new(
        OpenAiConfig::builder()
            .base_url(base_url)
            .api_key(api_key)
            .default_model(model.clone())
            .timeout(Duration::from_secs(180))
            .build()?,
    )?;

    let client: Arc<dyn llm::LlmApi> = Arc::new(client);
    let architect = Architect::new(Arc::clone(&client), model.clone());
    let story_concept = resolve_story_concept();
    let (world_state_schema, player_state_schema, available_characters) = sample_inputs();
    let response = architect
        .generate_graph(sample_request(
            &story_concept,
            Some(&world_state_schema),
            Some(&player_state_schema),
            &available_characters,
        ))
        .await?;
    let runtime_graph =
        RuntimeStoryGraph::from_story_graph(response.graph.clone()).map_err(|error| {
            std::io::Error::other(format!("failed to build runtime graph: {error:?}"))
        })?;
    runtime_graph.export_dot("graph.dot")?;

    println!("Architect smoke succeeded");
    println!("model: {model}");
    println!("start_node: {}", response.graph.start_node());
    println!("node_count: {}", response.graph.len());
    println!(
        "world_state_field_count: {}",
        response.world_state_schema.fields.len()
    );
    println!(
        "player_state_field_count: {}",
        response.player_state_schema.fields.len()
    );
    println!(
        "structured_output: {}",
        response.output.structured_output.is_some()
    );
    println!("dot_export: graph.dot");
    println!();
    println!("player_state_schema:");
    println!(
        "{}",
        serde_json::to_string_pretty(&response.player_state_schema)?
    );
    println!();
    println!("world_state_schema:");
    println!(
        "{}",
        serde_json::to_string_pretty(&response.world_state_schema)?
    );
    println!();
    println!("graph:");
    println!("{}", serde_json::to_string_pretty(&response.graph)?);

    Ok(())
}

fn require_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|_| {
            std::io::Error::other(format!("missing required environment variable: {name}"))
        })
        .map_err(Into::into)
}

fn resolve_story_concept() -> String {
    const DEFAULT_STORY_CONCEPT: &str = "Create a tiny 2-3 node story graph about a courier deciding whether to trust a guide in a flooded city.";

    let arg_concept = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    if !arg_concept.trim().is_empty() {
        return arg_concept;
    }

    DEFAULT_STORY_CONCEPT.to_owned()
}

fn sample_request<'a>(
    story_concept: &'a str,
    world_state_schema: Option<&'a WorldStateSchema>,
    player_state_schema: Option<&'a PlayerStateSchema>,
    available_characters: &'a [CharacterCard],
) -> ArchitectRequest<'a> {
    ArchitectRequest {
        story_concept,
        planned_story: None,
        world_state_schema,
        player_state_schema,
        available_characters,
    }
}

fn sample_inputs() -> (WorldStateSchema, PlayerStateSchema, Vec<CharacterCard>) {
    let mut world_state_schema = WorldStateSchema::new();
    world_state_schema.insert_field(
        "trust_level",
        StateFieldSchema::new(StateValueType::Int)
            .with_default(json!(0))
            .with_description("How much the courier trusts the guide"),
    );
    world_state_schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description("Whether the city flood gate has been opened"),
    );
    let mut player_state_schema = PlayerStateSchema::new();
    player_state_schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int)
            .with_default(json!(8))
            .with_description("How many coins the player currently carries"),
    );
    player_state_schema.insert_field(
        "dock_pass",
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description("Whether the player already holds a valid dock pass"),
    );

    (
        world_state_schema,
        player_state_schema,
        vec![
            CharacterCard {
                id: "merchant".to_owned(),
                name: "Haru".to_owned(),
                personality: "greedy but friendly trader".to_owned(),
                style: "talkative, casual, slightly cunning".to_owned(),
                state_schema: merchant_state_schema(),
                system_prompt:
                    "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion.".to_owned(),
            },
            CharacterCard {
                id: "guide".to_owned(),
                name: "Yuki".to_owned(),
                personality: "calm local guide who notices small details".to_owned(),
                style: "measured, clear, reassuring".to_owned(),
                state_schema: guide_state_schema(),
                system_prompt:
                    "You are a local guide. Stay observant, practical, and in character.".to_owned(),
            },
            CharacterCard {
                id: "boatman".to_owned(),
                name: "Ren".to_owned(),
                personality: "quiet ferryman with a dry sense of humor".to_owned(),
                style: "brief, understated, practical".to_owned(),
                state_schema: boatman_state_schema(),
                system_prompt:
                    "You are a seasoned boatman. Stay understated and avoid breaking immersion.".to_owned(),
            },
        ],
    )
}

fn merchant_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int)
            .with_default(json!(0))
            .with_description("How much Haru currently trusts the courier"),
    )])
}

fn guide_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "knows_safe_route".to_owned(),
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(true))
            .with_description("Whether Yuki knows a safe route through the flood"),
    )])
}

fn boatman_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "knows_safe_route".to_owned(),
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description("Whether Ren knows the canal gate approach"),
    )])
}
