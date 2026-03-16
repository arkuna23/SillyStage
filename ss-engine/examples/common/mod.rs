#![allow(dead_code)]

use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;

use agents::actor::CharacterCard;
use dotenvy::dotenv;
use futures_util::StreamExt;
use llm::{LlmApi, OpenAiClient, OpenAiConfig};
use serde_json::json;
use ss_engine::{
    Engine, EngineEvent, RuntimeAgentConfigs, RuntimeSnapshot, RuntimeState,
    StoryGenerationAgentConfigs, StoryResources,
};
use state::{PlayerStateSchema, StateFieldSchema, StateOp, StateValueType, WorldStateSchema};
use story::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Zh,
    En,
}

impl Language {
    pub fn text(self, zh: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::Zh => zh,
            Self::En => en,
        }
    }
}

pub struct DirectStoryBundle {
    pub story_id: String,
    pub introduction: String,
    pub story_graph: StoryGraph,
    pub character_cards: Vec<CharacterCard>,
    pub player_state_schema: PlayerStateSchema,
}

pub struct SmokeOptions {
    pub language: Language,
    pub use_planner: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeltaChannel {
    Narrator(usize),
    ActorThought(String),
    ActorDialogue(String),
}

#[derive(Default)]
struct DeltaPrinter {
    active: Option<DeltaChannel>,
}

impl DeltaPrinter {
    fn push(
        &mut self,
        channel: DeltaChannel,
        prefix: impl AsRef<str>,
        delta: &str,
    ) -> Result<(), Box<dyn Error>> {
        if self.active.as_ref() != Some(&channel) {
            self.finish_line()?;
            print!("{}", prefix.as_ref());
            io::stdout().flush()?;
            self.active = Some(channel);
        }

        print!("{delta}");
        io::stdout().flush()?;
        Ok(())
    }

    fn finish_line(&mut self) -> Result<(), Box<dyn Error>> {
        if self.active.take().is_some() {
            println!();
            io::stdout().flush()?;
        }

        Ok(())
    }
}

pub fn resolve_smoke_options(allow_planner: bool) -> Result<SmokeOptions, Box<dyn Error>> {
    let mut language = Language::Zh;
    let mut use_planner = false;
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lang" => {
                let value = args.next().ok_or_else(|| {
                    io::Error::other("missing language after --lang, expected 'zh' or 'en'")
                })?;
                language = parse_language(&value)?;
            }
            "--help" | "-h" => {
                print_usage_and_exit(allow_planner);
            }
            "--planner" if allow_planner => {
                use_planner = true;
            }
            "--planner" => {
                return Err(io::Error::other(
                    "the --planner flag is only supported by the from_resources smoke",
                )
                .into());
            }
            _ if arg.starts_with("--lang=") => {
                let value = arg
                    .split_once('=')
                    .map(|(_, value)| value)
                    .ok_or_else(|| io::Error::other("invalid --lang argument"))?;
                language = parse_language(value)?;
            }
            _ => {
                return Err(io::Error::other(format!("unexpected argument: {arg}")).into());
            }
        }
    }

    Ok(SmokeOptions {
        language,
        use_planner,
    })
}

pub fn resolve_language_from_args() -> Result<Language, Box<dyn Error>> {
    Ok(resolve_smoke_options(false)?.language)
}

pub fn build_client_from_env() -> Result<(Arc<dyn LlmApi>, String), Box<dyn Error>> {
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

    Ok((Arc::new(client), model))
}

pub fn shared_generation_agent_configs(
    client: Arc<dyn LlmApi>,
    model: impl Into<String>,
) -> StoryGenerationAgentConfigs {
    StoryGenerationAgentConfigs::shared(client, model)
}

pub fn shared_runtime_agent_configs(
    client: Arc<dyn LlmApi>,
    model: impl Into<String>,
) -> RuntimeAgentConfigs {
    RuntimeAgentConfigs::shared(client, model)
}

