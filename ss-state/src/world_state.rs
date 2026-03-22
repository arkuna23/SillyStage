use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::update::{StateOp, StateUpdate};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorMemoryKind {
    PlayerInput,
    Narration,
    Dialogue,
    Thought,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorMemoryEntry {
    pub speaker_id: String,
    pub speaker_name: String,
    pub kind: ActorMemoryKind,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WorldStatePromptView<'a> {
    current_node: &'a str,
    active_characters: &'a [String],
    custom: &'a HashMap<String, Value>,
    character_state: &'a HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectorWorldStateView<'a> {
    current_node: &'a str,
    active_characters: &'a [String],
    custom: &'a HashMap<String, Value>,
    player_state: &'a HashMap<String, Value>,
    character_state: &'a HashMap<String, HashMap<String, Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObservableWorldStateView<'a> {
    current_node: &'a str,
    active_characters: &'a [String],
    custom: &'a HashMap<String, Value>,
    player_state: &'a HashMap<String, Value>,
    character_state: &'a HashMap<String, HashMap<String, Value>>,
    actor_shared_history: &'a [ActorMemoryEntry],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub current_node: String,
    pub active_characters: Vec<String>,
    pub custom: HashMap<String, Value>,
    #[serde(default)]
    pub player_state: HashMap<String, Value>,
    #[serde(default)]
    pub character_state: HashMap<String, HashMap<String, Value>>,
    #[serde(default)]
    pub actor_shared_history: Vec<ActorMemoryEntry>,
    #[serde(default)]
    pub actor_private_memory: HashMap<String, Vec<ActorMemoryEntry>>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            current_node: "start".to_string(),
            active_characters: Vec::new(),
            custom: HashMap::new(),
            player_state: HashMap::new(),
            character_state: HashMap::new(),
            actor_shared_history: Vec::new(),
            actor_private_memory: HashMap::new(),
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

    pub fn with_player_state(mut self, player_state: HashMap<String, Value>) -> Self {
        self.player_state = player_state;
        self
    }

    pub fn with_actor_shared_history(mut self, history: Vec<ActorMemoryEntry>) -> Self {
        self.actor_shared_history = history;
        self
    }

    pub fn with_actor_private_memory(
        mut self,
        memory: HashMap<String, Vec<ActorMemoryEntry>>,
    ) -> Self {
        self.actor_private_memory = memory;
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

    pub fn player_states(&self) -> &HashMap<String, Value> {
        &self.player_state
    }

    pub fn player_state(&self, key: &str) -> Option<&Value> {
        self.player_state.get(key)
    }

    pub fn set_player_state(&mut self, key: impl Into<String>, value: Value) {
        self.player_state.insert(key.into(), value);
    }

    pub fn remove_player_state(&mut self, key: &str) -> Option<Value> {
        self.player_state.remove(key)
    }

    pub fn has_player_state(&self, key: &str) -> bool {
        self.player_state.contains_key(key)
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

    pub fn actor_shared_history(&self) -> &[ActorMemoryEntry] {
        &self.actor_shared_history
    }

    pub fn actor_private_memory(&self, character: &str) -> &[ActorMemoryEntry] {
        self.actor_private_memory
            .get(character)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn recent_actor_shared_history(&self, limit: usize) -> Vec<ActorMemoryEntry> {
        tail_entries(&self.actor_shared_history, limit)
    }

    pub fn recent_actor_private_memory(
        &self,
        character: &str,
        limit: usize,
    ) -> Vec<ActorMemoryEntry> {
        self.actor_private_memory
            .get(character)
            .map(|entries| tail_entries(entries, limit))
            .unwrap_or_default()
    }

    pub fn push_shared_memory(&mut self, entry: ActorMemoryEntry, limit: usize) {
        self.actor_shared_history.push(entry);
        trim_entries(&mut self.actor_shared_history, limit);
    }

    pub fn push_actor_shared_history(&mut self, entry: ActorMemoryEntry, limit: usize) {
        self.push_shared_memory(entry, limit);
    }

    pub fn push_player_input_shared_memory(&mut self, text: impl Into<String>, limit: usize) {
        self.push_shared_memory(
            ActorMemoryEntry {
                speaker_id: "player".to_owned(),
                speaker_name: "Player".to_owned(),
                kind: ActorMemoryKind::PlayerInput,
                text: text.into(),
            },
            limit,
        );
    }

    pub fn push_actor_private_memory(
        &mut self,
        character: impl Into<String>,
        entry: ActorMemoryEntry,
        limit: usize,
    ) {
        let entries = self
            .actor_private_memory
            .entry(character.into())
            .or_default();
        entries.push(entry);
        trim_entries(entries, limit);
    }

    pub fn clear_actor_shared_history(&mut self) {
        self.actor_shared_history.clear();
    }

    pub fn clear_actor_private_memory(&mut self, character: &str) {
        self.actor_private_memory.remove(character);
    }

    pub fn clear_all_actor_private_memory(&mut self) {
        self.actor_private_memory.clear();
    }

    pub fn clear_actor_memory(&mut self) {
        self.clear_actor_shared_history();
        self.clear_all_actor_private_memory();
    }

    pub fn without_actor_memory(&self) -> Self {
        let mut clone = self.clone();
        clone.clear_actor_memory();
        clone
    }

    pub fn actor_prompt_view(&self) -> WorldStatePromptView<'_> {
        WorldStatePromptView {
            current_node: &self.current_node,
            active_characters: &self.active_characters,
            custom: &self.custom,
            character_state: &self.character_state,
        }
    }

    pub fn prompt_view(&self) -> WorldStatePromptView<'_> {
        self.actor_prompt_view()
    }

    pub fn director_prompt_view(&self) -> DirectorWorldStateView<'_> {
        DirectorWorldStateView {
            current_node: &self.current_node,
            active_characters: &self.active_characters,
            custom: &self.custom,
            player_state: &self.player_state,
            character_state: &self.character_state,
        }
    }

    pub fn observable_prompt_view(&self) -> ObservableWorldStateView<'_> {
        ObservableWorldStateView {
            current_node: &self.current_node,
            active_characters: &self.active_characters,
            custom: &self.custom,
            player_state: &self.player_state,
            character_state: &self.character_state,
            actor_shared_history: &self.actor_shared_history,
        }
    }

    pub fn narrator_prompt_view(&self) -> ObservableWorldStateView<'_> {
        self.observable_prompt_view()
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
            StateOp::SetPlayerState { key, value } => {
                self.player_state.insert(key, value);
            }
            StateOp::RemovePlayerState { key } => {
                self.player_state.remove(&key);
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

fn trim_entries(entries: &mut Vec<ActorMemoryEntry>, limit: usize) {
    if limit == 0 {
        entries.clear();
        return;
    }

    if entries.len() > limit {
        let remove_count = entries.len() - limit;
        entries.drain(..remove_count);
    }
}

fn tail_entries(entries: &[ActorMemoryEntry], limit: usize) -> Vec<ActorMemoryEntry> {
    if limit == 0 {
        return Vec::new();
    }

    let start = entries.len().saturating_sub(limit);
    entries[start..].to_vec()
}
