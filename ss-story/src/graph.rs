use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::node::NarrativeNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGraph {
    pub start_node: String,

    pub nodes: Vec<NarrativeNode>,
}

impl StoryGraph {
    pub fn new(start_node: impl Into<String>, nodes: Vec<NarrativeNode>) -> Self {
        Self {
            start_node: start_node.into(),
            nodes,
        }
    }

    pub fn start_node(&self) -> &str {
        &self.start_node
    }

    pub fn get_node(&self, node_id: &str) -> Option<&NarrativeNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut NarrativeNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn has_node(&self, node_id: &str) -> bool {
        self.nodes.iter().any(|node| node.id == node_id)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn node_ids(&self) -> Vec<&str> {
        self.nodes.iter().map(|node| node.id.as_str()).collect()
    }

    pub fn node_index_map(&self) -> HashMap<&str, usize> {
        self.nodes
            .iter()
            .enumerate()
            .map(|(idx, node)| (node.id.as_str(), idx))
            .collect()
    }
}
