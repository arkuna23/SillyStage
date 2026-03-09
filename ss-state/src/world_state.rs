use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::update::{StateOp, StateUpdate};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub current_node: String,
    pub active_characters: Vec<String>,
    pub custom: HashMap<String, Value>,
    #[serde(default)]
    pub character_state: HashMap<String, HashMap<String, Value>>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_node: "start".to_string(),
            active_characters: Vec::new(),
            custom: HashMap::new(),
            character_state: HashMap::new(),
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

    pub fn with_character_state(
        mut self,
        character_state: HashMap<String, HashMap<String, Value>>,
    ) -> Self {
        self.character_state = character_state;
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

    pub fn character_states(&self, character: &str) -> Option<&HashMap<String, Value>> {
        self.character_state.get(character)
    }

    pub fn character_state(&self, character: &str, key: &str) -> Option<&Value> {
        self.character_state
            .get(character)
            .and_then(|state| state.get(key))
    }

    pub fn set_character_state(
        &mut self,
        character: impl Into<String>,
        key: impl Into<String>,
        value: Value,
    ) {
        let entry = self.character_state.entry(character.into()).or_default();
        entry.insert(key.into(), value);
    }

    pub fn remove_character_state(&mut self, character: &str, key: &str) -> Option<Value> {
        let removed = self
            .character_state
            .get_mut(character)
            .and_then(|state| state.remove(key));

        if self
            .character_state
            .get(character)
            .is_some_and(HashMap::is_empty)
        {
            self.character_state.remove(character);
        }

        removed
    }

    pub fn has_character_state(&self, character: &str, key: &str) -> bool {
        self.character_state
            .get(character)
            .is_some_and(|state| state.contains_key(key))
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
            StateOp::SetCharacterState {
                character,
                key,
                value,
            } => {
                self.set_character_state(character, key, value);
            }
            StateOp::RemoveCharacterState { character, key } => {
                self.remove_character_state(&character, &key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::WorldState;
    use crate::update::{StateOp, StateUpdate};

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
}
