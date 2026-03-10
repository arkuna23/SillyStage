use std::collections::HashMap;

use agents::actor::CharacterCard;
use serde::{Deserialize, Serialize};
use state::{PlayerStateSchema, WorldState, WorldStateSchema};
use story::runtime_graph::{GraphBuildError, RuntimeStoryGraph};
use story::{NarrativeNode, StoryGraph};

pub(crate) struct RuntimeStatePartsMut<'a> {
    pub runtime_graph: &'a RuntimeStoryGraph,
    pub character_cards: &'a [CharacterCard],
    pub player_state_schema: &'a PlayerStateSchema,
    pub world_state: &'a mut WorldState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResources {
    story_id: String,
    story_concept: String,
    character_cards: Vec<CharacterCard>,
    player_state_schema: PlayerStateSchema,
    world_state_schema_seed: Option<WorldStateSchema>,
}

#[derive(Debug)]
pub struct RuntimeState {
    story_id: String,
    runtime_graph: RuntimeStoryGraph,
    character_cards: Vec<CharacterCard>,
    character_card_index: HashMap<String, usize>,
    player_state_schema: PlayerStateSchema,
    world_state: WorldState,
    turn_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub story_id: String,
    pub world_state: WorldState,
    pub turn_index: u64,
}

impl StoryResources {
    pub fn new(
        story_id: impl Into<String>,
        story_concept: impl Into<String>,
        character_cards: Vec<CharacterCard>,
        player_state_schema: PlayerStateSchema,
    ) -> Result<Self, RuntimeError> {
        let story_id = story_id.into();
        let story_concept = story_concept.into();
        validate_story_id(&story_id)?;
        validate_story_concept(&story_concept)?;
        validate_character_cards(&character_cards)?;

        Ok(Self {
            story_id,
            story_concept,
            character_cards,
            player_state_schema,
            world_state_schema_seed: None,
        })
    }

    pub fn with_world_state_schema_seed(mut self, world_state_schema: WorldStateSchema) -> Self {
        self.world_state_schema_seed = Some(world_state_schema);
        self
    }

    pub fn story_id(&self) -> &str {
        &self.story_id
    }

    pub fn story_concept(&self) -> &str {
        &self.story_concept
    }

    pub fn character_cards(&self) -> &[CharacterCard] {
        &self.character_cards
    }

    pub fn player_state_schema(&self) -> &PlayerStateSchema {
        &self.player_state_schema
    }

    pub fn world_state_schema_seed(&self) -> Option<&WorldStateSchema> {
        self.world_state_schema_seed.as_ref()
    }
}

impl RuntimeState {
    pub fn new(
        story_id: impl Into<String>,
        runtime_graph: RuntimeStoryGraph,
        character_cards: Vec<CharacterCard>,
        player_state_schema: PlayerStateSchema,
    ) -> Result<Self, RuntimeError> {
        let start_node = runtime_graph
            .graph
            .node_weight(runtime_graph.start_node)
            .expect("runtime graph start node should always exist");
        let world_state = WorldState::new(start_node.id.clone())
            .with_active_characters(start_node.characters.clone());

        Self::from_parts(
            story_id.into(),
            runtime_graph,
            character_cards,
            player_state_schema,
            world_state,
            0,
        )
    }

    pub fn from_story_graph(
        story_id: impl Into<String>,
        story_graph: StoryGraph,
        character_cards: Vec<CharacterCard>,
        player_state_schema: PlayerStateSchema,
    ) -> Result<Self, RuntimeError> {
        let runtime_graph =
            RuntimeStoryGraph::from_story_graph(story_graph).map_err(RuntimeError::GraphBuild)?;
        Self::new(
            story_id,
            runtime_graph,
            character_cards,
            player_state_schema,
        )
    }

    pub fn from_story_resources(
        resources: &StoryResources,
        story_graph: StoryGraph,
    ) -> Result<Self, RuntimeError> {
        Self::from_story_graph(
            resources.story_id(),
            story_graph,
            resources.character_cards().to_vec(),
            resources.player_state_schema().clone(),
        )
    }

    pub fn from_snapshot(
        story_id: impl Into<String>,
        runtime_graph: RuntimeStoryGraph,
        character_cards: Vec<CharacterCard>,
        player_state_schema: PlayerStateSchema,
        snapshot: RuntimeSnapshot,
    ) -> Result<Self, RuntimeError> {
        let story_id = story_id.into();
        if snapshot.story_id != story_id {
            return Err(RuntimeError::StoryIdMismatch {
                resource_story_id: story_id,
                snapshot_story_id: snapshot.story_id,
            });
        }

        Self::from_parts(
            snapshot.story_id,
            runtime_graph,
            character_cards,
            player_state_schema,
            snapshot.world_state,
            snapshot.turn_index,
        )
    }

    pub fn snapshot(&self) -> RuntimeSnapshot {
        RuntimeSnapshot {
            story_id: self.story_id.clone(),
            world_state: self.world_state.clone(),
            turn_index: self.turn_index,
        }
    }

    pub fn story_id(&self) -> &str {
        &self.story_id
    }

    pub fn runtime_graph(&self) -> &RuntimeStoryGraph {
        &self.runtime_graph
    }

    pub fn world_state(&self) -> &WorldState {
        &self.world_state
    }

    pub fn player_state_schema(&self) -> &PlayerStateSchema {
        &self.player_state_schema
    }

