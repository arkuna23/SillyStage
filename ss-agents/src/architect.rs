use std::collections::BTreeMap;
use std::sync::Arc;

use crate::actor::CharacterCard;
use llm::{ChatRequest, LlmApi};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::StateOp;
use state::schema::{PlayerStateSchema, StateFieldSchema, WorldStateSchema};
use story::{NarrativeNode, StoryGraph, Transition};
use tracing::{error, warn};

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
    pub current_section: &'a str,
    pub section_index: usize,
    pub total_sections: usize,
    pub section_summaries: &'a [String],
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

#[derive(Debug, Clone, Serialize)]
struct ArchitectCharacterSummary {
    id: String,
    name: String,
    role_summary: String,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    state_schema_keys: BTreeMap<String, CompactStateField>,
}

#[derive(Debug, Clone, Serialize)]
struct CompactStateField {
    value_type: state::StateValueType,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
struct RecentSectionDetailNode {
    id: String,
    title: String,
    scene_summary: String,
    goal: String,
    characters: Vec<String>,
    transition_targets: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    on_enter_update_keys: Vec<String>,
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
        let (bundle, output) = self
            .chat_json_with_repair(
                include_str!("./prompts/architect.txt"),
                user_prompt,
                ArchitectRepairTarget::FullGraph,
                Self::parse_json_output::<ArchitectOutputBundle>,
                |_| Ok(()),
            )
            .await?;
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
        let chat_request = self.build_draft_chat_request(
            DraftPromptKind::Init,
            DraftPromptInput {
                story_concept: req.story_concept,
                planned_story: Some(req.planned_story),
                current_section: req.current_section,
                section_index: req.section_index,
                total_sections: req.total_sections,
                section_summaries: &[],
                graph_summary: req.graph_summary,
                recent_nodes: req.recent_nodes,
                target_node_count: req.target_node_count,
                world_state_schema: req.world_state_schema,
                player_state_schema: req.player_state_schema,
                available_characters: req.available_characters,
            },
        )?;
        let (bundle, output) = self
            .chat_json_with_repair(
                include_str!("./prompts/architect_draft_init.txt"),
                chat_request.messages[1].content.clone(),
                ArchitectRepairTarget::DraftInit,
                Self::parse_json_output::<ArchitectDraftOutputBundle>,
                validate_draft_init_bundle,
            )
            .await?;

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
        let chat_request = self.build_draft_chat_request(
            DraftPromptKind::Continue,
            DraftPromptInput {
                story_concept: req.story_concept,
                planned_story: None,
                current_section: req.current_section,
                section_index: req.section_index,
                total_sections: req.total_sections,
                section_summaries: req.section_summaries,
                graph_summary: req.graph_summary,
                recent_nodes: req.recent_nodes,
                target_node_count: req.target_node_count,
                world_state_schema: Some(req.world_state_schema),
                player_state_schema: Some(req.player_state_schema),
                available_characters: req.available_characters,
            },
        )?;
        let (bundle, output) = self
            .chat_json_with_repair(
                include_str!("./prompts/architect_draft_continue.txt"),
                chat_request.messages[1].content.clone(),
                ArchitectRepairTarget::DraftContinue,
                Self::parse_json_output::<ArchitectDraftOutputBundle>,
                validate_draft_continue_bundle,
            )
            .await?;

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
        let world_schema_json = serde_json::to_string_pretty(&compact_schema_map(
            req.world_state_schema.map(|schema| &schema.fields),
        ))
        .map_err(ArchitectError::SerializeSchema)?;
        let player_schema_json = serde_json::to_string_pretty(&compact_schema_map(
            req.player_state_schema.map(|schema| &schema.fields),
        ))
        .map_err(ArchitectError::SerializePlayerSchema)?;
        let planned_story = req.planned_story.unwrap_or("null");

