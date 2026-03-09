use serde::{Deserialize, Serialize};

use crate::transition::Transition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeNode {
    pub id: String,

    pub title: String,

    pub scene: String,

    pub goal: String,

    pub characters: Vec<String>,

    pub transitions: Vec<Transition>,
}

impl NarrativeNode {
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        scene: impl Into<String>,
        goal: impl Into<String>,
        characters: Vec<String>,
        transitions: Vec<Transition>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            scene: scene.into(),
            goal: goal.into(),
            characters,
            transitions,
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

    pub fn has_character(&self, character: &str) -> bool {
        self.characters.iter().any(|c| c == character)
    }

    pub fn is_terminal(&self) -> bool {
        self.transitions.is_empty()
    }
}
