use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;
use state::schema::StateFieldSchema;
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};
use story::{Condition, ConditionScope, NarrativeNode, Transition};

use crate::actor::CharacterCardSummaryRef;

pub(crate) fn compact_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(value)
}

pub(crate) fn render_sections(sections: &[(&str, String)]) -> String {
    sections
        .iter()
        .map(|(title, body)| format!("{title}:\n{}", body.trim_end()))
        .collect::<Vec<_>>()
        .join("\n\n")
}

pub(crate) fn render_list_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        return "- none".to_owned();
    }

    lines
        .iter()
        .map(|line| format!("- {line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn render_character_summaries(summaries: &[CharacterCardSummaryRef<'_>]) -> String {
    render_list_lines(
        &summaries
            .iter()
            .map(|summary| {
                let tendencies = if summary.tendencies.is_empty() {
                    "none".to_owned()
                } else {
                    summary.tendencies.join("; ")
                };

                format!(
                    "{} | {} | personality={} | style={} | tendencies={} | state_schema={}",
                    summary.id,
                    summary.name,
                    normalize_inline_text(summary.personality),
                    normalize_inline_text(summary.style),
                    normalize_inline_text(&tendencies),
                    render_state_schema_fields(summary.state_schema),
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn render_state_schema_fields(
    fields: &std::collections::HashMap<String, StateFieldSchema>,
) -> String {
    if fields.is_empty() {
        return "none".to_owned();
    }

    let ordered = fields
        .iter()
        .map(|(key, field)| (key.clone(), field))
        .collect::<BTreeMap<_, _>>();

    ordered
        .into_iter()
        .map(|(key, field)| {
            let mut line = format!("{key}:{}", compact_json(&field.value_type).unwrap_or_default());
            if let Some(default) = &field.default {
                line.push_str(&format!(" default={}", compact_value(default)));
            }
            if let Some(description) = &field.description {
                line.push_str(&format!(" desc={}", normalize_inline_text(description)));
            }
            line
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(crate) fn render_node(node: &NarrativeNode) -> String {
    let transition_lines = if node.transitions.is_empty() {
        "none".to_owned()
    } else {
        node.transitions
            .iter()
            .map(render_transition)
            .collect::<Vec<_>>()
            .join("; ")
    };
    let on_enter_updates = if node.on_enter_updates.is_empty() {
        "none".to_owned()
    } else {
        node.on_enter_updates
            .iter()
            .map(|op| compact_json(op).unwrap_or_else(|_| "<invalid>".to_owned()))
            .collect::<Vec<_>>()
            .join("; ")
    };

    render_sections(&[
        ("id", node.id.clone()),
        ("title", node.title.clone()),
        ("scene", node.scene.clone()),
        ("goal", node.goal.clone()),
        ("characters", node.characters.join(", ")),
        ("transitions", transition_lines),
        ("on_enter_updates", on_enter_updates),
    ])
}

pub(crate) fn render_optional_node(node: Option<&NarrativeNode>) -> String {
    node.map(render_node).unwrap_or_else(|| "null".to_owned())
}

pub(crate) fn render_actor_history(entries: &[ActorMemoryEntry]) -> String {
    if entries.is_empty() {
        return "- none".to_owned();
    }

    entries
        .iter()
        .map(|entry| {
            format!(
                "- [{}|{}|{}] {}",
                entry.speaker_id,
                entry.speaker_name,
                actor_memory_kind_label(entry.kind.clone()),
                normalize_inline_text(&entry.text)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) fn render_actor_world_state(world_state: &WorldState) -> String {
    render_world_state_sections(
        world_state.current_node(),
        world_state.active_characters(),
        &world_state.custom,
        None,
        Some(&world_state.character_state),
    )
}

pub(crate) fn render_director_world_state(world_state: &WorldState) -> String {
    render_world_state_sections(
        world_state.current_node(),
        world_state.active_characters(),
        &world_state.custom,
        Some(world_state.player_states()),
        Some(&world_state.character_state),
    )
}

pub(crate) fn render_observable_world_state(world_state: &WorldState) -> String {
    render_sections(&[
        ("current_node", world_state.current_node().to_owned()),
        (
            "active_characters",
            if world_state.active_characters().is_empty() {
                "none".to_owned()
            } else {
                world_state.active_characters().join(", ")
            },
        ),
        ("world_state", render_sorted_map(&world_state.custom)),
        ("player_state", render_sorted_map(world_state.player_states())),
        ("character_state", render_character_state(&world_state.character_state)),
        (
            "shared_history",
            render_actor_history(world_state.actor_shared_history()),
        ),
    ])
}

pub(crate) fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn render_transition(transition: &Transition) -> String {
    match &transition.condition {
        Some(condition) => format!("{} if {}", transition.to, render_condition(condition)),
        None => format!("{} if always", transition.to),
    }
}

fn render_condition(condition: &Condition) -> String {
    match condition.scope {
        ConditionScope::Global => format!(
            "world:{} {} {}",
            condition.key,
            compact_json(&condition.op).unwrap_or_default(),
            compact_value(&condition.value)
        ),
        ConditionScope::Player => format!(
            "player:{} {} {}",
            condition.key,
            compact_json(&condition.op).unwrap_or_default(),
            compact_value(&condition.value)
        ),
        ConditionScope::Character => format!(
            "character:{}:{} {} {}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key,
            compact_json(&condition.op).unwrap_or_default(),
            compact_value(&condition.value)
        ),
    }
}

fn actor_memory_kind_label(kind: ActorMemoryKind) -> &'static str {
    match kind {
        ActorMemoryKind::PlayerInput => "player_input",
        ActorMemoryKind::Dialogue => "dialogue",
        ActorMemoryKind::Thought => "thought",
        ActorMemoryKind::Action => "action",
    }
}

fn render_world_state_sections(
    current_node: &str,
    active_characters: &[String],
    custom: &std::collections::HashMap<String, Value>,
    player_state: Option<&std::collections::HashMap<String, Value>>,
    character_state: Option<&std::collections::HashMap<String, std::collections::HashMap<String, Value>>>,
) -> String {
    let mut sections = vec![
        ("current_node", current_node.to_owned()),
        (
            "active_characters",
            if active_characters.is_empty() {
                "none".to_owned()
            } else {
                active_characters.join(", ")
            },
        ),
        ("world_state", render_sorted_map(custom)),
    ];

    if let Some(player_state) = player_state {
        sections.push(("player_state", render_sorted_map(player_state)));
    }

    if let Some(character_state) = character_state {
        sections.push(("character_state", render_character_state(character_state)));
    }

    render_sections(&sections)
}

fn render_sorted_map(map: &std::collections::HashMap<String, Value>) -> String {
    if map.is_empty() {
        return "none".to_owned();
    }

    map.iter()
        .map(|(key, value)| (key.clone(), value))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(key, value)| format!("{key}={}", compact_value(value)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_character_state(
    map: &std::collections::HashMap<String, std::collections::HashMap<String, Value>>,
) -> String {
    if map.is_empty() {
        return "none".to_owned();
    }

    map.iter()
        .map(|(character, state)| (character.clone(), state))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .map(|(character, state)| format!("{character}: {}", render_sorted_map(state)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn compact_value(value: &Value) -> String {
    compact_json(value).unwrap_or_else(|_| "null".to_owned())
}