        let character_summaries: Vec<ArchitectCharacterSummary> = req
            .available_characters
            .iter()
            .map(compact_character_summary)
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
        kind: DraftPromptKind,
        input: DraftPromptInput<'_>,
    ) -> Result<ChatRequest, ArchitectError> {
        let mut builder = ChatRequest::builder()
            .model(self.model.clone())
            .system_message(kind.system_prompt())
            .user_message(self.build_draft_user_prompt(kind, input)?)
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
        kind: DraftPromptKind,
        input: DraftPromptInput<'_>,
    ) -> Result<String, ArchitectError> {
        let graph_summary_json = serde_json::to_string_pretty(&input.graph_summary)
            .map_err(ArchitectError::SerializeGraphSummary)?;
        let section_summaries_json = serde_json::to_string_pretty(&input.section_summaries)
            .map_err(ArchitectError::SerializeSectionSummaries)?;
        let recent_nodes_json = serde_json::to_string_pretty(
            &input
                .recent_nodes
                .iter()
                .map(compact_recent_section_node)
                .collect::<Vec<_>>(),
        )
        .map_err(ArchitectError::SerializeRecentNodes)?;
        let world_schema_json = serde_json::to_string_pretty(&compact_schema_map(
            input.world_state_schema.map(|schema| &schema.fields),
        ))
        .map_err(ArchitectError::SerializeSchema)?;
        let player_schema_json = serde_json::to_string_pretty(&compact_schema_map(
            input.player_state_schema.map(|schema| &schema.fields),
        ))
        .map_err(ArchitectError::SerializePlayerSchema)?;
        let character_summaries: Vec<ArchitectCharacterSummary> = input
            .available_characters
            .iter()
            .map(compact_character_summary)
            .collect();
        let characters_json = serde_json::to_string_pretty(&character_summaries)
            .map_err(ArchitectError::SerializeCharacters)?;

        match kind {
            DraftPromptKind::Init => Ok(format!(
                r#"STORY_CONCEPT:
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

RECENT_SECTION_DETAIL:
{}

WORLD_STATE_SCHEMA_SEED:
{}

PLAYER_STATE_SCHEMA_SEED:
{}

AVAILABLE_CHARACTERS:
{}
"#,
                input.story_concept,
                input.planned_story.unwrap_or("null"),
                input.current_section,
                input.section_index,
                input.total_sections,
                input.target_node_count,
                graph_summary_json,
                recent_nodes_json,
                world_schema_json,
                player_schema_json,
                characters_json
            )),
            DraftPromptKind::Continue => Ok(format!(
                r#"STORY_CONCEPT:
{}

CURRENT_SECTION:
{}

SECTION_INDEX:
{}

TOTAL_SECTIONS:
{}

SECTION_SUMMARIES:
{}

TARGET_NODE_COUNT:
{}

GRAPH_SUMMARY:
{}

RECENT_SECTION_DETAIL:
{}

WORLD_STATE_SCHEMA:
{}

PLAYER_STATE_SCHEMA:
{}

AVAILABLE_CHARACTERS:
{}
"#,
                input.story_concept,
                input.current_section,
                input.section_index,
                input.total_sections,
                section_summaries_json,
                input.target_node_count,
                graph_summary_json,
                recent_nodes_json,
                world_schema_json,
                player_schema_json,
                characters_json
            )),
        }
    }

    async fn chat_json_with_repair<T, P, V>(
        &self,
        system_prompt: &str,
        user_prompt: String,
        repair_target: ArchitectRepairTarget,
        parse: P,
        validate: V,
    ) -> Result<(T, llm::ChatResponse), ArchitectError>
    where
        T: DeserializeOwned,
        P: Fn(&llm::ChatResponse) -> Result<T, ArchitectError>,
        V: Fn(&T) -> Result<(), ArchitectError>,
    {
        match self.chat_json(system_prompt, user_prompt.clone()).await {
            Ok(output) => match parse(&output).and_then(|parsed| {
                validate(&parsed)?;
                Ok(parsed)
            }) {
                Ok(parsed) => Ok((parsed, output)),
                Err(error) => {
                    warn!(
                        target = "architect",
                        mode = %repair_target.as_str(),
                        error = %error,
                        raw_output = %output.message.content,
                        "architect output failed validation, attempting repair"
                    );
                    self.repair_and_parse(
                        repair_target,
                        &output.message.content,
                        &error,
                        parse,
                        validate,
                    )
                    .await
                }
            },
            Err(ArchitectError::Llm(llm::LlmError::StructuredOutputParse {
                message,
                raw_content,
            })) => {
                error!(
                    target = "architect",
                    mode = %repair_target.as_str(),
                    error = %message,
                    raw_output = %raw_content,
                    "architect returned invalid json, attempting repair"
                );
                self.repair_and_parse(
                    repair_target,
                    &raw_content,
                    &ArchitectError::InvalidJson(serde_json::Error::io(std::io::Error::other(
                        message,
                    ))),
                    parse,
                    validate,
                )
                .await
            }
            Err(error) => Err(error),
        }
    }