    pub fn world_state_mut(&mut self) -> &mut WorldState {
        &mut self.world_state
    }

    pub fn turn_index(&self) -> u64 {
        self.turn_index
    }

    pub fn advance_turn(&mut self) -> u64 {
        self.turn_index = self.turn_index.saturating_add(1);
        self.turn_index
    }

    pub fn character_cards(&self) -> &[CharacterCard] {
        &self.character_cards
    }

    pub fn character_card(&self, character_id: &str) -> Option<&CharacterCard> {
        self.character_card_index
            .get(character_id)
            .and_then(|index| self.character_cards.get(*index))
    }

    pub fn current_node(&self) -> Result<&NarrativeNode, RuntimeError> {
        let node_id = self.world_state.current_node();
        let node_index = self
            .runtime_graph
            .get_node_index(node_id)
            .ok_or_else(|| RuntimeError::MissingCurrentNode(node_id.to_owned()))?;

        self.runtime_graph
            .graph
            .node_weight(node_index)
            .ok_or_else(|| RuntimeError::MissingCurrentNode(node_id.to_owned()))
    }

    pub fn active_character_cards(&self) -> Result<Vec<&CharacterCard>, RuntimeError> {
        self.world_state
            .active_characters()
            .iter()
            .map(|character_id| {
                self.character_card(character_id)
                    .ok_or_else(|| RuntimeError::MissingCharacterCard(character_id.clone()))
            })
            .collect()
    }

    pub(crate) fn engine_parts(&mut self) -> RuntimeStatePartsMut<'_> {
        RuntimeStatePartsMut {
            runtime_graph: &self.runtime_graph,
            character_cards: &self.character_cards,
            player_state_schema: &self.player_state_schema,
            world_state: &mut self.world_state,
        }
    }

    fn from_parts(
        story_id: String,
        runtime_graph: RuntimeStoryGraph,
        character_cards: Vec<CharacterCard>,
        player_state_schema: PlayerStateSchema,
        world_state: WorldState,
        turn_index: u64,
    ) -> Result<Self, RuntimeError> {
        let character_card_index = build_character_card_index(&character_cards)?;
        validate_runtime_graph_characters(&runtime_graph, &character_card_index)?;
        validate_world_state(&world_state, &runtime_graph, &character_card_index)?;

        Ok(Self {
            story_id,
            runtime_graph,
            character_cards,
            character_card_index,
            player_state_schema,
            world_state,
            turn_index,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("failed to build runtime graph: {0:?}")]
    GraphBuild(GraphBuildError),
    #[error("story_id cannot be empty")]
    EmptyStoryId,
    #[error("story_concept cannot be empty")]
    EmptyStoryConcept,
    #[error("story resources requires at least one character card")]
    EmptyCharacterCards,
    #[error(
        "runtime snapshot story_id '{snapshot_story_id}' does not match resource story_id '{resource_story_id}'"
    )]
    StoryIdMismatch {
        resource_story_id: String,
        snapshot_story_id: String,
    },
    #[error("current node '{0}' not found in runtime graph")]
    MissingCurrentNode(String),
    #[error("missing character card for id '{0}'")]
    MissingCharacterCard(String),
    #[error("duplicate character card id '{0}'")]
    DuplicateCharacterCard(String),
}

fn validate_story_id(story_id: &str) -> Result<(), RuntimeError> {
    if story_id.trim().is_empty() {
        return Err(RuntimeError::EmptyStoryId);
    }

    Ok(())
}

fn validate_story_concept(story_concept: &str) -> Result<(), RuntimeError> {
    if story_concept.trim().is_empty() {
        return Err(RuntimeError::EmptyStoryConcept);
    }

    Ok(())
}

fn validate_character_cards(character_cards: &[CharacterCard]) -> Result<(), RuntimeError> {
    if character_cards.is_empty() {
        return Err(RuntimeError::EmptyCharacterCards);
    }

    build_character_card_index(character_cards).map(|_| ())
}

fn build_character_card_index(
    character_cards: &[CharacterCard],
) -> Result<HashMap<String, usize>, RuntimeError> {
    let mut index = HashMap::with_capacity(character_cards.len());

    for (position, card) in character_cards.iter().enumerate() {
        if index.insert(card.id.clone(), position).is_some() {
            return Err(RuntimeError::DuplicateCharacterCard(card.id.clone()));
        }
    }

    Ok(index)
}

fn validate_runtime_graph_characters(
    runtime_graph: &RuntimeStoryGraph,
    character_card_index: &HashMap<String, usize>,
) -> Result<(), RuntimeError> {
    for node in runtime_graph.graph.node_weights() {
        for character_id in &node.characters {
            if !character_card_index.contains_key(character_id) {
                return Err(RuntimeError::MissingCharacterCard(character_id.clone()));
            }
        }
    }

    Ok(())
}

fn validate_world_state(
    world_state: &WorldState,
    runtime_graph: &RuntimeStoryGraph,
    character_card_index: &HashMap<String, usize>,
) -> Result<(), RuntimeError> {
    if runtime_graph
        .get_node_index(world_state.current_node())
        .is_none()
    {
        return Err(RuntimeError::MissingCurrentNode(
            world_state.current_node().to_owned(),
        ));
    }

    for character_id in world_state.active_characters() {
        if !character_card_index.contains_key(character_id) {
            return Err(RuntimeError::MissingCharacterCard(character_id.clone()));
        }
    }

    Ok(())
}
