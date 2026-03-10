use serde_json::json;
use ss_state::{ActorMemoryEntry, ActorMemoryKind, StateOp, StateUpdate, WorldState};

#[test]
fn character_state_round_trip_works() {
    let mut state = WorldState::default();

    state.set_character_state("Haru", "trust", json!(3));

    assert_eq!(state.character_state("Haru", "trust"), Some(&json!(3)));
    assert!(state.has_character_state("Haru", "trust"));
}

#[test]
fn removing_last_character_field_cleans_up_character_map() {
    let mut state = WorldState::default();
    state.set_character_state("Haru", "trust", json!(3));

    assert_eq!(
        state.remove_character_state("Haru", "trust"),
        Some(json!(3))
    );
    assert_eq!(state.character_states("Haru"), None);
}

#[test]
fn apply_update_supports_character_state_ops() {
    let mut state = WorldState::default();
    let update = StateUpdate::new()
        .push(StateOp::SetCharacterState {
            character: "Yuki".to_owned(),
            key: "mood".to_owned(),
            value: json!("curious"),
        })
        .push(StateOp::RemoveCharacterState {
            character: "Yuki".to_owned(),
            key: "mood".to_owned(),
        });

    state.apply_update(update);

    assert_eq!(state.character_states("Yuki"), None);
}

#[test]
fn actor_shared_history_respects_limit() {
    let mut state = WorldState::default();

    for index in 0..3 {
        state.push_shared_memory(
            ActorMemoryEntry {
                speaker_id: "merchant".to_owned(),
                speaker_name: "Old Merchant".to_owned(),
                kind: ActorMemoryKind::Dialogue,
                text: format!("line {index}"),
            },
            2,
        );
    }

    assert_eq!(state.actor_shared_history().len(), 2);
    assert_eq!(state.actor_shared_history()[0].text, "line 1");
    assert_eq!(state.actor_shared_history()[1].text, "line 2");
}

#[test]
fn player_input_shared_memory_uses_visible_player_entry() {
    let mut state = WorldState::default();

    state.push_player_input_shared_memory("Open the flood gate.", 4);

    assert_eq!(state.actor_shared_history().len(), 1);
    assert_eq!(state.actor_shared_history()[0].speaker_id, "player");
    assert_eq!(state.actor_shared_history()[0].speaker_name, "Player");
    assert_eq!(
        state.actor_shared_history()[0].kind,
        ActorMemoryKind::PlayerInput
    );
    assert_eq!(state.actor_shared_history()[0].text, "Open the flood gate.");
}

#[test]
fn actor_private_memory_respects_limit_per_character() {
    let mut state = WorldState::default();

    for index in 0..3 {
        state.push_actor_private_memory(
            "merchant",
            ActorMemoryEntry {
                speaker_id: "merchant".to_owned(),
                speaker_name: "Old Merchant".to_owned(),
                kind: ActorMemoryKind::Thought,
                text: format!("thought {index}"),
            },
            2,
        );
    }

    assert_eq!(state.actor_private_memory("merchant").len(), 2);
    assert_eq!(state.actor_private_memory("merchant")[0].text, "thought 1");
    assert_eq!(state.actor_private_memory("merchant")[1].text, "thought 2");
}

#[test]
fn without_actor_memory_clears_hidden_memory_only() {
    let mut state = WorldState::new("dock");
    state.set_state("flood_gate_open", json!(false));
    state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Dialogue,
            text: "Deal?".to_owned(),
        },
        4,
    );
    state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "Maybe this works.".to_owned(),
        },
        4,
    );

    let sanitized = state.without_actor_memory();

    assert_eq!(sanitized.current_node, "dock");
    assert_eq!(sanitized.state("flood_gate_open"), Some(&json!(false)));
    assert!(sanitized.actor_shared_history().is_empty());
    assert!(sanitized.actor_private_memory("merchant").is_empty());
}

#[test]
fn observable_prompt_view_keeps_shared_history_but_hides_private_memory() {
    let mut state = WorldState::new("dock");
    state.set_state("flood_gate_open", json!(false));
    state.push_actor_shared_history(
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::PlayerInput,
            text: "The dock is still holding.".to_owned(),
        },
        4,
    );
    state.push_actor_private_memory(
        "merchant",
        ActorMemoryEntry {
            speaker_id: "merchant".to_owned(),
            speaker_name: "Old Merchant".to_owned(),
            kind: ActorMemoryKind::Thought,
            text: "I should keep the shortcut secret.".to_owned(),
        },
        4,
    );

    let serialized = serde_json::to_value(state.observable_prompt_view())
        .expect("observable view should serialize");

    assert!(serialized.get("actor_shared_history").is_some());
    assert!(serialized.get("character_state").is_some());
    assert!(serialized.get("actor_private_memory").is_none());
}