pub fn build_story_resources(language: Language) -> Result<StoryResources, Box<dyn Error>> {
    let resources = StoryResources::new(
        match language {
            Language::Zh => "harbor_passage_zh",
            Language::En => "harbor_passage_en",
        },
        language.text(
            "生成一个适合多轮互动的微型剧情：暴雨淹没港口后，信使需要说服商人和向导，找到一条通往运河闸门的安全路线。故事需要适合玩家持续对话推进，使用现有角色，不要超过 3 个节点，并生成必要的全局 world state schema。",
            "Create a tiny multi-turn interactive story: after a storm floods the harbor, a courier must persuade a merchant and a guide to reveal a safe route to the canal gate. The story should support continued player dialogue, use the provided characters, stay within 3 nodes, and generate the necessary global world state schema.",
        ),
        localized_character_cards(language),
        Some(localized_player_state_schema(language)),
    )?
    .with_world_state_schema_seed(localized_world_state_schema_seed(language));

    Ok(resources)
}

pub fn build_direct_story_bundle(language: Language) -> DirectStoryBundle {
    let dock = NarrativeNode::new(
        "dock",
        language.text("暴雨码头", "Storm Dock"),
        language.text(
            "暴雨过后的港口被海水淹没。商人 Haru 和向导 Yuki 正在争论，是否要带信使穿过仍在上涨的水道。",
            "The harbor is half-flooded after the storm. Haru the merchant and Yuki the guide are arguing over whether the courier should cross the still-rising canal route.",
        ),
        language.text(
            "让玩家决定先信任谁，并尝试争取一条通往运河闸门的路线。",
            "Let the player decide whom to trust first and try to secure a route toward the canal gate.",
        ),
        vec!["merchant".to_owned(), "guide".to_owned()],
        vec![Transition::new(
            "canal_gate",
            Condition::for_character("merchant", "trust", ConditionOperator::Gte, json!(2)),
        )],
        vec![],
    );
    let canal_gate = NarrativeNode::new(
        "canal_gate",
        language.text("运河闸门", "Canal Gate"),
        language.text(
            "队伍来到闸门前，船夫 Ren 正在检查锁链和被泥沙堵住的机械结构。",
            "The group reaches the canal gate, where Ren the boatman is checking the chains and the silt-clogged mechanism.",
        ),
        language.text(
            "让玩家推动下一步行动，并决定是否现在就打开闸门。",
            "Let the player push the next action and decide whether the gate should be opened now.",
        ),
        vec!["merchant".to_owned(), "boatman".to_owned()],
        vec![],
        vec![StateOp::SetState {
            key: "flood_gate_open".to_owned(),
            value: json!(true),
        }],
    );

    DirectStoryBundle {
        story_id: match language {
            Language::Zh => "harbor_passage_direct_zh".to_owned(),
            Language::En => "harbor_passage_direct_en".to_owned(),
        },
        introduction: language
            .text(
                "暴雨后的港口仍被潮水和碎木覆盖。信使刚到码头，商人 Haru 想趁乱赚一笔，而向导 Yuki 则坚持要先确认一条安全路线。玩家必须从这场争执中找到突破口，决定谁值得先信任。",
                "The harbor is still covered in storm water and drifting debris. The courier has just arrived at the dock, where Haru the merchant wants to profit from the chaos while Yuki the guide insists on securing a safe route first. The player must find an opening in this dispute and decide whom to trust first.",
            )
            .to_owned(),
        story_graph: StoryGraph::new("dock", vec![dock, canal_gate]),
        character_cards: localized_character_cards(language),
        player_state_schema: localized_player_state_schema(language),
    }
}

pub fn localized_player_description(language: Language) -> &'static str {
    language.text(
        "你是一名谨慎但固执的信使，背着一只装有药品和密封文书的挎包，眼下最重要的是在洪水彻底切断道路前把物资送达。",
        "You are a cautious but stubborn courier carrying a satchel of medicine and sealed documents. Your priority is to get the supplies through before the flood cuts off the route completely.",
    )
}

