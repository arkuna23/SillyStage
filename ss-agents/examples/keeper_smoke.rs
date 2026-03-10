use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use dotenvy::dotenv;
use llm::{OpenAiClient, OpenAiConfig};
use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::director::{ActorPurpose, NarratorPurpose};
use ss_agents::keeper::{
    Keeper, KeeperActorSegment, KeeperActorSegmentKind, KeeperBeat, KeeperPhase, KeeperRequest,
};
use state::WorldState;
use state::schema::{StateFieldSchema, StateValueType};
use story::NarrativeNode;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("keeper smoke failed: {error}");
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
    let keeper = Keeper::new(&client, model.clone())?;
    let character_cards = sample_character_cards();
    let market = market_node();
    let dock = dock_node();
    let dock_world_state = dock_world_state();
    let input_completed_beats: Vec<KeeperBeat> = Vec::new();
    let output_completed_beats = vec![
        KeeperBeat::Narrator {
            purpose: NarratorPurpose::DescribeTransition,
            text: "The market lane gives way to a flooded dock where ropes whip against the posts."
                .to_owned(),
        },
        KeeperBeat::Actor {
            speaker_id: "merchant".to_owned(),
            purpose: ActorPurpose::AdvanceGoal,
            visible_segments: vec![
                KeeperActorSegment {
                    kind: KeeperActorSegmentKind::Dialogue,
                    text: "We're committed now. The gate is our best chance.".to_owned(),
                },
                KeeperActorSegment {
                    kind: KeeperActorSegmentKind::Action,
                    text: "Haru lifts the lantern and steps onto the slick boards.".to_owned(),
                },
            ],
        },
    ];

    let input_request = KeeperRequest {
        phase: KeeperPhase::AfterPlayerInput,
        player_input: "I tell Yuki that I will take the canal route and keep moving.",
        previous_node: None,
        current_node: &dock,
        character_cards: &character_cards,
        world_state: &dock_world_state,
        completed_beats: &input_completed_beats,
    };
    print_result("after_player_input", &model, &keeper, input_request).await?;

    let output_request = KeeperRequest {
        phase: KeeperPhase::AfterTurnOutputs,
        player_input: "I tell Yuki that I will take the canal route and keep moving.",
        previous_node: Some(&market),
        current_node: &dock,
        character_cards: &character_cards,
        world_state: &dock_world_state,
        completed_beats: &output_completed_beats,
    };
    print_result("after_turn_outputs", &model, &keeper, output_request).await?;

    Ok(())
}

fn require_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|_| {
            std::io::Error::other(format!("missing required environment variable: {name}"))
        })
        .map_err(Into::into)
}

async fn print_result(
    label: &str,
    model: &str,
    keeper: &Keeper<'_>,
    request: KeeperRequest<'_>,
) -> Result<(), Box<dyn Error>> {
    let original_world_state = request.world_state.clone();
    let response = keeper.keep(request).await?;
    let mut applied_world_state = original_world_state;
    applied_world_state.apply_update(response.update.clone());

    println!("=== keeper smoke: {label} ===");
    println!("model: {model}");
    println!("state_update:");
    println!("{}", serde_json::to_string_pretty(&response.update)?);
    println!("world_state_after_apply:");
    println!("{}", serde_json::to_string_pretty(&applied_world_state)?);
    println!();

    Ok(())
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
            state_schema: merchant_state_schema(),
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
            state_schema: guide_state_schema(),
            system_prompt: "You are a local guide. Stay observant.".to_owned(),
        },
    ]
}

fn merchant_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int)
            .with_default(json!(0))
            .with_description("How much Haru currently trusts the player"),
    )])
}

fn guide_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "knows_safe_route".to_owned(),
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(true))
            .with_description("Whether Yuki knows the safe path through the docks"),
    )])
}

fn market_node() -> NarrativeNode {
    NarrativeNode::new(
        "market",
        "Night Market",
        "Lantern light flickers over wet cobblestones and shuttered stalls.",
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

fn dock_world_state() -> WorldState {
    let mut world_state = WorldState::new("dock");
    world_state.set_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]);
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_character_state("merchant", "trust", json!(2));
    world_state.set_character_state("guide", "knows_safe_route", json!(true));
    world_state.push_player_input_shared_memory(
        "I tell Yuki that I will take the canal route and keep moving.",
        8,
    );
    world_state
}
