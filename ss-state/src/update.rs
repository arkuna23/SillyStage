use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateUpdate {
    pub ops: Vec<StateOp>,
}

impl StateUpdate {
    pub fn new() -> Self {
        Self { ops: Vec::new() }
    }

    pub fn push(mut self, op: StateOp) -> Self {
        self.ops.push(op);
        self
    }

    pub fn add_op(&mut self, op: StateOp) {
        self.ops.push(op);
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StateOp {
    SetCurrentNode {
        node_id: String,
    },

    SetActiveCharacters {
        characters: Vec<String>,
    },

    AddActiveCharacter {
        character: String,
    },

    RemoveActiveCharacter {
        character: String,
    },

    SetState {
        key: String,
        value: Value,
    },

    RemoveState {
        key: String,
    },

    SetCharacterState {
        character: String,
        key: String,
        value: Value,
    },

    RemoveCharacterState {
        character: String,
        key: String,
    },
}