pub fn seed_runtime_state(runtime_state: &mut RuntimeState) {
    runtime_state
        .world_state_mut()
        .set_state("flood_gate_open", json!(false));
    runtime_state
        .world_state_mut()
        .set_state("route_revealed", json!(false));
    runtime_state
        .world_state_mut()
        .set_player_state("coins", json!(8));
    runtime_state
        .world_state_mut()
        .set_player_state("dock_pass", json!(false));
    runtime_state
        .world_state_mut()
        .set_character_state("merchant", "trust", json!(1));
    runtime_state
        .world_state_mut()
        .set_character_state("guide", "knows_safe_route", json!(true));
    runtime_state.world_state_mut().set_character_state(
        "boatman",
        "knows_safe_route",
        json!(false),
    );
}

pub fn print_startup_banner(
    language: Language,
    mode_name: &str,
    model: &str,
    runtime_state: &RuntimeState,
    introduction: &str,
    character_cards: &[CharacterCard],
) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        language.text("Engine smoke 已启动", "Engine smoke started")
    );
    println!("mode: {mode_name}");
    println!("lang: {}", language.text("中文", "English"));
    println!("model: {model}");
    println!("story_id: {}", runtime_state.story_id());
    print_runtime_summary(
        &runtime_state.snapshot(),
        language.text("初始状态", "Initial state"),
    )?;
    println!("{}", language.text("故事开场：", "Story introduction:"));
    println!("{introduction}");
    println!();
    print_character_roster(language, character_cards);
    println!(
        "{}",
        language.text(
            "输入内容推进剧情，输入 exit / quit / 退出 结束。",
            "Type input to advance the story. Use exit / quit / 退出 to stop.",
        )
    );
    println!();
    Ok(())
}

pub fn print_story_generation_result(
    language: Language,
    graph: &StoryGraph,
    world_state_schema: &WorldStateSchema,
    player_state_schema: &PlayerStateSchema,
) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        language.text(
            "已从 StoryResources 生成剧情。",
            "Story generated from StoryResources."
        )
    );
    println!("start_node: {}", graph.start_node());
    println!("node_count: {}", graph.len());
    println!(
        "player_state_field_count: {}",
        player_state_schema.fields.len()
    );
    println!(
        "world_state_field_count: {}",
        world_state_schema.fields.len()
    );
    println!(
        "{}",
        language.text("player_state_schema:", "player_state_schema:")
    );
    println!("{}", serde_json::to_string_pretty(player_state_schema)?);
    println!(
        "{}",
        language.text("world_state_schema:", "world_state_schema:")
    );
    println!("{}", serde_json::to_string_pretty(world_state_schema)?);
    println!("{}", language.text("graph:", "graph:"));
    println!("{}", serde_json::to_string_pretty(graph)?);
    println!();
    Ok(())
}

pub fn print_planned_story(language: Language, story_script: &str) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        language.text("Planner 已生成剧本草案：", "Planner generated story draft:")
    );
    println!("{story_script}");
    println!();
    Ok(())
}

pub fn print_direct_story_summary(
    language: Language,
    graph: &StoryGraph,
) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        language.text(
            "已加载内置剧情图，跳过 StoryResources 和 Architect。",
            "Loaded built-in story graph, skipping StoryResources and Architect.",
        )
    );
    println!("start_node: {}", graph.start_node());
    println!("node_count: {}", graph.len());
    println!("{}", language.text("graph:", "graph:"));
    println!("{}", serde_json::to_string_pretty(graph)?);
    println!();
    Ok(())
}

