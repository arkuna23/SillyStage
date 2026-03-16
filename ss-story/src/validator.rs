use crate::StoryGraph;
use state::StateOp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphValidationError {
    NonCanonicalIdentifierValue {
        key: String,
        value_repr: String,
        context: String,
    },
}

impl std::fmt::Display for GraphValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NonCanonicalIdentifierValue {
                key,
                value_repr,
                context,
            } => write!(
                f,
                "state key '{key}' must use a canonical snake_case identifier value, got {value_repr} in {context}"
            ),
        }
    }
}

impl std::error::Error for GraphValidationError {}

pub fn validate_graph_state_conventions(graph: &StoryGraph) -> Result<(), GraphValidationError> {
    for node in &graph.nodes {
        for transition in &node.transitions {
            if let Some(condition) = &transition.condition
                && is_identifier_like_key(&condition.key)
            {
                validate_identifier_value(
                    &condition.key,
                    &condition.value,
                    format!("transition from '{}' to '{}'", node.id, transition.to),
                )?;
            }
        }

        for update in &node.on_enter_updates {
            match update {
                StateOp::SetState { key, value }
                | StateOp::SetPlayerState { key, value }
                | StateOp::SetCharacterState { key, value, .. }
                    if is_identifier_like_key(key) =>
                {
                    validate_identifier_value(
                        key,
                        value,
                        format!("on_enter_updates of node '{}'", node.id),
                    )?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn validate_identifier_value(
    key: &str,
    value: &serde_json::Value,
    context: String,
) -> Result<(), GraphValidationError> {
    match value {
        serde_json::Value::String(value) if is_ascii_snake_case_identifier(value) => Ok(()),
        _ => Err(GraphValidationError::NonCanonicalIdentifierValue {
            key: key.to_owned(),
            value_repr: value.to_string(),
            context,
        }),
    }
}

fn is_identifier_like_key(key: &str) -> bool {
    matches!(
        key,
        "current_event" | "current_route" | "route" | "current_phase" | "travel_phase"
    )
}

fn is_ascii_snake_case_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use state::StateOp;

    use crate::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

    use super::validate_graph_state_conventions;

    #[test]
    fn accepts_canonical_identifier_like_values() {
        let graph = StoryGraph::new(
            "start",
            vec![
                NarrativeNode::new(
                    "start",
                    "Gate",
                    "The courier reaches the gate.",
                    "Open the story.",
                    vec![],
                    vec![Transition::new(
                        "next",
                        Condition::new(
                            "current_event",
                            ConditionOperator::Eq,
                            json!("approaching_swamp"),
                        ),
                    )],
                    vec![StateOp::SetState {
                        key: "current_event".to_owned(),
                        value: json!("approaching_swamp"),
                    }],
                ),
                NarrativeNode::new(
                    "next",
                    "Swamp Edge",
                    "The courier approaches the swamp.",
                    "Continue the route.",
                    vec![],
                    vec![],
                    vec![],
                ),
            ],
        );

        assert!(validate_graph_state_conventions(&graph).is_ok());
    }

    #[test]
    fn rejects_natural_language_identifier_like_values() {
        let graph = StoryGraph::new(
            "start",
            vec![NarrativeNode::new(
                "start",
                "Gate",
                "The courier reaches the gate.",
                "Open the story.",
                vec![],
                vec![],
                vec![StateOp::SetState {
                    key: "current_event".to_owned(),
                    value: json!("接近沼泽"),
                }],
            )],
        );

        let error = validate_graph_state_conventions(&graph)
            .expect_err("natural language value should fail");
        assert!(error.to_string().contains("current_event"));
        assert!(
            error
                .to_string()
                .contains("canonical snake_case identifier")
        );
    }
}
