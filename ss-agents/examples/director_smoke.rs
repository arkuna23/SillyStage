use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use dotenvy::dotenv;
use llm::{OpenAiClient, OpenAiConfig};
use serde_json::json;
use ss_agents::actor::CharacterCard;
use ss_agents::director::{Director, DirectorResult, ResponseBeat};
use state::schema::{StateFieldSchema, StateValueType};
use state::{StateOp, WorldState};
use story::runtime_graph::RuntimeStoryGraph;
use story::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("director smoke failed: {error}");
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
    let director = Director::new(&client, model.clone())?;
    let runtime_graph = sample_runtime_graph()?;
    let character_cards = sample_character_cards();

    let mut stay_world_state = stay_world_state();
    let stay_result = director
        .decide_strict(&runtime_graph, &mut stay_world_state, &character_cards)
        .await?;
    verify_stay_result(&stay_result, &stay_world_state)?;
    print_result("stay", &model, &stay_result, &stay_world_state)?;

    let mut move_world_state = move_world_state();
    let move_result = director
        .decide_strict(&runtime_graph, &mut move_world_state, &character_cards)
        .await?;
    verify_move_result(&move_result, &move_world_state)?;
    print_result("move", &model, &move_result, &move_world_state)?;

    Ok(())
}

fn require_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|_| {
            std::io::Error::other(format!("missing required environment variable: {name}"))
        })
        .map_err(Into::into)
}

fn sample_runtime_graph() -> Result<RuntimeStoryGraph, Box<dyn Error>> {
    let start_node = NarrativeNode::new(
        "dock",
        "Flooded Dock",
        "A courier meets a guide at a half-submerged dock.",
        "Decide whether to trust the guide and move toward the canal gate.",
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![Transition::new(
            "canal_gate",
            Condition::for_character("merchant", "trust", ConditionOperator::Gte, json!(2)),
        )],
        vec![],
    );
    let next_node = NarrativeNode::new(
        "canal_gate",
        "Canal Gate",
        "The group reaches the locked canal gate while water rises nearby.",
        "Choose how to open the gate and keep the route stable.",
        vec!["merchant".to_owned(), "boatman".to_owned()],
        vec![],
        vec![
            StateOp::SetState {
                key: "flood_gate_open".to_owned(),
                value: json!(true),
            },
            StateOp::SetCharacterState {
                character: "boatman".to_owned(),
                key: "knows_safe_route".to_owned(),
                value: json!(true),
            },
        ],
    );

    RuntimeStoryGraph::from_story_graph(StoryGraph::new("dock", vec![start_node, next_node]))
        .map_err(|error| {
            std::io::Error::other(format!("failed to build runtime graph: {error:?}")).into()
        })
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
            system_prompt:
                "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion.".to_owned(),
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
            system_prompt:
                "You are a local guide. Stay observant, practical, and in character.".to_owned(),
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
            state_schema: boatman_state_schema(),
            system_prompt:
                "You are a seasoned boatman. Stay understated and avoid breaking immersion.".to_owned(),
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

fn boatman_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "knows_safe_route".to_owned(),
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description("Whether Ren knows the canal gate approach"),
    )])
}

fn stay_world_state() -> WorldState {
    let mut world_state = WorldState::new("dock");
    world_state.set_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]);
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_character_state("merchant", "trust", json!(1));
    world_state.set_character_state("boatman", "knows_safe_route", json!(false));
    world_state
}

fn move_world_state() -> WorldState {
    let mut world_state = WorldState::new("dock");
    world_state.set_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]);
    world_state.set_state("flood_gate_open", json!(false));
    world_state.set_character_state("merchant", "trust", json!(3));
    world_state.set_character_state("boatman", "knows_safe_route", json!(false));
    world_state
}

fn verify_stay_result(
    result: &DirectorResult,
    world_state: &WorldState,
) -> Result<(), Box<dyn Error>> {
    if result.transitioned {
        return Err(std::io::Error::other("stay scenario unexpectedly transitioned").into());
    }
    if result.previous_node_id != "dock" || result.current_node_id != "dock" {
        return Err(std::io::Error::other("stay scenario ended on the wrong node").into());
    }
    if world_state.current_node != "dock" {
        return Err(std::io::Error::other("stay scenario mutated current_node").into());
    }
    if world_state.state("flood_gate_open") != Some(&json!(false)) {
        return Err(std::io::Error::other("stay scenario mutated global state").into());
    }

    validate_response_plan(result, &["merchant", "guide"])
}

fn verify_move_result(
    result: &DirectorResult,
    world_state: &WorldState,
) -> Result<(), Box<dyn Error>> {
    if !result.transitioned {
        return Err(std::io::Error::other("move scenario did not transition").into());
    }
    if result.previous_node_id != "dock" || result.current_node_id != "canal_gate" {
        return Err(std::io::Error::other("move scenario ended on the wrong node").into());
    }
    if world_state.current_node != "canal_gate" {
        return Err(std::io::Error::other("move scenario did not update current_node").into());
    }
    if world_state.active_characters != vec!["merchant".to_owned(), "boatman".to_owned()] {
        return Err(std::io::Error::other("move scenario did not update active_characters").into());
    }
    if world_state.state("flood_gate_open") != Some(&json!(true)) {
        return Err(
            std::io::Error::other("move scenario did not apply global on_enter_update").into(),
        );
    }
    if world_state.character_state("boatman", "knows_safe_route") != Some(&json!(true)) {
        return Err(
            std::io::Error::other("move scenario did not apply character on_enter_update").into(),
        );
    }

    validate_response_plan(result, &["merchant", "boatman"])
}

fn validate_response_plan(
    result: &DirectorResult,
    allowed_speakers: &[&str],
) -> Result<(), Box<dyn Error>> {
    if result.response_plan.is_empty() {
        return Err(std::io::Error::other("response plan was empty").into());
    }

    for beat in &result.response_plan.beats {
        if let ResponseBeat::Actor { speaker_id, .. } = beat
            && !allowed_speakers
                .iter()
                .any(|candidate| candidate == speaker_id)
        {
            return Err(std::io::Error::other(format!(
                "response plan referenced invalid speaker: {speaker_id}"
            ))
            .into());
        }
    }

    Ok(())
}

fn print_result(
    scenario: &str,
    model: &str,
    result: &DirectorResult,
    world_state: &WorldState,
) -> Result<(), Box<dyn Error>> {
    println!("=== director smoke: {scenario} ===");
    println!("model: {model}");
    println!("transitioned: {}", result.transitioned);
    println!("previous_node: {}", result.previous_node_id);
    println!("current_node: {}", result.current_node_id);
    println!("beats:");
    for (index, beat) in result.response_plan.beats.iter().enumerate() {
        match beat {
            ResponseBeat::Narrator { purpose } => {
                println!("  {}. narrator::{purpose:?}", index + 1);
            }
            ResponseBeat::Actor {
                speaker_id,
                purpose,
            } => {
                println!("  {}. actor::{speaker_id}::{purpose:?}", index + 1);
            }
        }
    }
    println!("world_state:");
    println!("{}", serde_json::to_string_pretty(world_state)?);
    println!("director_result:");
    println!("{}", serde_json::to_string_pretty(result)?);
    println!();

    Ok(())
}
