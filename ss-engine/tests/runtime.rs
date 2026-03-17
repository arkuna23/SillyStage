use std::collections::HashMap;

use agents::actor::CharacterCard;
use serde_json::json;
use ss_engine::{RuntimeSnapshot, RuntimeState, StoryResources};
use state::{PlayerStateSchema, StateFieldSchema, StateOp, StateValueType, WorldStateSchema};
use story::{NarrativeNode, StoryGraph};

fn sample_character_cards() -> Vec<CharacterCard> {
    vec![
        CharacterCard {
            id: "merchant".to_owned(),
            name: "Haru".to_owned(),
            personality: "greedy but friendly trader".to_owned(),
            style: "talkative, casual".to_owned(),
            state_schema: HashMap::new(),
            system_prompt: "Stay in character.".to_owned(),
        },
        CharacterCard {
            id: "guide".to_owned(),
            name: "Yuki".to_owned(),
            personality: "calm local guide".to_owned(),
            style: "measured".to_owned(),
            state_schema: HashMap::new(),
            system_prompt: "Stay observant.".to_owned(),
        },
    ]
}

fn sample_story_graph() -> StoryGraph {
    StoryGraph::new(
        "dock",
        vec![
            NarrativeNode::new(
                "dock",
                "Flooded Dock",
                "A flooded dock at dusk.",
                "Decide whether to trust the guide.",
                vec!["merchant".to_owned(), "guide".to_owned()],
                vec![],
                vec![],
            ),
            NarrativeNode::new(
                "canal_gate",
                "Canal Gate",
                "A narrow ledge beside the gate.",
                "Open the route.",
                vec!["guide".to_owned()],
                vec![],
                vec![],
            ),
        ],
    )
}

fn sample_story_graph_with_start_on_enter_updates() -> StoryGraph {
    StoryGraph::new(
        "dock",
        vec![
            NarrativeNode::new(
                "dock",
                "Flooded Dock",
                "A flooded dock at dusk.",
                "Decide whether to trust the guide.",
                vec!["merchant".to_owned(), "guide".to_owned()],
                vec![],
                vec![
                    StateOp::SetState {
                        key: "entered_dock".to_owned(),
                        value: json!(true),
                    },
                    StateOp::SetPlayerState {
                        key: "coins".to_owned(),
                        value: json!(3),
                    },
                    StateOp::SetCharacterState {
                        character: "merchant".to_owned(),
                        key: "trust".to_owned(),
                        value: json!(1),
                    },
                ],
            ),
            NarrativeNode::new(
                "canal_gate",
                "Canal Gate",
                "A narrow ledge beside the gate.",
                "Open the route.",
                vec!["guide".to_owned()],
                vec![],
                vec![],
            ),
        ],
    )
}

fn sample_player_state_schema() -> PlayerStateSchema {
    let mut schema = PlayerStateSchema::new();
    schema.insert_field(
        "coins",
        StateFieldSchema::new(StateValueType::Int).with_default(json!(0)),
    );
    schema
}

fn sample_world_state_schema() -> WorldStateSchema {
    let mut schema = WorldStateSchema::new();
    schema.insert_field(
        "flood_gate_open",
        StateFieldSchema::new(StateValueType::Bool).with_default(json!(false)),
    );
    schema
}

fn sample_player_description() -> &'static str {
    "A cautious courier carrying medicine and trying not to attract attention."
}

fn sample_story_resources() -> StoryResources {
    StoryResources::new(
        "flooded_city_demo",
        "A flooded city courier story.",
        sample_character_cards(),
        Some(sample_player_state_schema()),
    )
    .expect("story resources should build")
    .with_world_state_schema_seed(sample_world_state_schema())
}