pub async fn run_interactive_loop(
    engine: &mut Engine,
    language: Language,
) -> Result<(), Box<dyn Error>> {
    loop {
        let Some(player_input) = read_player_input(language)? else {
            println!("{}", language.text("已结束。", "Goodbye."));
            return Ok(());
        };

        if player_input.trim().is_empty() {
            continue;
        }

        let stream_result = engine.run_turn_stream(&player_input).await;
        let mut stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => {
                println!(
                    "{}: {error}",
                    language.text("回合启动失败", "Failed to start turn")
                );
                println!();
                continue;
            }
        };

        let mut printer = DeltaPrinter::default();
        let mut saw_terminal_event = false;

        while let Some(event) = stream.next().await {
            match event {
                EngineEvent::TurnStarted {
                    next_turn_index, ..
                } => {
                    println!(
                        "{} {next_turn_index}",
                        language.text("=== 回合", "=== Turn")
                    );
                }
                EngineEvent::PlayerInputRecorded { .. } => {
                    println!(
                        "{}",
                        language.text("已记录玩家输入。", "Player input recorded.")
                    );
                }
                EngineEvent::KeeperApplied {
                    phase,
                    update,
                    snapshot,
                } => {
                    printer.finish_line()?;
                    println!(
                        "{} {:?}, ops={}, node={}",
                        language.text("Keeper 已应用阶段", "Keeper applied phase"),
                        phase,
                        update.ops.len(),
                        snapshot.world_state.current_node
                    );
                }
                EngineEvent::DirectorCompleted { result, .. } => {
                    printer.finish_line()?;
                    println!(
                        "{} {} -> {}, beats={}",
                        language.text("Director 规划完成:", "Director completed:"),
                        result.previous_node_id,
                        result.current_node_id,
                        result.response_plan.beats.len()
                    );
                }
                EngineEvent::SessionCharacterCreated { character, .. } => {
                    printer.finish_line()?;
                    println!(
                        "{} {} ({})",
                        language.text("临时角色创建:", "Session character created:"),
                        character.display_name,
                        character.session_character_id
                    );
                }
                EngineEvent::SessionCharacterEnteredScene {
                    session_character_id,
                    ..
                } => {
                    printer.finish_line()?;
                    println!(
                        "{} {}",
                        language.text("临时角色进场:", "Session character entered scene:"),
                        session_character_id
                    );
                }
                EngineEvent::SessionCharacterLeftScene {
                    session_character_id,
                    ..
                } => {
                    printer.finish_line()?;
                    println!(
                        "{} {}",
                        language.text("临时角色离场:", "Session character left scene:"),
                        session_character_id
                    );
                }
                EngineEvent::NarratorStarted {
                    beat_index,
                    purpose,
                } => {
                    printer.finish_line()?;
                    println!(
                        "{} #{beat_index} {:?}",
                        language.text("Narrator 开始", "Narrator started"),
                        purpose
                    );
                }
                EngineEvent::NarratorTextDelta {
                    beat_index,
                    purpose,
                    delta,
                } => {
                    printer.push(
                        DeltaChannel::Narrator(beat_index),
                        format!("[narrator #{beat_index} {purpose:?}] "),
                        &delta,
                    )?;
                }
                EngineEvent::NarratorCompleted { .. } => {
                    printer.finish_line()?;
                }
                EngineEvent::ActorStarted {
                    beat_index,
                    speaker_id,
                    purpose,
                } => {
                    printer.finish_line()?;
                    println!(
                        "{} #{beat_index} {speaker_id} {:?}",
                        language.text("Actor 开始", "Actor started"),
                        purpose
                    );
                }
                EngineEvent::ActorThoughtDelta {
                    speaker_id, delta, ..
                } => {
                    printer.push(
                        DeltaChannel::ActorThought(speaker_id.clone()),
                        format!("[thought:{speaker_id}] "),
                        &delta,
                    )?;
                }
                EngineEvent::ActorActionComplete {
                    speaker_id, text, ..
                } => {
                    printer.finish_line()?;
                    println!("[action:{speaker_id}] {text}");
                }
                EngineEvent::ActorDialogueDelta {
                    speaker_id, delta, ..
                } => {
                    printer.push(
                        DeltaChannel::ActorDialogue(speaker_id.clone()),
                        format!("[dialogue:{speaker_id}] "),
                        &delta,
                    )?;
                }
                EngineEvent::ActorCompleted { .. } => {
                    printer.finish_line()?;
                }
                EngineEvent::TurnCompleted { result } => {
                    printer.finish_line()?;
                    println!(
                        "{} {}",
                        language.text("回合完成。turn_index=", "Turn completed. turn_index="),
                        result.turn_index
                    );
                    print_runtime_summary(
                        &result.snapshot,
                        language.text("回合结束状态", "End-of-turn state"),
                    )?;
                    println!();
                    saw_terminal_event = true;
                    break;
                }
                EngineEvent::TurnFailed {
                    stage,
                    error,
                    snapshot,
                } => {
                    printer.finish_line()?;
                    println!(
                        "{} {:?}: {error}",
                        language.text("回合失败于", "Turn failed at"),
                        stage
                    );
                    print_runtime_summary(
                        &snapshot,
                        language.text("失败时状态", "State at failure"),
                    )?;
                    println!();
                    saw_terminal_event = true;
                    break;
                }
            }
        }

        if !saw_terminal_event {
            return Err(io::Error::other(
                "engine stream ended without TurnCompleted or TurnFailed",
            )
            .into());
        }
    }
}

