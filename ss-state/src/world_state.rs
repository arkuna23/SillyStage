use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::update::{StateOp, StateUpdate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub current_node: String,
    pub active_characters: Vec<String>,
    pub custom: HashMap<String, Value>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_node: "start".to_string(),
            active_characters: Vec::new(),
            custom: HashMap::new(),
        }
    }
}

impl WorldState {
    pub fn new(current_node: impl Into<String>) -> Self {
        Self {
            current_node: current_node.into(),
            ..Default::default()
        }
    }

    pub fn with_active_characters(mut self, characters: Vec<String>) -> Self {
        self.active_characters = characters;
        self
    }

    pub fn with_custom(mut self, custom: HashMap<String, Value>) -> Self {
        self.custom = custom;
        self
    }

    pub fn current_node(&self) -> &str {
        &self.current_node
    }

    pub fn set_current_node(&mut self, node_id: impl Into<String>) {
        self.current_node = node_id.into();
    }

    pub fn active_characters(&self) -> &[String] {
        &self.active_characters
    }

    pub fn set_active_characters(&mut self, characters: Vec<String>) {
        self.active_characters = characters;
    }

    pub fn add_active_character(&mut self, character: impl Into<String>) {
        let character = character.into();
        if !self.active_characters.iter().any(|c| c == &character) {
            self.active_characters.push(character);
        }
    }

    pub fn remove_active_character(&mut self, character: &str) -> bool {
        if let Some(index) = self.active_characters.iter().position(|c| c == character) {
            self.active_characters.remove(index);
            true
        } else {
            false
        }
    }

    pub fn state(&self, key: &str) -> Option<&Value> {
        self.custom.get(key)
    }

    pub fn set_state(&mut self, key: impl Into<String>, value: Value) {
        self.custom.insert(key.into(), value);
    }

    pub fn remove_state(&mut self, key: &str) -> Option<Value> {
        self.custom.remove(key)
    }

    pub fn has_state(&self, key: &str) -> bool {
        self.custom.contains_key(key)
    }

    pub fn apply_update(&mut self, update: StateUpdate) {
        for op in update.ops {
            self.apply_op(op);
        }
    }

    pub fn apply_op(&mut self, op: StateOp) {
        match op {
            StateOp::SetCurrentNode { node_id } => {
                self.current_node = node_id;
            }
            StateOp::SetActiveCharacters { characters } => {
                self.active_characters = characters;
            }
            StateOp::AddActiveCharacter { character } => {
                if !self.active_characters.iter().any(|c| c == &character) {
                    self.active_characters.push(character);
                }
            }
            StateOp::RemoveActiveCharacter { character } => {
                if let Some(index) = self.active_characters.iter().position(|c| c == &character) {
                    self.active_characters.remove(index);
                }
            }
            StateOp::SetState { key, value } => {
                self.custom.insert(key, value);
            }
            StateOp::RemoveState { key } => {
                self.custom.remove(&key);
            }
        }
    }
}
