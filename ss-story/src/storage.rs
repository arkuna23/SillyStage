use std::fs;
use std::path::Path;

use crate::graph::StoryGraph;

pub fn load_from_str(json: &str) -> Result<StoryGraph, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn load_from_file(path: impl AsRef<Path>) -> Result<StoryGraph, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let graph = serde_json::from_str::<StoryGraph>(&content)?;
    Ok(graph)
}

pub fn save_to_string(graph: &StoryGraph) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(graph)
}

pub fn save_to_file(
    graph: &StoryGraph,
    path: impl AsRef<Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(graph)?;
    fs::write(path, json)?;
    Ok(())
}
