use std::collections::HashMap;
use std::fs;
use std::path::Path;

use petgraph::dot::{Config, Dot};
use petgraph::graph::{Graph, NodeIndex};

use crate::condition::Condition;
use crate::graph::StoryGraph;
use crate::node::NarrativeNode;

#[derive(Debug, Clone)]
pub struct StoryEdge {
    pub condition: Option<Condition>,
}

#[derive(Debug)]
pub struct RuntimeStoryGraph {
    pub graph: Graph<NarrativeNode, StoryEdge>,
    pub start_node: NodeIndex,
    pub node_map: HashMap<String, NodeIndex>,
}

#[derive(Debug)]
pub enum GraphBuildError {
    MissingStartNode(String),
    MissingTargetNode { from: String, to: String },
    DuplicateNodeId(String),
}

fn dot_label_attr(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("label=\"{escaped}\"")
}

impl RuntimeStoryGraph {
    pub fn from_story_graph(story: StoryGraph) -> Result<Self, GraphBuildError> {
        let mut graph = Graph::<NarrativeNode, StoryEdge>::new();
        let mut node_map = HashMap::<String, NodeIndex>::new();

        for node in &story.nodes {
            if node_map.contains_key(&node.id) {
                return Err(GraphBuildError::DuplicateNodeId(node.id.clone()));
            }

            let index = graph.add_node(node.clone());
            node_map.insert(node.id.clone(), index);
        }

        for node in &story.nodes {
            let from_index = *node_map
                .get(&node.id)
                .expect("node index should exist after node insertion");

            for transition in &node.transitions {
                let to_index = node_map.get(&transition.to).copied().ok_or_else(|| {
                    GraphBuildError::MissingTargetNode {
                        from: node.id.clone(),
                        to: transition.to.clone(),
                    }
                })?;

                graph.add_edge(
                    from_index,
                    to_index,
                    StoryEdge {
                        condition: transition.condition.clone(),
                    },
                );
            }
        }

        let start_node = node_map
            .get(&story.start_node)
            .copied()
            .ok_or_else(|| GraphBuildError::MissingStartNode(story.start_node.clone()))?;

        Ok(Self {
            graph,
            start_node,
            node_map,
        })
    }

    pub fn get_node_index(&self, node_id: &str) -> Option<NodeIndex> {
        self.node_map.get(node_id).copied()
    }

    pub fn export_dot(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let dot = Dot::with_attr_getters(
            &self.graph,
            &[Config::EdgeNoLabel],
            &|_, edge| {
                edge.weight()
                    .condition
                    .as_ref()
                    .map(|cond| dot_label_attr(&format!("{cond:?}")))
                    .unwrap_or_default()
            },
            &|_, (_, node)| dot_label_attr(&node.id),
        );

        fs::write(path, format!("{dot:?}"))
    }
}
