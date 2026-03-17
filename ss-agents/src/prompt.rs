use std::collections::{BTreeMap, HashMap};

use serde::Serialize;
use serde_json::Value;
use state::schema::StateFieldSchema;
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};
use story::{Condition, ConditionOperator, ConditionScope, NarrativeNode, Transition};

use crate::actor::CharacterCardSummaryRef;

const DEFAULT_USER_NAME: &str = "User";

pub(crate) struct CharacterTemplateContext<'a> {
    pub(crate) character_name: &'a str,
    pub(crate) player_name: Option<&'a str>,
    pub(crate) state_schema: &'a HashMap<String, StateFieldSchema>,
    pub(crate) state_values: Option<&'a HashMap<String, Value>>,
}

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

pub(crate) fn render_character_text(text: &str, context: &CharacterTemplateContext<'_>) -> String {
    let mut rendered = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start_index) = remaining.find("{{") {
        rendered.push_str(&remaining[..start_index]);
        let after_start = &remaining[start_index + 2..];

        let Some(end_index) = after_start.find("}}") else {
            rendered.push_str(&remaining[start_index..]);
            return rendered;
        };

        let placeholder = &remaining[start_index..start_index + 2 + end_index + 2];
        let key = after_start[..end_index].trim();

        if let Some(value) = resolve_character_template_value(key, context) {
            rendered.push_str(&value);
        } else {
            rendered.push_str(placeholder);
        }

        remaining = &after_start[end_index + 2..];
    }

    rendered.push_str(remaining);
    rendered
}

pub(crate) fn render_character_summaries(
    summaries: &[CharacterCardSummaryRef<'_>],
    player_name: Option<&str>,
) -> String {
    render_list_lines(
        &summaries
            .iter()
            .map(|summary| {
                let template_context = CharacterTemplateContext {
                    character_name: summary.name,
                    player_name,
                    state_schema: summary.state_schema,
                    state_values: summary.state_values,
                };

                format!(
                    "{} | {} | personality={} | style={} | state_schema={}",
                    summary.id,
                    summary.name,
                    normalize_inline_text(&render_character_text(
                        summary.personality,
                        &template_context
                    )),
                    normalize_inline_text(&render_character_text(summary.style, &template_context)),
                    render_state_schema_fields(summary.state_schema),
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn render_player(name: Option<&str>, description: &str) -> String {
    render_sections(&[
        ("name", name.unwrap_or(DEFAULT_USER_NAME).to_owned()),
        ("description", description.to_owned()),
    ])
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
            let mut line = format!(
                "{key}:{}",
                compact_json(&field.value_type).unwrap_or_default()
            );
            if let Some(default) = &field.default {
                line.push_str(&format!(" default={}", compact_value(default)));
            }
            if let Some(enum_values) = &field.enum_values {
                line.push_str(&format!(
                    " enum={}",
                    compact_json(enum_values).unwrap_or_default()
                ));
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
    let transition_lines = render_list_lines(
        &node
            .transitions
            .iter()
            .map(render_transition)
            .collect::<Vec<_>>(),
    );

    render_sections(&[
        ("id", node.id.clone()),
        ("title", node.title.clone()),
        ("scene", node.scene.clone()),
        ("goal", node.goal.clone()),
        ("characters", node.characters.join(", ")),
        ("transitions", transition_lines),
    ])
}

pub(crate) fn render_optional_node(node: Option<&NarrativeNode>) -> String {
    node.map(render_node).unwrap_or_else(|| "null".to_owned())
}

pub(crate) fn render_keeper_node(node: &NarrativeNode) -> String {
    let transition_lines = if node.transitions.is_empty() {
        "- none".to_owned()
    } else {
        node.transitions
            .iter()
            .map(render_keeper_transition)
            .map(|line| format!("- {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    };

    render_sections(&[
        ("id", node.id.clone()),
        ("title", node.title.clone()),
        ("scene", node.scene.clone()),
        ("goal", node.goal.clone()),
        ("characters", node.characters.join(", ")),
        ("candidate_transitions", transition_lines),
    ])
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
        (
            "player_state",
            render_sorted_map(world_state.player_states()),
        ),
        (
            "character_state",
            render_character_state(&world_state.character_state),
        ),
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
        Some(condition) => format!(
            "to node={} when {}",
            transition.to,
            render_condition(condition)
        ),
        None => format!("to node={} when always", transition.to),
    }
}

fn render_condition(condition: &Condition) -> String {
    match condition.scope {
        ConditionScope::Global => format!(
            "global.{} {} {}",
            condition.key,
            render_condition_operator(&condition.op),
            compact_value(&condition.value)
        ),
        ConditionScope::Player => format!(
            "player.{} {} {}",
            condition.key,
            render_condition_operator(&condition.op),
            compact_value(&condition.value)
        ),
        ConditionScope::Character => format!(
            "character[{}].{} {} {}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key,
            render_condition_operator(&condition.op),
            compact_value(&condition.value)
        ),
    }
}

fn render_keeper_transition(transition: &Transition) -> String {
    match &transition.condition {
        Some(condition) => format!(
            "to node={} when {}",
            transition.to,
            render_keeper_condition(condition)
        ),
        None => format!("to node={} when always", transition.to),
    }
}

fn render_keeper_condition(condition: &Condition) -> String {
    let left = match condition.scope {
        ConditionScope::Global => format!("global.{}", condition.key),
        ConditionScope::Player => format!("player.{}", condition.key),
        ConditionScope::Character => format!(
            "character[{}].{}",
            condition.character.as_deref().unwrap_or("?"),
            condition.key
        ),
    };

    format!(
        "{left} {} {}",
        keeper_operator_symbol(&condition.op),
        compact_value(&condition.value)
    )
}

fn keeper_operator_symbol(operator: &ConditionOperator) -> &'static str {
    match operator {
        ConditionOperator::Eq => "==",
        ConditionOperator::Ne => "!=",
        ConditionOperator::Gt => ">",
        ConditionOperator::Gte => ">=",
        ConditionOperator::Lt => "<",
        ConditionOperator::Lte => "<=",
        ConditionOperator::Contains => "contains",
    }
}

fn render_condition_operator(operator: &story::ConditionOperator) -> &'static str {
    match operator {
        story::ConditionOperator::Eq => "==",
        story::ConditionOperator::Ne => "!=",
        story::ConditionOperator::Gt => ">",
        story::ConditionOperator::Gte => ">=",
        story::ConditionOperator::Lt => "<",
        story::ConditionOperator::Lte => "<=",
        story::ConditionOperator::Contains => "contains",
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
    character_state: Option<
        &std::collections::HashMap<String, std::collections::HashMap<String, Value>>,
    >,
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

fn resolve_character_template_value(
    key: &str,
    context: &CharacterTemplateContext<'_>,
) -> Option<String> {
    if key.is_empty() {
        return None;
    }

    match key {
        "char" => Some(context.character_name.to_owned()),
        "user" => Some(context.player_name.unwrap_or(DEFAULT_USER_NAME).to_owned()),
        _ => context
            .state_values
            .and_then(|state_values| state_values.get(key))
            .or_else(|| {
                context
                    .state_schema
                    .get(key)
                    .and_then(|field| field.default.as_ref())
            })
            .map(format_template_value),
    }
}

fn format_template_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Array(_) | Value::Object(_) | Value::Null => compact_value(value),
    }
}
