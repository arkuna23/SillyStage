use std::sync::Arc;

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use state::schema::{PlayerStateSchema, WorldStateSchema};
use story::{NarrativeNode, StoryGraph, Transition};

/// Architect 的输入
#[derive(Debug, Clone, Copy)]
pub struct ArchitectRequest<'a> {
    pub story_concept: &'a str,
    pub planned_story: Option<&'a str>,
    pub world_state_schema: Option<&'a WorldStateSchema>,
    pub player_state_schema: Option<&'a PlayerStateSchema>,
    pub available_characters: &'a [CharacterCard],
}

/// Architect 的输出
#[derive(Debug, Clone, Serialize)]
pub struct ArchitectResponse {
    pub graph: StoryGraph,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
    pub output: llm::ChatResponse,
}

#[derive(Debug, Clone, Copy)]
pub struct ArchitectDraftInitRequest<'a> {
    pub story_concept: &'a str,
    pub planned_story: &'a str,
    pub current_section: &'a str,
    pub section_index: usize,
    pub total_sections: usize,
    pub graph_summary: &'a [GraphSummaryNode],
    pub recent_nodes: &'a [NarrativeNode],
    pub target_node_count: usize,
    pub world_state_schema: Option<&'a WorldStateSchema>,
    pub player_state_schema: Option<&'a PlayerStateSchema>,
    pub available_characters: &'a [CharacterCard],
}

