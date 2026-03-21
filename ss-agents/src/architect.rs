use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use crate::actor::CharacterCard;
use crate::prompt::{
    ArchitectPromptProfiles, CharacterTemplateContext, PromptProfile, compact_json,
    merge_system_prompt, render_character_text, render_prompt_modules,
};
use llm::{ChatRequest, LlmApi};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::StateOp;
use state::schema::{PlayerStateSchema, StateFieldSchema, WorldStateSchema};
use story::{NarrativeNode, StoryGraph, Transition};
use tracing::{error, warn};

/// Architect 的输入
#[derive(Debug, Clone)]
pub struct ArchitectRequest<'a> {
    pub story_concept: &'a str,
    pub planned_story: Option<&'a str>,
    pub world_state_schema: Option<&'a WorldStateSchema>,
    pub player_state_schema: Option<&'a PlayerStateSchema>,
    pub available_characters: &'a [CharacterCard],
    pub lorebook_base: Option<&'a str>,
    pub lorebook_matched: Option<&'a str>,
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

#[derive(Debug, Clone)]
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
    pub lorebook_base: Option<&'a str>,
    pub lorebook_matched: Option<&'a str>,
}

#[derive(Debug, Clone)]
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
    pub lorebook_base: Option<&'a str>,
    pub lorebook_matched: Option<&'a str>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_values: Option<Vec<Value>>,
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
    prompt_profiles: ArchitectPromptProfiles,
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
            prompt_profiles: ArchitectPromptProfiles::default(),
            temperature,
            max_tokens,
        }
    }

    pub fn with_prompt_profiles(mut self, prompt_profiles: ArchitectPromptProfiles) -> Self {
        self.prompt_profiles = prompt_profiles;
        self
    }

    pub async fn generate_graph(
        &self,
        req: ArchitectRequest<'_>,
    ) -> Result<ArchitectResponse, ArchitectError> {
        let (system_prompt, user_prompt) = self.build_graph_prompts(&req)?;
        let (bundle, output) = self
            .chat_json_with_repair(
                &system_prompt,
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
        let (system_prompt, user_prompt) = self.build_draft_prompts(
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
                lorebook_base: req.lorebook_base,
                lorebook_matched: req.lorebook_matched,
            },
        )?;
        let (bundle, output) = self
            .chat_json_with_repair(
                &system_prompt,
                user_prompt,
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
        let (system_prompt, user_prompt) = self.build_draft_prompts(
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
                lorebook_base: req.lorebook_base,
                lorebook_matched: req.lorebook_matched,
            },
        )?;
        let (bundle, output) = self
            .chat_json_with_repair(
                &system_prompt,
                user_prompt,
                ArchitectRepairTarget::DraftContinue,
                Self::parse_json_output::<ArchitectDraftOutputBundle>,
                |bundle| {
                    validate_draft_continue_bundle(
                        bundle,
                        &req.graph_summary
                            .iter()
                            .map(|node| node.id.as_str())
                            .collect(),
                    )
                },
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

    fn build_graph_prompts(
        &self,
        req: &ArchitectRequest<'_>,
    ) -> Result<(String, String), ArchitectError> {
        let character_summaries = render_architect_character_summaries(
            &req.available_characters
                .iter()
                .map(compact_character_summary)
                .collect::<Vec<_>>(),
        );

        let system_prompt =
            self.render_profile_system_prompt(&self.prompt_profiles.graph, |key| match key {
                "story_concept" => Some(req.story_concept.to_owned()),
                "lorebook_base" => req.lorebook_base.map(str::to_owned),
                "planned_story" => Some(req.planned_story.unwrap_or("null").to_owned()),
                "available_characters" => Some(character_summaries.clone()),
                "world_state_schema_seed" => Some(render_compact_schema_text(
                    req.world_state_schema.map(|schema| &schema.fields),
                )),
                "player_state_schema_seed" => Some(render_compact_schema_text(
                    req.player_state_schema.map(|schema| &schema.fields),
                )),
                "lorebook_matched" => req.lorebook_matched.map(str::to_owned),
                _ => None,
            });
        let user_prompt =
            self.render_profile_user_prompt(&self.prompt_profiles.graph, |key| match key {
                "story_concept" => Some(req.story_concept.to_owned()),
                "lorebook_base" => req.lorebook_base.map(str::to_owned),
                "planned_story" => Some(req.planned_story.unwrap_or("null").to_owned()),
                "available_characters" => Some(character_summaries.clone()),
                "world_state_schema_seed" => Some(render_compact_schema_text(
                    req.world_state_schema.map(|schema| &schema.fields),
                )),
                "player_state_schema_seed" => Some(render_compact_schema_text(
                    req.player_state_schema.map(|schema| &schema.fields),
                )),
                "lorebook_matched" => req.lorebook_matched.map(str::to_owned),
                _ => None,
            });

        Ok((system_prompt, user_prompt))
    }

    fn build_draft_prompts(
        &self,
        kind: DraftPromptKind,
        input: DraftPromptInput<'_>,
    ) -> Result<(String, String), ArchitectError> {
        let character_summaries = render_architect_character_summaries(
            &input
                .available_characters
                .iter()
                .map(compact_character_summary)
                .collect::<Vec<_>>(),
        );

        match kind {
            DraftPromptKind::Init => {
                let graph_summary = compact_json(&input.graph_summary)
                    .map_err(ArchitectError::SerializeGraphSummary)?;
                let recent_section_detail = compact_json(
                    &input
                        .recent_nodes
                        .iter()
                        .map(compact_recent_section_node)
                        .collect::<Vec<_>>(),
                )
                .map_err(ArchitectError::SerializeRecentNodes)?;

                let system_prompt =
                    self.render_profile_system_prompt(&self.prompt_profiles.draft_init, |key| {
                        match key {
                            "story_concept" => Some(input.story_concept.to_owned()),
                            "lorebook_base" => input.lorebook_base.map(str::to_owned),
                            "planned_story" => {
                                Some(input.planned_story.unwrap_or("null").to_owned())
                            }
                            "available_characters" => Some(character_summaries.clone()),
                            "world_state_schema_seed" => Some(render_compact_schema_text(
                                input.world_state_schema.map(|schema| &schema.fields),
                            )),
                            "player_state_schema_seed" => Some(render_compact_schema_text(
                                input.player_state_schema.map(|schema| &schema.fields),
                            )),
                            "current_section" => Some(input.current_section.to_owned()),
                            "section_index" => Some(input.section_index.to_string()),
                            "total_sections" => Some(input.total_sections.to_string()),
                            "target_node_count" => Some(input.target_node_count.to_string()),
                            "graph_summary" => Some(graph_summary.clone()),
                            "recent_section_detail" => Some(recent_section_detail.clone()),
                            "lorebook_matched" => input.lorebook_matched.map(str::to_owned),
                            _ => None,
                        }
                    });
                let user_prompt =
                    self.render_profile_user_prompt(&self.prompt_profiles.draft_init, |key| {
                        match key {
                            "story_concept" => Some(input.story_concept.to_owned()),
                            "lorebook_base" => input.lorebook_base.map(str::to_owned),
                            "planned_story" => {
                                Some(input.planned_story.unwrap_or("null").to_owned())
                            }
                            "available_characters" => Some(character_summaries.clone()),
                            "world_state_schema_seed" => Some(render_compact_schema_text(
                                input.world_state_schema.map(|schema| &schema.fields),
                            )),
                            "player_state_schema_seed" => Some(render_compact_schema_text(
                                input.player_state_schema.map(|schema| &schema.fields),
                            )),
                            "current_section" => Some(input.current_section.to_owned()),
                            "section_index" => Some(input.section_index.to_string()),
                            "total_sections" => Some(input.total_sections.to_string()),
                            "target_node_count" => Some(input.target_node_count.to_string()),
                            "graph_summary" => Some(graph_summary.clone()),
                            "recent_section_detail" => Some(recent_section_detail.clone()),
                            "lorebook_matched" => input.lorebook_matched.map(str::to_owned),
                            _ => None,
                        }
                    });

                Ok((system_prompt, user_prompt))
            }
            DraftPromptKind::Continue => {
                let graph_summary = compact_json(&input.graph_summary)
                    .map_err(ArchitectError::SerializeGraphSummary)?;
                let recent_section_detail = compact_json(
                    &input
                        .recent_nodes
                        .iter()
                        .map(compact_recent_section_node)
                        .collect::<Vec<_>>(),
                )
                .map_err(ArchitectError::SerializeRecentNodes)?;

                let system_prompt = self.render_profile_system_prompt(
                    &self.prompt_profiles.draft_continue,
                    |key| match key {
                        "story_concept" => Some(input.story_concept.to_owned()),
                        "lorebook_base" => input.lorebook_base.map(str::to_owned),
                        "available_characters" => Some(character_summaries.clone()),
                        "world_state_schema" => Some(render_compact_schema_text(
                            input.world_state_schema.map(|schema| &schema.fields),
                        )),
                        "player_state_schema" => Some(render_compact_schema_text(
                            input.player_state_schema.map(|schema| &schema.fields),
                        )),
                        "section_summaries" => {
                            Some(render_section_summaries(input.section_summaries))
                        }
                        "current_section" => Some(input.current_section.to_owned()),
                        "section_index" => Some(input.section_index.to_string()),
                        "total_sections" => Some(input.total_sections.to_string()),
                        "target_node_count" => Some(input.target_node_count.to_string()),
                        "graph_summary" => Some(graph_summary.clone()),
                        "recent_section_detail" => Some(recent_section_detail.clone()),
                        "lorebook_matched" => input.lorebook_matched.map(str::to_owned),
                        _ => None,
                    },
                );
                let user_prompt =
                    self.render_profile_user_prompt(&self.prompt_profiles.draft_continue, |key| {
                        match key {
                            "story_concept" => Some(input.story_concept.to_owned()),
                            "lorebook_base" => input.lorebook_base.map(str::to_owned),
                            "available_characters" => Some(character_summaries.clone()),
                            "world_state_schema" => Some(render_compact_schema_text(
                                input.world_state_schema.map(|schema| &schema.fields),
                            )),
                            "player_state_schema" => Some(render_compact_schema_text(
                                input.player_state_schema.map(|schema| &schema.fields),
                            )),
                            "section_summaries" => {
                                Some(render_section_summaries(input.section_summaries))
                            }
                            "current_section" => Some(input.current_section.to_owned()),
                            "section_index" => Some(input.section_index.to_string()),
                            "total_sections" => Some(input.total_sections.to_string()),
                            "target_node_count" => Some(input.target_node_count.to_string()),
                            "graph_summary" => Some(graph_summary.clone()),
                            "recent_section_detail" => Some(recent_section_detail.clone()),
                            "lorebook_matched" => input.lorebook_matched.map(str::to_owned),
                            _ => None,
                        }
                    });

                Ok((system_prompt, user_prompt))
            }
        }
    }

    fn render_profile_system_prompt<F>(&self, profile: &PromptProfile, resolve: F) -> String
    where
        F: Fn(&str) -> Option<String>,
    {
        merge_system_prompt(
            &profile.system_prompt,
            &render_prompt_modules(&profile.system_modules, resolve),
        )
    }

    fn render_profile_user_prompt<F>(&self, profile: &PromptProfile, resolve: F) -> String
    where
        F: Fn(&str) -> Option<String>,
    {
        render_prompt_modules(&profile.user_modules, resolve)
    }

    async fn chat_json_with_repair<T, P, V>(
        &self,
        system_prompt: &str,
        user_message: String,
        repair_target: ArchitectRepairTarget,
        parse: P,
        validate: V,
    ) -> Result<(T, llm::ChatResponse), ArchitectError>
    where
        T: DeserializeOwned,
        P: Fn(&llm::ChatResponse) -> Result<T, ArchitectError>,
        V: Fn(&T) -> Result<(), ArchitectError>,
    {
        match self.chat_json(system_prompt, user_message.clone()).await {
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
            .chat_json(&self.prompt_profiles.repair_system_prompt, repair_prompt)
            .await?;
        let parsed = parse(&repaired)?;
        validate(&parsed)?;
        Ok((parsed, repaired))
    }

    async fn chat_json(
        &self,
        system_prompt: &str,
        user_message: String,
    ) -> Result<llm::ChatResponse, ArchitectError> {
        let mut builder = ChatRequest::builder()
            .model(self.model.clone())
            .system_message(system_prompt)
            .user_message(user_message)
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

#[derive(Clone)]
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
    lorebook_base: Option<&'a str>,
    lorebook_matched: Option<&'a str>,
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
  "world_state_schema": {
    "fields": {
      "gate_open": {
        "value_type": "bool",
        "default": false,
        "description": "whether the gate is open"
      }
    }
  },
  "player_state_schema": {
    "fields": {
      "trust": {
        "value_type": "int",
        "default": 0,
        "description": "how much the player is trusted"
      }
    }
  },
  "introduction": "text"
}"#
            }
            Self::DraftInit => {
                r#"{
  "nodes": [NarrativeNode],
  "transition_patches": [NodeTransitionPatch],
  "section_summary": "text",
  "start_node": "node_id",
  "world_state_schema": {
    "fields": {
      "gate_open": {
        "value_type": "bool",
        "default": false,
        "description": "whether the gate is open"
      }
    }
  },
  "player_state_schema": {
    "fields": {
      "trust": {
        "value_type": "int",
        "default": 0,
        "description": "how much the player is trusted"
      }
    }
  },
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
    let template_context = CharacterTemplateContext {
        character_name: &character.name,
        player_name: None,
        state_schema: &character.state_schema,
        state_values: None,
    };
    let role_summary_parts = vec![
        render_character_text(&character.personality, &template_context),
        render_character_text(&character.style, &template_context),
    ];

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
                        enum_values: value.enum_values.clone(),
                    },
                )
            })
            .collect()
    })
}

