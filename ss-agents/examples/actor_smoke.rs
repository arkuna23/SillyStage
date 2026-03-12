use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use dotenvy::dotenv;
use llm::{OpenAiClient, OpenAiConfig};
use ss_agents::actor::{
    Actor, ActorRequest, ActorResponse, ActorSegmentKind, ActorStreamEvent, CharacterCard,
};
use ss_agents::director::ActorPurpose;
use state::WorldState;
use state::schema::{StateFieldSchema, StateValueType};
use story::NarrativeNode;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("actor smoke failed: {error}");
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
    let actor = Actor::new(Arc::clone(&client), model.clone())?;
    let (character, cast) = sample_cast();
    let mut world_state = sample_world_state();

    println!("Actor smoke started");
    println!("model: {model}");
    println!("speaker_id: {}", character.id);
    println!("speaker_name: {}", character.name);
    println!();

    world_state.push_player_input_shared_memory(
        "Can you get us through the flooded canal before the tide turns?",
        6,
    );
    let first_node = sample_node("merchant_pitch");
    let first_response = run_turn(
        &actor,
        sample_request(&character, &cast, &first_node),
        &mut world_state,
        "turn_1",
    )
    .await?;
    validate_response(&first_response)?;
    print_memory_summary(&world_state, &character.id, "after_turn_1")?;

    world_state.push_player_input_shared_memory(
        "Then show me why I should trust your route over the guide's.",
        6,
    );
    let second_node = sample_node("merchant_follow_up");
    let second_response = run_turn(
        &actor,
        sample_request(&character, &cast, &second_node),
        &mut world_state,
        "turn_2",
    )
    .await?;
    validate_response(&second_response)?;
    print_memory_summary(&world_state, &character.id, "after_turn_2")?;
    print_summary(&second_response)?;

    Ok(())
}

fn require_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|_| {
            std::io::Error::other(format!("missing required environment variable: {name}"))
        })
        .map_err(Into::into)
}

async fn run_turn(
    actor: &Actor,
    request: ActorRequest<'_>,
    world_state: &mut WorldState,
    label: &str,
) -> Result<ActorResponse, Box<dyn Error>> {
    println!("{label}: start");
    let mut stream = actor.perform_stream(request, world_state).await?;
    let mut final_response = None;

    while let Some(event) = futures_util::StreamExt::next(&mut stream).await {
        match event? {
            ActorStreamEvent::DialogueDelta { delta } => {
                println!("[dialogue] {delta}");
            }
            ActorStreamEvent::ThoughtDelta { delta } => {
                println!("[thought] {delta}");
            }
            ActorStreamEvent::ActionComplete { text } => {
                println!("[action] {text}");
            }
            ActorStreamEvent::Done { response } => {
                final_response = Some(response);
            }
        }
    }

    let response = final_response
        .ok_or_else(|| std::io::Error::other("actor stream ended without a final response"))?;
    println!("{label}: done");
    println!();
    Ok(response)
}

fn sample_cast() -> (CharacterCard, Vec<CharacterCard>) {
    let merchant = CharacterCard {
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
        system_prompt: "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion. For this diagnostic scene, output exactly one <thought>, then one <action>, then one <dialogue>. Each segment must be short, non-empty, and distinct. On the second turn, clearly continue from the recent shared history and private thoughts memory if present."
            .to_owned(),
    };
    let guide = CharacterCard {
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
        system_prompt: "You are a local guide. Stay observant, practical, and in character."
            .to_owned(),
    };

    (merchant.clone(), vec![merchant, guide])
}

fn merchant_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "trust".to_owned(),
        StateFieldSchema::new(StateValueType::Int)
            .with_default(serde_json::json!(0))
            .with_description("How much Haru currently trusts the player"),
    )])
}

fn guide_state_schema() -> HashMap<String, StateFieldSchema> {
    HashMap::from([(
        "knows_safe_route".to_owned(),
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(serde_json::json!(true))
            .with_description("Whether Yuki knows the safe route through the flood"),
    )])
}