#[derive(Debug, Clone, Copy)]
pub struct ArchitectDraftContinueRequest<'a> {
    pub story_concept: &'a str,
    pub planned_story: &'a str,
    pub current_section: &'a str,
    pub section_index: usize,
    pub total_sections: usize,
    pub graph_summary: &'a [GraphSummaryNode],
    pub recent_nodes: &'a [NarrativeNode],
    pub target_node_count: usize,
    pub world_state_schema: &'a WorldStateSchema,
    pub player_state_schema: &'a PlayerStateSchema,
    pub available_characters: &'a [CharacterCard],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphSummaryNode {
    pub id: String,
    pub title: String,
    pub scene_summary: String,
    pub goal: String,
    pub characters: Vec<String>,
    pub transition_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTransitionPatch {
    pub node_id: String,
    pub add_transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectDraftInitResponse {
    pub nodes: Vec<NarrativeNode>,
    pub transition_patches: Vec<NodeTransitionPatch>,
    pub start_node: String,
    pub world_state_schema: WorldStateSchema,
    pub player_state_schema: PlayerStateSchema,
    pub introduction: String,
    pub section_summary: String,
    pub output: llm::ChatResponse,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArchitectDraftChunkResponse {
    pub nodes: Vec<NarrativeNode>,
    pub transition_patches: Vec<NodeTransitionPatch>,
    pub section_summary: String,
    pub output: llm::ChatResponse,
}

#[derive(Debug, Clone, Deserialize)]
struct ArchitectOutputBundle {
    graph: StoryGraph,
    world_state_schema: WorldStateSchema,
    #[serde(default)]
    player_state_schema: Option<PlayerStateSchema>,
    introduction: String,
}

#[derive(Debug, Clone, Deserialize)]
struct ArchitectDraftOutputBundle {
    #[serde(default)]
    nodes: Vec<NarrativeNode>,
    #[serde(default)]
    transition_patches: Vec<NodeTransitionPatch>,
    #[serde(default)]
    start_node: Option<String>,
    #[serde(default)]
    world_state_schema: Option<WorldStateSchema>,
    #[serde(default)]
    player_state_schema: Option<PlayerStateSchema>,
    #[serde(default)]
    introduction: Option<String>,
    #[serde(default)]
    section_summary: Option<String>,
}

/// Architect agent
pub struct Architect {
    client: Arc<dyn LlmApi>,
    model: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Architect {
    pub fn new(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        Self::new_with_options(client, model, None, None)
    }

    pub fn new_with_options(
        client: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Self {
        Self {
            client,
            model: model.into(),
            temperature,
            max_tokens,
        }
    }

    pub async fn generate_graph(
        &self,
        req: ArchitectRequest<'_>,
    ) -> Result<ArchitectResponse, ArchitectError> {
        let user_prompt = self.build_user_prompt(&req)?;

        let output = self
            .client
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(self.model.clone())
                    .system_message(include_str!("./prompts/architect.txt"))
                    .user_message(user_prompt)
                    .response_format(llm::ResponseFormat::JsonObject);
                if let Some(temperature) = self.temperature {
                    builder = builder.temperature(temperature);
                }
                if let Some(max_tokens) = self.max_tokens {
                    builder = builder.max_tokens(max_tokens);
                }
                builder.build()?
            })
            .await?;

        let bundle: ArchitectOutputBundle = output
            .structured_output
            .as_ref()
            .ok_or_else(|| ArchitectError::MissingOutput)
            .and_then(|r| serde_json::from_value(r.clone()).map_err(ArchitectError::InvalidJson))?;
        let player_state_schema = bundle
            .player_state_schema
            .or_else(|| req.player_state_schema.cloned())
            .unwrap_or_default();

        Ok(ArchitectResponse {
            graph: bundle.graph,
            world_state_schema: bundle.world_state_schema,
            player_state_schema,
            introduction: bundle.introduction,
            output,
        })
    }

    pub async fn start_draft(
        &self,
        req: ArchitectDraftInitRequest<'_>,
    ) -> Result<ArchitectDraftInitResponse, ArchitectError> {
        let output = self
            .client
            .chat(self.build_draft_chat_request(
                ArchitectDraftMode::Init,
                DraftPromptInput {
                    story_concept: req.story_concept,
                    planned_story: req.planned_story,
                    current_section: req.current_section,
                    section_index: req.section_index,
                    total_sections: req.total_sections,
                    graph_summary: req.graph_summary,
                    recent_nodes: req.recent_nodes,
                    target_node_count: req.target_node_count,
                    world_state_schema: req.world_state_schema,
                    player_state_schema: req.player_state_schema,
                    available_characters: req.available_characters,
                },
            )?)
            .await?;

        let bundle: ArchitectDraftOutputBundle = output
            .structured_output
            .as_ref()
            .ok_or(ArchitectError::MissingOutput)
            .and_then(|value| {
                serde_json::from_value(value.clone()).map_err(ArchitectError::InvalidJson)
            })?;

        if bundle.nodes.is_empty() {
            return Err(ArchitectError::InvalidDraftOutput(
                "architect draft init returned no nodes".to_owned(),
            ));
        }

        Ok(ArchitectDraftInitResponse {
            nodes: bundle.nodes,
            transition_patches: bundle.transition_patches,
            start_node: bundle.start_node.ok_or_else(|| {
                ArchitectError::InvalidDraftOutput(
                    "architect draft init did not provide start_node".to_owned(),
                )
            })?,
            world_state_schema: bundle.world_state_schema.ok_or_else(|| {
                ArchitectError::InvalidDraftOutput(
                    "architect draft init did not provide world_state_schema".to_owned(),
                )
            })?,
            player_state_schema: bundle.player_state_schema.unwrap_or_default(),
            introduction: bundle.introduction.ok_or_else(|| {
                ArchitectError::InvalidDraftOutput(
                    "architect draft init did not provide introduction".to_owned(),
                )
            })?,
            section_summary: bundle.section_summary.ok_or_else(|| {
                ArchitectError::InvalidDraftOutput(
                    "architect draft init did not provide section_summary".to_owned(),
                )
            })?,
            output,
        })
    }

    pub async fn continue_draft(
        &self,
        req: ArchitectDraftContinueRequest<'_>,
    ) -> Result<ArchitectDraftChunkResponse, ArchitectError> {
        let output = self
            .client
            .chat(self.build_draft_chat_request(
                ArchitectDraftMode::Continue,
                DraftPromptInput {
                    story_concept: req.story_concept,
                    planned_story: req.planned_story,
                    current_section: req.current_section,
                    section_index: req.section_index,
                    total_sections: req.total_sections,
                    graph_summary: req.graph_summary,
                    recent_nodes: req.recent_nodes,
                    target_node_count: req.target_node_count,
                    world_state_schema: Some(req.world_state_schema),
                    player_state_schema: Some(req.player_state_schema),
                    available_characters: req.available_characters,
                },
            )?)
            .await?;

        let bundle: ArchitectDraftOutputBundle = output
            .structured_output
            .as_ref()
            .ok_or(ArchitectError::MissingOutput)
            .and_then(|value| {
                serde_json::from_value(value.clone()).map_err(ArchitectError::InvalidJson)
            })?;

        if bundle.nodes.is_empty() {
            return Err(ArchitectError::InvalidDraftOutput(
                "architect draft continue returned no nodes".to_owned(),
            ));
        }

        Ok(ArchitectDraftChunkResponse {
            nodes: bundle.nodes,
            transition_patches: bundle.transition_patches,
            section_summary: bundle.section_summary.ok_or_else(|| {
                ArchitectError::InvalidDraftOutput(
                    "architect draft continue did not provide section_summary".to_owned(),
                )
            })?,
            output,
        })
    }

    fn build_user_prompt(&self, req: &ArchitectRequest<'_>) -> Result<String, ArchitectError> {
        let world_schema_json = serde_json::to_string_pretty(&req.world_state_schema)
            .map_err(ArchitectError::SerializeSchema)?;
        let player_schema_json = serde_json::to_string_pretty(&req.player_state_schema)
            .map_err(ArchitectError::SerializePlayerSchema)?;
        let planned_story = req.planned_story.unwrap_or("null");

        let character_summaries: Vec<CharacterCardSummaryRef<'_>> = req
            .available_characters
            .iter()
            .map(CharacterCard::summary_ref)
            .collect();
        let characters_json = serde_json::to_string_pretty(&character_summaries)
            .map_err(ArchitectError::SerializeCharacters)?;

        Ok(format!(
            r#"STORY_CONCEPT:
{}

PLANNED_STORY:
{}

WORLD_STATE_SCHEMA_SEED:
{}

PLAYER_STATE_SCHEMA_SEED:
{}

AVAILABLE_CHARACTERS:
{}
"#,
            req.story_concept,
            planned_story,
            world_schema_json,
            player_schema_json,
            characters_json
        ))
    }

    fn build_draft_chat_request(
        &self,
        mode: ArchitectDraftMode,
        input: DraftPromptInput<'_>,
    ) -> Result<ChatRequest, ArchitectError> {
        let mut builder = ChatRequest::builder()
            .model(self.model.clone())
            .system_message(include_str!("./prompts/architect_draft.txt"))
            .user_message(self.build_draft_user_prompt(mode, input)?)
            .response_format(llm::ResponseFormat::JsonObject);
        if let Some(temperature) = self.temperature {
            builder = builder.temperature(temperature);
        }
        if let Some(max_tokens) = self.max_tokens {
            builder = builder.max_tokens(max_tokens);
        }
        builder.build().map_err(ArchitectError::from)
    }

    fn build_draft_user_prompt(
        &self,
        mode: ArchitectDraftMode,
        input: DraftPromptInput<'_>,
    ) -> Result<String, ArchitectError> {
        let graph_summary_json = serde_json::to_string_pretty(&input.graph_summary)
            .map_err(ArchitectError::SerializeGraphSummary)?;
        let recent_nodes_json = serde_json::to_string_pretty(&input.recent_nodes)
            .map_err(ArchitectError::SerializeRecentNodes)?;
        let world_schema_json = serde_json::to_string_pretty(&input.world_state_schema)
            .map_err(ArchitectError::SerializeSchema)?;
        let player_schema_json = serde_json::to_string_pretty(&input.player_state_schema)
            .map_err(ArchitectError::SerializePlayerSchema)?;
        let character_summaries: Vec<CharacterCardSummaryRef<'_>> = input
            .available_characters
            .iter()
            .map(CharacterCard::summary_ref)
            .collect();
        let characters_json = serde_json::to_string_pretty(&character_summaries)
            .map_err(ArchitectError::SerializeCharacters)?;

        Ok(format!(
            r#"MODE:
{}

STORY_CONCEPT:
{}

PLANNED_STORY:
{}

CURRENT_SECTION:
{}

SECTION_INDEX:
{}

TOTAL_SECTIONS:
{}

TARGET_NODE_COUNT:
{}

GRAPH_SUMMARY:
{}

RECENT_SECTION_NODES:
{}

WORLD_STATE_SCHEMA:
{}

PLAYER_STATE_SCHEMA:
{}

AVAILABLE_CHARACTERS:
{}
"#,
            mode.as_str(),
            input.story_concept,
            input.planned_story,
            input.current_section,
            input.section_index,
            input.total_sections,
            input.target_node_count,
            graph_summary_json,
            recent_nodes_json,
            world_schema_json,
            player_schema_json,
            characters_json
        ))
    }
}

#[derive(Debug, Clone, Copy)]
enum ArchitectDraftMode {
    Init,
    Continue,
}

impl ArchitectDraftMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Init => "init_draft",
            Self::Continue => "continue_draft",
        }
    }
}

#[derive(Clone, Copy)]
struct DraftPromptInput<'a> {
    story_concept: &'a str,
    planned_story: &'a str,
    current_section: &'a str,
    section_index: usize,
    total_sections: usize,
    graph_summary: &'a [GraphSummaryNode],
    recent_nodes: &'a [NarrativeNode],
    target_node_count: usize,
    world_state_schema: Option<&'a WorldStateSchema>,
    player_state_schema: Option<&'a PlayerStateSchema>,
    available_characters: &'a [CharacterCard],
}

/// Architect 错误类型
#[derive(Debug, thiserror::Error)]
pub enum ArchitectError {
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error(transparent)]
    InvalidJson(serde_json::Error),
    #[error(transparent)]
    SerializeSchema(serde_json::Error),
    #[error(transparent)]
    SerializePlayerSchema(serde_json::Error),
    #[error(transparent)]
    SerializeCharacters(serde_json::Error),
    #[error(transparent)]
    SerializeGraphSummary(serde_json::Error),
    #[error(transparent)]
    SerializeRecentNodes(serde_json::Error),
    #[error("missing json output")]
    MissingOutput,
    #[error("{0}")]
    InvalidDraftOutput(String),
}
