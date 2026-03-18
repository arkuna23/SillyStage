use crate::StoryGraph;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphValidationError {}

impl std::fmt::Display for GraphValidationError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}

impl std::error::Error for GraphValidationError {}

pub fn validate_graph_state_conventions(_: &StoryGraph) -> Result<(), GraphValidationError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use state::StateOp;

    use crate::{Condition, ConditionOperator, NarrativeNode, StoryGraph, Transition};

    use super::validate_graph_state_conventions;

    #[test]
    fn accepts_natural_language_identifier_like_values() {
        let graph = StoryGraph::new(
            "start",
            vec![NarrativeNode::new(
                "start",
                "Gate",
                "The courier reaches the gate.",
                "Open the story.",
                vec![],
                vec![Transition::new(
                    "next",
                    Condition::new("current_event", ConditionOperator::Eq, json!("Meeting Aqua")),
                )],
                vec![StateOp::SetState {
                    key: "current_event".to_owned(),
                    value: json!("Meeting Aqua"),
                }],
            )],
        );

        assert!(validate_graph_state_conventions(&graph).is_ok());
    }
}