#[test]
fn story_resources_store_generation_inputs_and_seed() {
    let resources = sample_story_resources().with_planned_story(
        "Title:\nFlooded City Courier\n\nOpening Situation:\nThe courier arrives at the dock.",
    );

    assert_eq!(resources.story_id(), "flooded_city_demo");
    assert_eq!(resources.story_concept(), "A flooded city courier story.");
    assert_eq!(resources.character_cards().len(), 2);
    assert!(resources.planned_story().is_some());
    assert!(
        resources
            .player_state_schema_seed()
            .expect("player schema seed should exist")
            .has_field("coins")
    );
    assert!(
        resources
            .world_state_schema_seed()
            .expect("seed should exist")
            .has_field("flood_gate_open")
    );
}

#[test]
fn story_resources_reject_invalid_inputs() {
    let empty_story_id = StoryResources::new(
        "   ",
        "A flooded city courier story.",
        sample_character_cards(),
        Some(sample_player_state_schema()),
    )
    .expect_err("empty story_id should fail");
    assert!(empty_story_id.to_string().contains("story_id"));

    let empty_story_concept = StoryResources::new(
        "flooded_city_demo",
        "   ",
        sample_character_cards(),
        Some(sample_player_state_schema()),
    )
    .expect_err("empty story_concept should fail");
    assert!(empty_story_concept.to_string().contains("story_concept"));

    let empty_character_cards = StoryResources::new(
        "flooded_city_demo",
        "A flooded city courier story.",
        Vec::new(),
        Some(sample_player_state_schema()),
    )
    .expect_err("empty character cards should fail");
    assert!(empty_character_cards.to_string().contains("character card"));

    let duplicate_character_cards = StoryResources::new(
        "flooded_city_demo",
        "A flooded city courier story.",
        vec![
            sample_character_cards()[0].clone(),
            sample_character_cards()[0].clone(),
        ],
        Some(sample_player_state_schema()),
    )
    .expect_err("duplicate character cards should fail");
    assert!(duplicate_character_cards.to_string().contains("duplicate"));
}

#[test]
fn runtime_state_can_build_from_story_resources() {
    let resources = sample_story_resources();
    let runtime_state = RuntimeState::from_story_resources(
        &resources,
        sample_story_graph(),
        sample_player_description(),
        resources
            .player_state_schema_seed()
            .cloned()
            .expect("player schema seed should exist"),
    )
    .expect("runtime");

    assert_eq!(runtime_state.story_id(), resources.story_id());
    assert!(runtime_state.player_state_schema().has_field("coins"));
    assert_eq!(
        runtime_state.player_description(),
        sample_player_description()
    );
    assert_eq!(runtime_state.world_state().current_node(), "dock");
}