fn render_architect_character_summaries(summaries: &[ArchitectCharacterSummary]) -> String {
    if summaries.is_empty() {
        return "- none".to_owned();
    }

    summaries
        .iter()
        .map(|summary| {
            format!(
                "- {} | {} | role={} | state_schema={}",
                summary.id,
                summary.name,
                summary.role_summary,
                render_state_schema_fields_from_compact(&summary.state_schema_keys)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_compact_schema_text(
    fields: Option<&std::collections::HashMap<String, StateFieldSchema>>,
) -> String {
    compact_schema_map(fields)
        .map(|fields| render_state_schema_fields_from_compact(&fields))
        .unwrap_or_else(|| "null".to_owned())
}

fn render_state_schema_fields_from_compact(fields: &BTreeMap<String, CompactStateField>) -> String {
    if fields.is_empty() {
        return "none".to_owned();
    }

    fields
        .iter()
        .map(|(key, field)| {
            let mut line = format!(
                "{key}:{}",
                compact_json(&field.value_type).unwrap_or_default()
            );
            if let Some(default) = &field.default {
                line.push_str(&format!(
                    " default={}",
                    compact_json(default).unwrap_or_default()
                ));
            }
            if let Some(enum_values) = &field.enum_values {
                line.push_str(&format!(
                    " enum={}",
                    compact_json(enum_values).unwrap_or_default()
                ));
            }
            line
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_section_summaries(section_summaries: &[String]) -> String {
    if section_summaries.is_empty() {
        return "- none".to_owned();
    }

    section_summaries
        .iter()
        .enumerate()
        .map(|(idx, summary)| format!("- [{}] {}", idx, summary))
        .collect::<Vec<_>>()
        .join("\n")
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
    let returned_node_ids: HashSet<&str> =
        bundle.nodes.iter().map(|node| node.id.as_str()).collect();
    if let Some(start_node) = &bundle.start_node
        && !returned_node_ids.contains(start_node.as_str())
    {
        return Err(ArchitectError::InvalidDraftOutput(format!(
            "architect draft init start_node '{}' is not included in returned nodes",
            start_node
        )));
    }
    validate_draft_chunk_transitions(bundle, &HashSet::new())?;
    Ok(())
}

fn validate_draft_continue_bundle(
    bundle: &ArchitectDraftOutputBundle,
    existing_node_ids: &HashSet<&str>,
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
    validate_draft_chunk_transitions(bundle, existing_node_ids)?;
    Ok(())
}

fn validate_draft_chunk_transitions(
    bundle: &ArchitectDraftOutputBundle,
    existing_node_ids: &HashSet<&str>,
) -> Result<(), ArchitectError> {
    let mut returned_node_ids = HashSet::new();
    for node in &bundle.nodes {
        if existing_node_ids.contains(node.id.as_str()) {
            return Err(ArchitectError::InvalidDraftOutput(format!(
                "architect draft reused existing graph node id '{}' as a new node; existing graph nodes [{}] may only be referenced in transition targets or transition_patches",
                node.id,
                sorted_ids(existing_node_ids),
            )));
        }
        if !returned_node_ids.insert(node.id.as_str()) {
            return Err(ArchitectError::InvalidDraftOutput(format!(
                "architect draft returned duplicate node id '{}' within this response; returned node ids seen so far [{}]",
                node.id,
                sorted_ids(&returned_node_ids),
            )));
        }
    }

    let valid_targets: HashSet<&str> = existing_node_ids
        .iter()
        .copied()
        .chain(returned_node_ids.iter().copied())
        .collect();

    for node in &bundle.nodes {
        for transition in &node.transitions {
            if !valid_targets.contains(transition.to.as_str()) {
                return Err(ArchitectError::InvalidDraftOutput(format!(
                    "architect draft transition from '{}' points to missing node '{}'; allowed targets are existing graph nodes [{}] or returned nodes [{}]",
                    node.id,
                    transition.to,
                    sorted_ids(existing_node_ids),
                    sorted_ids(&returned_node_ids),
                )));
            }
        }
    }

    for patch in &bundle.transition_patches {
        if !valid_targets.contains(patch.node_id.as_str()) {
            return Err(ArchitectError::InvalidDraftOutput(format!(
                "architect draft attempted to patch missing node '{}'; allowed patch targets are existing graph nodes [{}] or returned nodes [{}]",
                patch.node_id,
                sorted_ids(existing_node_ids),
                sorted_ids(&returned_node_ids),
            )));
        }
        for transition in &patch.add_transitions {
            if !valid_targets.contains(transition.to.as_str()) {
                return Err(ArchitectError::InvalidDraftOutput(format!(
                    "architect draft patch for '{}' points to missing node '{}'; allowed targets are existing graph nodes [{}] or returned nodes [{}]",
                    patch.node_id,
                    transition.to,
                    sorted_ids(existing_node_ids),
                    sorted_ids(&returned_node_ids),
                )));
            }
        }
    }

    Ok(())
}

fn sorted_ids(ids: &HashSet<&str>) -> String {
    let mut values: Vec<&str> = ids.iter().copied().collect();
    values.sort_unstable();
    values.join(", ")
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
