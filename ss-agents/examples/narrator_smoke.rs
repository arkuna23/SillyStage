use std::error::Error;
use std::time::Duration;

use dotenvy::dotenv;
use llm::{OpenAiClient, OpenAiConfig};
use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::director::NarratorPurpose;
use ss_agents::narrator::{Narrator, NarratorRequest, NarratorResponse, NarratorStreamEvent};
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};
use story::NarrativeNode;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("narrator smoke failed: {error}");
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
    let narrator = Narrator::new(&client, model.clone())?;

    let character_cards = sample_character_cards();

    print_response(
        "describe_scene",
        &model,
        run_scenario(
            &narrator,
            NarratorRequest {
                purpose: NarratorPurpose::DescribeScene,
                previous_node: None,
                current_node: dock_node(),
                character_cards: character_cards.clone(),
                world_state: dock_world_state(),
            },
        )
        .await?,
    )?;

    print_response(
        "describe_transition",
        &model,
        run_scenario(
            &narrator,
            NarratorRequest {
                purpose: NarratorPurpose::DescribeTransition,
                previous_node: Some(market_node()),
                current_node: dock_node(),
                character_cards: character_cards.clone(),
                world_state: dock_world_state(),
            },
        )
        .await?,
    )?;

    print_response(
        "describe_result",
        &model,
        run_scenario(
            &narrator,
            NarratorRequest {
                purpose: NarratorPurpose::DescribeResult,
                previous_node: None,
                current_node: gate_node(),
                character_cards,
                world_state: gate_world_state(),
            },
        )
        .await?,
    )?;

    Ok(())
}

fn require_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|_| {
            std::io::Error::other(format!("missing required environment variable: {name}"))
        })
        .map_err(Into::into)
}

async fn run_scenario(
    narrator: &Narrator<'_>,
    request: NarratorRequest,
) -> Result<NarratorResponse, Box<dyn Error>> {
    let mut stream = narrator.narrate_stream(request).await?;
    let mut final_response = None;

    while let Some(event) = futures_util::StreamExt::next(&mut stream).await {
        match event? {
            NarratorStreamEvent::TextDelta { delta } => {
                print!("{delta}");
            }
            NarratorStreamEvent::Done { response } => {
                final_response = Some(response);
            }
        }
    }

    println!();
    let response = final_response
        .ok_or_else(|| std::io::Error::other("narrator stream ended without a final response"))?;
    if response.text.trim().is_empty() {
        return Err(std::io::Error::other("narrator returned empty text").into());
    }

    Ok(response)
}

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![
        CharacterCard {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual, slightly cunning".to_owned(),
            tendencies: vec![
                "likes profitable deals".to_owned(),
                "avoids danger".to_owned(),
                "tries to maintain good relationships".to_owned(),
            ],
            system_prompt: "You are a traveling merchant. Stay in character.".to_owned(),
        },
        CharacterCard {
            id: "guide".to_owned(),
            name: "Yuki".to_owned(),
            personality: "calm local guide who notices small details".to_owned(),
            style: "measured, clear, reassuring".to_owned(),
            tendencies: vec![
                "prefers careful plans".to_owned(),
                "protects civilians".to_owned(),
                "shares local knowledge sparingly".to_owned(),
            ],
            system_prompt: "You are a local guide. Stay observant.".to_owned(),
        },
        CharacterCard {
            id: "boatman".to_owned(),
            name: "Ren".to_owned(),
            personality: "quiet ferryman with a dry sense of humor".to_owned(),
            style: "brief, understated, practical".to_owned(),
            tendencies: vec![
                "avoids unnecessary risk".to_owned(),
                "values loyalty".to_owned(),
                "keeps useful tools nearby".to_owned(),
            ],
            system_prompt: "You are a seasoned boatman. Stay understated.".to_owned(),
        },
    ]
}

fn market_node() -> NarrativeNode {
    NarrativeNode::new(
        "market",
        "Night Market",
        "Lantern light flickers over a narrow market lane full of wet cobblestones and shuttered stalls.",
        "Reach the dock before the route closes.",
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![],
        vec![],
    )
}

fn dock_node() -> NarrativeNode {
    NarrativeNode::new(
        "dock",
        "Flooded Dock",
        "A flooded dock creaks under rising water while ropes snap against old posts.",
        "Decide whether to trust the guide and push toward the canal gate.",
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![],
        vec![],
    )
}

fn gate_node() -> NarrativeNode {
    NarrativeNode::new(
        "canal_gate",
        "Canal Gate",
        "The locked canal gate looms over black water as the group gathers beside a narrow service ledge.",
        "Stabilize the route and open the gate before the water rises higher.",
        vec!["merchant".to_owned(), "boatman".to_owned()],
        vec![],
        vec![],
    )
}

fn dock_world_state() -> WorldState {
    let mut world_state = WorldState::new("dock");
    world_state.set_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]);
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_character_state("merchant", "trust", json!(2));
    world_state.set_character_state("guide", "knows_safe_route", json!(true));
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "guide".to_owned(),
            speaker_name: "Yuki".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "The dock is barely holding, but the canal gate is still reachable.".to_owned(),
        },
        8,
    );

    world_state
}

fn gate_world_state() -> WorldState {
    let mut world_state = WorldState::new("canal_gate");
    world_state.set_active_characters(vec!["merchant".to_owned(), "boatman".to_owned()]);
    world_state.set_state("flood_gate_open", json!(true));
    world_state.set_character_state("boatman", "knows_safe_route", json!(true));
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Haru".to_owned(),
            kind: ActorMemoryKind::Action,
            text: "Haru wedges a crate beneath the winch and hauls the gate chain upward."
                .to_owned(),
        },
        8,
    );
    world_state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "boatman".to_owned(),
            speaker_name: "Ren".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "That'll hold for now. The safe route is open if we move quickly.".to_owned(),
        },
        8,
    );

    world_state
}

fn print_response(
    scenario: &str,
    model: &str,
    response: NarratorResponse,
) -> Result<(), Box<dyn Error>> {
    println!("=== narrator smoke: {scenario} ===");
    println!("model: {model}");
    println!("narrator_response:");
    println!("{}", serde_json::to_string_pretty(&response)?);
    println!();

    Ok(())
}
