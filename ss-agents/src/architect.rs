use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use llm::{ChatRequest, LlmApi};
use serde::Serialize;
use state::schema::WorldStateSchema;
use story::graph::StoryGraph;

/// Architect 的输入
#[derive(Debug, Clone, Copy)]
pub struct ArchitectRequest<'a> {
    pub story_concept: &'a str,
    pub world_state_schema: &'a WorldStateSchema,
    pub available_characters: &'a [CharacterCard],
}

/// Architect 的输出
#[derive(Debug, Clone, Serialize)]
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

        let graph: StoryGraph = output
            .structured_output
            .as_ref()
            .ok_or_else(|| ArchitectError::MissingOutput)
            .and_then(|r| serde_json::from_value(r.clone()).map_err(ArchitectError::InvalidJson))?;

        Ok(ArchitectResponse { graph, output })
    }

    fn build_user_prompt(&self, req: &ArchitectRequest<'_>) -> Result<String, ArchitectError> {
        let schema_json = serde_json::to_string_pretty(&req.world_state_schema)
            .map_err(ArchitectError::SerializeSchema)?;

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

WORLD_STATE_SCHEMA:
{}

AVAILABLE_CHARACTERS:
{}
"#,
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
