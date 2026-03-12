use std::sync::Arc;

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use state::schema::{PlayerStateSchema, WorldStateSchema};
use story::graph::StoryGraph;

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

#[derive(Debug, Clone, Deserialize)]
struct ArchitectOutputBundle {
    graph: StoryGraph,
    world_state_schema: WorldStateSchema,
    #[serde(default)]
    player_state_schema: Option<PlayerStateSchema>,
    introduction: String,
}

/// Architect agent
pub struct Architect {
    client: Arc<dyn LlmApi>,
    model: String,
}

impl Architect {
    pub fn new(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    pub async fn generate_graph(
        &self,
        req: ArchitectRequest<'_>,
    ) -> Result<ArchitectResponse, ArchitectError> {
        let user_prompt = self.build_user_prompt(&req)?;

        let output = self
            .client
            .chat(
                ChatRequest::builder()
                    .model(self.model.clone())
                    .system_message(include_str!("./prompts/architect.txt"))
                    .user_message(user_prompt)
                    .response_format(llm::ResponseFormat::JsonObject)
                    .build()?,
            )
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
    #[error("missing json output")]
    MissingOutput,
}