#[test]
fn new_initializes_from_start_node() {
    let runtime_state = RuntimeState::from_story_graph(
        "flooded_city_demo",
        sample_story_graph(),
        sample_character_cards(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect("runtime state should build");

    assert_eq!(runtime_state.story_id(), "flooded_city_demo");
    assert_eq!(runtime_state.turn_index(), 0);
    assert!(runtime_state.player_state_schema().has_field("coins"));
    assert_eq!(
        runtime_state.player_description(),
        sample_player_description()
    );
    assert_eq!(runtime_state.world_state().current_node(), "dock");
    assert_eq!(
        runtime_state.world_state().active_characters(),
        &["merchant".to_owned(), "guide".to_owned()]
    );
    assert_eq!(
        runtime_state
            .current_node()
            .expect("current node should exist")
            .title(),
        "Flooded Dock"
    );
}

#[test]
fn new_applies_start_node_on_enter_updates() {
    let runtime_state = RuntimeState::from_story_graph(
        "flooded_city_demo",
        sample_story_graph_with_start_on_enter_updates(),
        sample_character_cards(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect("runtime state should build");

    assert_eq!(
        runtime_state.world_state().state("entered_dock"),
        Some(&json!(true))
    );
    assert_eq!(
        runtime_state.world_state().player_state("coins"),
        Some(&json!(3))
    );
    assert_eq!(
        runtime_state
            .world_state()
            .character_states("merchant")
            .and_then(|state| state.get("trust")),
        Some(&json!(1))
    );
}

#[test]
fn active_character_cards_follow_world_state() {
    let mut runtime_state = RuntimeState::from_story_graph(
        "flooded_city_demo",
        sample_story_graph(),
        sample_character_cards(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect("runtime state should build");
    runtime_state
        .world_state_mut()
        .set_active_characters(vec!["guide".to_owned()]);

    let active_cards = runtime_state
        .active_character_cards()
        .expect("active character cards should resolve");

    assert_eq!(active_cards.len(), 1);
    assert_eq!(active_cards[0].id, "guide");
    assert_eq!(
        runtime_state
            .character_card("merchant")
            .expect("merchant card")
            .name,
        "Haru"
    );
}

#[test]
fn snapshot_round_trip_restores_dynamic_state_only() {
    let mut runtime_state = RuntimeState::from_story_graph(
        "flooded_city_demo",
        sample_story_graph(),
        sample_character_cards(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect("runtime state should build");
    runtime_state.set_player_description("A disguised courier posing as a dock clerk.");
    runtime_state
        .world_state_mut()
        .set_current_node("canal_gate".to_owned());
    runtime_state
        .world_state_mut()
        .set_active_characters(vec!["guide".to_owned()]);
    runtime_state
        .world_state_mut()
        .set_state("flood_gate_open", json!(true));
    runtime_state
        .world_state_mut()
        .set_player_state("coins", json!(7));
    runtime_state.advance_turn();
    runtime_state.advance_turn();

    let snapshot = runtime_state.snapshot();
    let serialized = serde_json::to_value(&snapshot).expect("snapshot should serialize");

    assert!(serialized.get("story_id").is_some());
    assert!(serialized.get("player_description").is_some());
    assert!(serialized.get("world_state").is_some());
    assert!(serialized.get("turn_index").is_some());
    assert!(serialized.get("runtime_graph").is_none());
    assert!(serialized.get("character_cards").is_none());

    let restored = RuntimeState::from_snapshot(
        "flooded_city_demo",
        story::runtime_graph::RuntimeStoryGraph::from_story_graph(sample_story_graph())
            .expect("runtime graph should build"),
        sample_character_cards(),
        sample_player_state_schema(),
        snapshot,
    )
    .expect("snapshot should restore");

    assert_eq!(restored.story_id(), "flooded_city_demo");
    assert_eq!(
        restored.player_description(),
        "A disguised courier posing as a dock clerk."
    );
    assert_eq!(restored.turn_index(), 2);
    assert_eq!(restored.world_state().current_node(), "canal_gate");
    assert_eq!(
        restored.world_state().state("flood_gate_open"),
        Some(&json!(true))
    );
    assert_eq!(
        restored.world_state().player_state("coins"),
        Some(&json!(7))
    );
    assert_eq!(
        restored
            .active_character_cards()
            .expect("active character cards should resolve")[0]
            .id,
        "guide"
    );
}

#[test]
fn from_snapshot_rejects_story_id_mismatch() {
    let snapshot = RuntimeSnapshot {
        story_id: "different_story".to_owned(),
        player_description: sample_player_description().to_owned(),
        world_state: state::WorldState::new("dock")
            .with_active_characters(vec!["merchant".to_owned(), "guide".to_owned()]),
        turn_index: 1,
    };

    let error = RuntimeState::from_snapshot(
        "flooded_city_demo",
        story::runtime_graph::RuntimeStoryGraph::from_story_graph(sample_story_graph())
            .expect("runtime graph should build"),
        sample_character_cards(),
        sample_player_state_schema(),
        snapshot,
    )
    .expect_err("story id mismatch should fail");

    assert!(error.to_string().contains("different_story"));
    assert!(error.to_string().contains("flooded_city_demo"));
}

#[test]
fn from_story_graph_rejects_missing_character_cards() {
    let error = RuntimeState::from_story_graph(
        "flooded_city_demo",
        sample_story_graph(),
        Vec::new(),
        sample_player_description(),
        sample_player_state_schema(),
    )
    .expect_err("missing character cards should fail");

    assert!(error.to_string().contains("merchant"));
}