fn parse_language(value: &str) -> Result<Language, Box<dyn Error>> {
    match value {
        "zh" => Ok(Language::Zh),
        "en" => Ok(Language::En),
        _ => Err(io::Error::other(format!(
            "unsupported language '{value}', expected 'zh' or 'en'"
        ))
        .into()),
    }
}

fn print_usage_and_exit(allow_planner: bool) -> ! {
    if allow_planner {
        eprintln!("Usage: cargo run -p ss-engine --example <name> -- --lang <zh|en> [--planner]");
    } else {
        eprintln!("Usage: cargo run -p ss-engine --example <name> -- --lang <zh|en>");
    }
    std::process::exit(0);
}

fn require_env(name: &str) -> Result<String, Box<dyn Error>> {
    std::env::var(name)
        .map_err(|_| io::Error::other(format!("missing required environment variable: {name}")))
        .map_err(Into::into)
}

fn read_player_input(language: Language) -> Result<Option<String>, Box<dyn Error>> {
    print!("{}", language.text("你> ", "You> "));
    io::stdout().flush()?;

    let mut input = String::new();
    let read = io::stdin().read_line(&mut input)?;
    if read == 0 {
        return Ok(None);
    }

    let trimmed = input.trim();
    if matches!(trimmed, "exit" | "quit" | "退出") {
        return Ok(None);
    }

    Ok(Some(trimmed.to_owned()))
}

fn print_runtime_summary(snapshot: &RuntimeSnapshot, label: &str) -> Result<(), Box<dyn Error>> {
    let shared_tail: Vec<_> = snapshot
        .world_state
        .actor_shared_history
        .iter()
        .rev()
        .take(4)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    println!("{label}:");
    println!("  story_id: {}", snapshot.story_id);
    println!("  player_description: {}", snapshot.player_description);
    println!("  turn_index: {}", snapshot.turn_index);
    println!("  current_node: {}", snapshot.world_state.current_node);
    println!(
        "  active_characters: {:?}",
        snapshot.world_state.active_characters
    );
    println!(
        "  player_state: {}",
        serde_json::to_string_pretty(snapshot.world_state.player_states())?
    );
    println!(
        "  custom_state: {}",
        serde_json::to_string_pretty(&snapshot.world_state.custom)?
    );
    println!(
        "  character_state: {}",
        serde_json::to_string_pretty(&snapshot.world_state.character_state)?
    );
    println!(
        "  shared_history_tail: {}",
        serde_json::to_string_pretty(&shared_tail)?
    );
    Ok(())
}

fn print_character_roster(language: Language, character_cards: &[CharacterCard]) {
    println!("{}", language.text("可用角色：", "Available characters:"));
    for card in character_cards {
        println!("- {}", format_character_summary(language, card));
    }
    println!();
}

