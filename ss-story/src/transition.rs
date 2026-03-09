use serde::{Deserialize, Serialize};

use crate::condition::Condition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub to: String,

    pub condition: Option<Condition>,
}

impl Transition {
    pub fn new(to: impl Into<String>, condition: Condition) -> Self {
        Self {
            to: to.into(),
            condition: Some(condition),
        }
    }

    pub fn unconditional(to: impl Into<String>) -> Self {
        Self {
            to: to.into(),
            condition: None,
        }
    }

    pub fn is_unconditional(&self) -> bool {
        self.condition.is_none()
    }
}
