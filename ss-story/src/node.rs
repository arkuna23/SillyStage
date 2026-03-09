use serde::{Deserialize, Serialize};

use crate::transition::Transition;
use state::update::StateOp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeNode {
    pub id: String,
    pub title: String,
    pub scene: String,
    pub goal: String,
    pub characters: Vec<String>,
    pub transitions: Vec<Transition>,

    #[serde(default)]
    pub on_enter_updates: Vec<StateOp>,
}

impl NarrativeNode {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        scene: impl Into<String>,
        goal: impl Into<String>,
        characters: Vec<String>,
        transitions: Vec<Transition>,
        on_enter_updates: Vec<StateOp>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            scene: scene.into(),
            goal: goal.into(),
            characters,
            transitions,
            on_enter_updates,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn scene(&self) -> &str {
        &self.scene
    }

    pub fn goal(&self) -> &str {
        &self.goal
    }

    pub fn characters(&self) -> &[String] {
        &self.characters
    }

    pub fn transitions(&self) -> &[Transition] {
        &self.transitions
    }

    pub fn on_enter_updates(&self) -> &[StateOp] {
        &self.on_enter_updates
    }

    pub fn has_character(&self, character: &str) -> bool {
        self.characters.iter().any(|c| c == character)
    }

    pub fn is_terminal(&self) -> bool {
        self.transitions.is_empty()
    }
}
