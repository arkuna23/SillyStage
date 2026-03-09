use crate::actor::{CharacterCard, CharacterCardSummary};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use state::schema::WorldStateSchema;
use story::graph::StoryGraph;

/// Architect 的输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectRequest {
    pub story_concept: String,
    pub world_state_schema: WorldStateSchema,
    pub available_characters: Vec<CharacterCard>,
}

/// Architect 的输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectResponse {
    pub graph: StoryGraph,
    pub output: llm::ChatResponse,
}

/// Architect agent
pub struct Architect<'a> {
    client: &'a dyn LlmApi,
    model: String,
}

impl<'a> Architect<'a> {
    pub fn new(client: &'a dyn LlmApi, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }

    pub async fn generate_graph(
        &self,
        req: ArchitectRequest,
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

        let graph: StoryGraph = output
            .structured_output
            .as_ref()
            .ok_or_else(|| ArchitectError::MissingOutput)
            .and_then(|r| serde_json::from_value(r.clone()).map_err(ArchitectError::InvalidJson))?;

        Ok(ArchitectResponse { graph, output })
    }

    fn build_user_prompt(&self, req: &ArchitectRequest) -> Result<String, ArchitectError> {
        let schema_json = serde_json::to_string_pretty(&req.world_state_schema)
            .map_err(ArchitectError::SerializeSchema)?;

        let character_summaries: Vec<CharacterCardSummary> = req
            .available_characters
            .iter()
            .map(CharacterCard::summary)
            .collect();
        let characters_json = serde_json::to_string_pretty(&character_summaries)
            .map_err(ArchitectError::SerializeCharacters)?;

        Ok(format!(
            r#"STORY_CONCEPT:
{}

WORLD_STATE_SCHEMA:
{}

SCHEMA_NOTES:
- WORLD_STATE_SCHEMA.fields are global state fields shared by the whole story.
- WORLD_STATE_SCHEMA.character_fields are per-character private state fields keyed by character id.

CHARACTER_RULES:
- AVAILABLE_CHARACTERS are summarized character cards.
- Use character ids in all structural fields: node.characters, character-scoped conditions, and character state updates.
- Use character names only inside human-readable narrative text when helpful.

AVAILABLE_CHARACTERS:
{}

Design a compact interactive story graph for a demo."#,
            req.story_concept, schema_json, characters_json
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
    SerializeCharacters(serde_json::Error),
    #[error("missing json output")]
    MissingOutput,
}