    async fn repair_and_parse<T, P, V>(
        &self,
        repair_target: ArchitectRepairTarget,
        raw_output: &str,
        original_error: &ArchitectError,
        parse: P,
        validate: V,
    ) -> Result<(T, llm::ChatResponse), ArchitectError>
    where
        T: DeserializeOwned,
        P: Fn(&llm::ChatResponse) -> Result<T, ArchitectError>,
        V: Fn(&T) -> Result<(), ArchitectError>,
    {
        let repair_prompt = build_repair_prompt(repair_target, raw_output, original_error);
        let repaired = self
            .chat_json(
                include_str!("./prompts/architect_repair.txt"),
                repair_prompt,
            )
            .await?;
        let parsed = parse(&repaired)?;
        validate(&parsed)?;
        Ok((parsed, repaired))
    }

    async fn chat_json(
        &self,
        system_prompt: &str,
        user_prompt: String,
    ) -> Result<llm::ChatResponse, ArchitectError> {
        let mut builder = ChatRequest::builder()
            .model(self.model.clone())
            .system_message(system_prompt)
            .user_message(user_prompt)
            .response_format(llm::ResponseFormat::JsonObject);
        if let Some(temperature) = self.temperature {
            builder = builder.temperature(temperature);
        }
        if let Some(max_tokens) = self.max_tokens {
            builder = builder.max_tokens(max_tokens);
        }
        self.client
            .chat(builder.build()?)
            .await
            .map_err(ArchitectError::from)
    }

    fn parse_json_output<T: DeserializeOwned>(
        output: &llm::ChatResponse,
    ) -> Result<T, ArchitectError> {
        if let Some(structured_output) = &output.structured_output {
            return serde_json::from_value(structured_output.clone())
                .map_err(ArchitectError::InvalidJson);
        }

        serde_json::from_str(&output.message.content).map_err(ArchitectError::InvalidJson)
    }
}

#[derive(Debug, Clone, Copy)]
enum DraftPromptKind {
    Init,
    Continue,
}

impl DraftPromptKind {
    const fn system_prompt(self) -> &'static str {
        match self {
            Self::Init => include_str!("./prompts/architect_draft_init.txt"),
            Self::Continue => include_str!("./prompts/architect_draft_continue.txt"),
        }
    }
}

#[derive(Clone, Copy)]
struct DraftPromptInput<'a> {
    story_concept: &'a str,
    planned_story: Option<&'a str>,
    current_section: &'a str,
    section_index: usize,
    total_sections: usize,
    section_summaries: &'a [String],
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
    SerializeSectionSummaries(serde_json::Error),
    #[error(transparent)]
    SerializeRecentNodes(serde_json::Error),
    #[error("missing json output")]
    MissingOutput,
    #[error("{0}")]
    InvalidDraftOutput(String),
}

#[derive(Debug, Clone, Copy)]
enum ArchitectRepairTarget {
    FullGraph,
    DraftInit,
    DraftContinue,
}

impl ArchitectRepairTarget {
    const fn as_str(self) -> &'static str {
        match self {
            Self::FullGraph => "full_graph",
            Self::DraftInit => "draft_init",
            Self::DraftContinue => "draft_continue",
        }
    }

    const fn expected_shape(self) -> &'static str {
        match self {
            Self::FullGraph => {
                r#"{
  "graph": { "start_node": "node_id", "nodes": [NarrativeNode] },
  "world_state_schema": { "fields": {} },
  "player_state_schema": { "fields": {} },
  "introduction": "text"
}"#
            }
            Self::DraftInit => {
                r#"{
  "nodes": [NarrativeNode],
  "transition_patches": [NodeTransitionPatch],
  "section_summary": "text",
  "start_node": "node_id",
  "world_state_schema": { "fields": {} },
  "player_state_schema": { "fields": {} },
  "introduction": "text"
}"#
            }
            Self::DraftContinue => {
                r#"{
  "nodes": [NarrativeNode],
  "transition_patches": [NodeTransitionPatch],
  "section_summary": "text"
}"#
            }
        }
    }
}