fn sample_world_state() -> WorldState {
    let mut world_state = WorldState::new("merchant_pitch");
    world_state.set_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]);
    world_state.set_state("flood_gate_open", serde_json::json!(false));
    world_state.set_character_state("merchant", "trust", serde_json::json!(2));
    world_state.set_character_state("guide", "knows_safe_route", serde_json::json!(true));

    world_state
}

fn sample_request<'a>(
    character: &'a CharacterCard,
    cast: &'a [CharacterCard],
    node: &'a NarrativeNode,
) -> ActorRequest<'a> {
    ActorRequest {
        character,
        cast,
        player_description: "A stubborn courier carrying medicine and trying to judge who is worth trusting.",
        purpose: ActorPurpose::AdvanceGoal,
        node,
        memory_limit: Some(6),
    }
}

fn sample_node(node_id: &str) -> NarrativeNode {
    NarrativeNode::new(
        node_id,
        if node_id == "merchant_pitch" {
            "Dockside Bargain"
        } else {
            "Dockside Follow-up"
        },
        if node_id == "merchant_pitch" {
            "At the flooded dock, the merchant tries to persuade the guide to accept a risky but profitable route."
        } else {
            "The guide has heard the initial pitch, and the merchant now needs to continue the conversation without contradicting earlier turns."
        },
        if node_id == "merchant_pitch" {
            "Offer a profitable deal while showing a visible action and a brief inner thought."
        } else {
            "Continue naturally from the recent exchange while showing continuity with prior dialogue, action, and thought."
        },
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![],
        vec![],
    )
}

fn validate_response(response: &ActorResponse) -> Result<(), Box<dyn Error>> {
    if response.segments.is_empty() {
        return Err(std::io::Error::other("actor response contained no segments").into());
    }

    let mut dialogue_count = 0usize;
    let mut action_count = 0usize;
    let mut thought_count = 0usize;

    for segment in &response.segments {
        if segment.text.trim().is_empty() {
            return Err(std::io::Error::other("actor response contained an empty segment").into());
        }

        match segment.kind {
            ActorSegmentKind::Dialogue => dialogue_count += 1,
            ActorSegmentKind::Action => action_count += 1,
            ActorSegmentKind::Thought => thought_count += 1,
        }
    }

    if dialogue_count == 0 {
        return Err(std::io::Error::other("actor response contained no dialogue segment").into());
    }
    if action_count == 0 {
        return Err(std::io::Error::other("actor response contained no action segment").into());
    }
    if thought_count == 0 {
        return Err(std::io::Error::other("actor response contained no thought segment").into());
    }

    Ok(())
}

fn print_summary(response: &ActorResponse) -> Result<(), Box<dyn Error>> {
    let dialogue_count = response
        .segments
        .iter()
        .filter(|segment| matches!(segment.kind, ActorSegmentKind::Dialogue))
        .count();
    let action_count = response
        .segments
        .iter()
        .filter(|segment| matches!(segment.kind, ActorSegmentKind::Action))
        .count();
    let thought_count = response
        .segments
        .iter()
        .filter(|segment| matches!(segment.kind, ActorSegmentKind::Thought))
        .count();

    println!();
    println!("Actor smoke succeeded");
    println!("segment_count: {}", response.segments.len());
    println!("dialogue_count: {dialogue_count}");
    println!("action_count: {action_count}");
    println!("thought_count: {thought_count}");
    println!("actor_response:");
    println!("{}", serde_json::to_string_pretty(response)?);

    Ok(())
}

fn print_memory_summary(
    world_state: &WorldState,
    character_id: &str,
    label: &str,
) -> Result<(), Box<dyn Error>> {
    println!("{label}:");
    println!(
        "shared_history_count: {}",
        world_state.actor_shared_history().len()
    );
    println!(
        "private_memory_count: {}",
        world_state.actor_private_memory(character_id).len()
    );
    println!(
        "shared_history: {}",
        serde_json::to_string_pretty(world_state.actor_shared_history())?
    );
    println!(
        "private_memory: {}",
        serde_json::to_string_pretty(world_state.actor_private_memory(character_id))?
    );
    println!();

    Ok(())
}