fn format_character_summary(language: Language, card: &CharacterCard) -> String {
    match language {
        Language::Zh => format!(
            "{}（{}）：{}；风格：{}",
            card.name, card.id, card.personality, card.style
        ),
        Language::En => format!(
            "{} ({}) - {}; style: {}",
            card.name, card.id, card.personality, card.style
        ),
    }
}

fn localized_character_cards(language: Language) -> Vec<CharacterCard> {
    vec![
        CharacterCard {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: language.text("精明但友善的行商", "A shrewd but friendly trader").to_owned(),
            style: language
                .text("健谈、随意、略带狡黠", "Talkative, casual, slightly cunning")
                .to_owned(),
            state_schema: HashMap::from([(
                "trust".to_owned(),
                StateFieldSchema::new(StateValueType::Int)
                    .with_default(json!(0))
                    .with_description(language.text(
                        "Haru 当前对玩家的信任程度",
                        "How much Haru currently trusts the player",
                    )),
            )]),
            system_prompt: language
                .text(
                    "你是旅行商人 Haru。保持角色沉浸感，发言自然，不要跳出故事。",
                    "You are Haru, a traveling merchant. Stay immersed and speak naturally in character.",
                )
                .to_owned(),
        },
        CharacterCard {
            id: "guide".to_owned(),
            name: "Yuki".to_owned(),
            personality: language
                .text("冷静、擅长观察细节的本地向导", "A calm local guide who notices small details")
                .to_owned(),
            style: language
                .text("克制、清晰、可靠", "Measured, clear, reassuring")
                .to_owned(),
            state_schema: HashMap::from([(
                "knows_safe_route".to_owned(),
                StateFieldSchema::new(StateValueType::Bool)
                    .with_default(json!(true))
                    .with_description(language.text(
                        "Yuki 是否知道穿过港口的安全路线",
                        "Whether Yuki knows a safe route through the harbor",
                    )),
            )]),
            system_prompt: language
                .text(
                    "你是本地向导 Yuki。保持观察敏锐、表达克制，并始终停留在角色中。",
                    "You are Yuki, a local guide. Stay observant, restrained, and fully in character.",
                )
                .to_owned(),
        },
        CharacterCard {
            id: "boatman".to_owned(),
            name: "Ren".to_owned(),
            personality: language
                .text("沉默寡言、经验老到的船夫", "A quiet, seasoned boatman with dry humor")
                .to_owned(),
            style: language
                .text("简短、务实、克制", "Brief, practical, understated")
                .to_owned(),
            state_schema: HashMap::from([(
                "knows_safe_route".to_owned(),
                StateFieldSchema::new(StateValueType::Bool)
                    .with_default(json!(false))
                    .with_description(language.text(
                        "Ren 是否知道靠近闸门的航道",
                        "Whether Ren knows the approach near the gate",
                    )),
            )]),
            system_prompt: language
                .text(
                    "你是老练的船夫 Ren。保持冷静和节制，不要跳出故事。",
                    "You are Ren, a seasoned boatman. Stay calm, restrained, and immersed in the story.",
                )
                .to_owned(),
        },
    ]
}

fn localized_player_state_schema(language: Language) -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int)
            .with_default(json!(8))
            .with_description(language.text(
                "玩家当前携带的钱币数量",
                "How many coins the player currently carries",
            )),
    );
    schema.insert_field(
        "dock_pass",
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description(language.text(
                "玩家是否持有码头通行证",
                "Whether the player already has a valid dock pass",
            )),
    );
    schema
}

fn localized_world_state_schema_seed(language: Language) -> WorldStateSchema {
    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description(language.text(
                "运河闸门是否已经打开",
                "Whether the canal flood gate has been opened",
            )),
    );
    schema.insert_field(
        "route_revealed",
        StateFieldSchema::new(StateValueType::Bool)
            .with_default(json!(false))
            .with_description(language.text(
                "安全路线是否已经被公开说明",
                "Whether the safe route has already been revealed",
            )),
    );
    schema
}