fn compact_character_summary(character: &CharacterCard) -> ArchitectCharacterSummary {
    let mut role_summary_parts = vec![
        character.personality.trim().to_owned(),
        character.style.trim().to_owned(),
    ];
    if !character.tendencies.is_empty() {
        role_summary_parts.push(character.tendencies.join(", "));
    }

    ArchitectCharacterSummary {
        id: character.id.clone(),
        name: character.name.clone(),
        role_summary: truncate_text(&role_summary_parts.join(" | "), 180),
        state_schema_keys: compact_schema_map(Some(&character.state_schema)).unwrap_or_default(),
    }
}

fn compact_schema_map(
    fields: Option<&std::collections::HashMap<String, StateFieldSchema>>,
) -> Option<BTreeMap<String, CompactStateField>> {
    fields.map(|fields| {
        fields
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    CompactStateField {
                        value_type: value.value_type.clone(),
                        default: value.default.clone(),
                    },
                )
            })
            .collect()
    })
}

fn compact_recent_section_node(node: &NarrativeNode) -> RecentSectionDetailNode {
    RecentSectionDetailNode {
        id: node.id.clone(),
        title: node.title.clone(),
        scene_summary: truncate_text(&node.scene, 140),
        goal: truncate_text(&node.goal, 100),
        characters: node.characters.clone(),
        transition_targets: node
            .transitions
            .iter()
            .map(|transition| transition.to.clone())
            .collect(),
        on_enter_update_keys: collect_state_update_keys(&node.on_enter_updates),
    }
}

fn collect_state_update_keys(updates: &[StateOp]) -> Vec<String> {
    updates
        .iter()
        .map(|update| match update {
            StateOp::SetCurrentNode { node_id } => format!("current_node:{node_id}"),
            StateOp::SetActiveCharacters { .. } => "active_characters".to_owned(),
            StateOp::AddActiveCharacter { character } => format!("active+:{character}"),
            StateOp::RemoveActiveCharacter { character } => format!("active-:{character}"),
            StateOp::SetState { key, .. } => format!("world:{key}"),
            StateOp::RemoveState { key } => format!("world:{key}"),
            StateOp::SetPlayerState { key, .. } => format!("player:{key}"),
            StateOp::RemovePlayerState { key } => format!("player:{key}"),
            StateOp::SetCharacterState { character, key, .. } => {
                format!("character:{character}:{key}")
            }
            StateOp::RemoveCharacterState { character, key } => {
                format!("character:{character}:{key}")
            }
        })
        .collect()
}

fn build_repair_prompt(
    repair_target: ArchitectRepairTarget,
    raw_output: &str,
    original_error: &ArchitectError,
) -> String {
    format!(
        r#"TARGET:
{}

PARSER_ERROR:
{}

EXPECTED_SHAPE:
{}

RAW_OUTPUT:
{}
"#,
        repair_target.as_str(),
        original_error,
        repair_target.expected_shape(),
        raw_output
    )
}

fn validate_draft_init_bundle(bundle: &ArchitectDraftOutputBundle) -> Result<(), ArchitectError> {
    if bundle.nodes.is_empty() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft init returned no nodes".to_owned(),
        ));
    }
    if bundle.start_node.is_none() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft init did not provide start_node".to_owned(),
        ));
    }
    if bundle.world_state_schema.is_none() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft init did not provide world_state_schema".to_owned(),
        ));
    }
    if bundle.introduction.is_none() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft init did not provide introduction".to_owned(),
        ));
    }
    if bundle.section_summary.is_none() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft init did not provide section_summary".to_owned(),
        ));
    }
    Ok(())
}

fn validate_draft_continue_bundle(
    bundle: &ArchitectDraftOutputBundle,
) -> Result<(), ArchitectError> {
    if bundle.nodes.is_empty() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft continue returned no nodes".to_owned(),
        ));
    }
    if bundle.section_summary.is_none() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft continue did not provide section_summary".to_owned(),
        ));
    }
    if bundle.start_node.is_some() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft continue must not return start_node".to_owned(),
        ));
    }
    if bundle.world_state_schema.is_some() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft continue must not return world_state_schema".to_owned(),
        ));
    }
    if bundle.player_state_schema.is_some() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft continue must not return player_state_schema".to_owned(),
        ));
    }
    if bundle.introduction.is_some() {
        return Err(ArchitectError::InvalidDraftOutput(
            "architect draft continue must not return introduction".to_owned(),
        ));
    }
    Ok(())
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for (index, ch) in text.chars().enumerate() {
        if index >= max_chars {
            output.push_str("...");
            break;
        }
        output.push(ch);
    }
    output
}
