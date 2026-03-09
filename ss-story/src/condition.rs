use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::WorldState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    #[serde(default)]
    pub scope: ConditionScope,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character: Option<String>,
    pub key: String,
    pub op: ConditionOperator,
    pub value: Value,
}

impl Condition {
    pub fn new(key: impl Into<String>, op: ConditionOperator, value: Value) -> Self {
        Self {
            scope: ConditionScope::Global,
            character: None,
            key: key.into(),
            op,
            value,
        }
    }

    pub fn for_character(
        character: impl Into<String>,
        key: impl Into<String>,
        op: ConditionOperator,
        value: Value,
    ) -> Self {
        Self {
            scope: ConditionScope::Character,
            character: Some(character.into()),
            key: key.into(),
            op,
            value,
        }
    }

    pub fn matches(&self, world_state: &WorldState) -> bool {
        let Some(actual) = self.resolve_value(world_state) else {
            return false;
        };

        match self.op {
            ConditionOperator::Eq => actual == &self.value,
            ConditionOperator::Ne => actual != &self.value,
            ConditionOperator::Gt => {
                compare_numbers(actual, &self.value).is_some_and(|o| o.is_gt())
            }
            ConditionOperator::Gte => {
                compare_numbers(actual, &self.value).is_some_and(|o| o.is_gt() || o.is_eq())
            }
            ConditionOperator::Lt => {
                compare_numbers(actual, &self.value).is_some_and(|o| o.is_lt())
            }
            ConditionOperator::Lte => {
                compare_numbers(actual, &self.value).is_some_and(|o| o.is_lt() || o.is_eq())
            }
            ConditionOperator::Contains => contains_value(actual, &self.value),
        }
    }

    fn resolve_value<'a>(&self, world_state: &'a WorldState) -> Option<&'a Value> {
        match self.scope {
            ConditionScope::Global => world_state.state(&self.key),
            ConditionScope::Character => self
                .character
                .as_deref()
                .and_then(|character| world_state.character_state(character, &self.key)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConditionScope {
    #[default]
    Global,
    Character,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
}

fn compare_numbers(left: &Value, right: &Value) -> Option<std::cmp::Ordering> {
    let left = left.as_f64()?;
    let right = right.as_f64()?;
    left.partial_cmp(&right)
}

fn contains_value(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Array(values), _) => values.iter().any(|value| value == expected),
        (Value::String(actual), Value::String(expected)) => actual.contains(expected),
        (Value::Object(map), Value::String(expected)) => map.contains_key(expected),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use state::update::StateOp;

    use super::{Condition, ConditionOperator, ConditionScope};
    use state::WorldState;

    #[test]
    fn legacy_condition_json_defaults_to_global_scope() {
        let condition: Condition = serde_json::from_value(json!({
            "key": "trust_level",
            "op": "gte",
            "value": 2
        }))
        .expect("legacy condition should deserialize");

        assert_eq!(condition.scope, ConditionScope::Global);
        assert_eq!(condition.character, None);
    }

    #[test]
    fn global_condition_reads_global_state() {
        let mut world_state = WorldState::default();
        world_state.apply_op(StateOp::SetState {
            key: "trust_level".to_owned(),
            value: json!(3),
        });

        let condition = Condition::new("trust_level", ConditionOperator::Gte, json!(2));

        assert!(condition.matches(&world_state));
    }

    #[test]
    fn character_condition_reads_character_state() {
        let mut world_state = WorldState::default();
        world_state.apply_op(StateOp::SetCharacterState {
            character: "Haru".to_owned(),
            key: "trust".to_owned(),
            value: json!(4),
        });

        let condition = Condition::for_character("Haru", "trust", ConditionOperator::Gte, json!(3));

        assert!(condition.matches(&world_state));
    }

    #[test]
    fn missing_character_state_does_not_match() {
        let world_state = WorldState::default();
        let condition = Condition::for_character("Yuki", "trust", ConditionOperator::Eq, json!(1));

        assert!(!condition.matches(&world_state));
    }
}
