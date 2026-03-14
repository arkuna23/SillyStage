use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct PlannerRequest<'a> {
    pub story_concept: &'a str,
    pub available_characters: &'a [CharacterCard],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerResponse {
    pub story_script: String,
    pub output: llm::ChatResponse,
}

pub struct Planner {
    client: Arc<dyn LlmApi>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Planner {
    pub fn new(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, PlannerError> {
        Self::new_with_options(client, model, None, None)
    }

    pub fn new_with_options(
        client: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Self, PlannerError> {
        Ok(Self {
            client,
            model: model.into(),
            system_prompt: include_str!("./prompts/planner.txt").to_owned(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_prompt_file(
        client: Arc<dyn LlmApi>,
        model: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, PlannerError> {
        let system_prompt = fs::read_to_string(path).map_err(PlannerError::ReadPrompt)?;

        Ok(Self {
            client,
            model: model.into(),
            system_prompt,
            temperature: None,
            max_tokens: None,
        })
    }

    pub async fn plan(&self, request: PlannerRequest<'_>) -> Result<PlannerResponse, PlannerError> {
        let user_prompt = self.build_user_prompt(&request)?;
        let output = self
            .client
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(self.model.clone())
                    .system_message(self.system_prompt.clone())
                    .user_message(user_prompt);
                if let Some(temperature) = self.temperature {
                    builder = builder.temperature(temperature);
                }
                if let Some(max_tokens) = self.max_tokens {
                    builder = builder.max_tokens(max_tokens);
                }
                builder.build()?
            })
            .await?;

        Ok(PlannerResponse {
            story_script: output.message.content.clone(),
            output,
        })
    }

    fn build_user_prompt(&self, request: &PlannerRequest<'_>) -> Result<String, PlannerError> {
        let character_summaries: Vec<CharacterCardSummaryRef<'_>> = request
            .available_characters
            .iter()
            .map(CharacterCard::summary_ref)
            .collect();
        let characters_json = serde_json::to_string_pretty(&character_summaries)
            .map_err(PlannerError::SerializeCharacters)?;

        Ok(format!(
            r#"STORY_CONCEPT:
{}

AVAILABLE_CHARACTERS:
{}
"#,
            request.story_concept, characters_json
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error(transparent)]
    SerializeCharacters(serde_json::Error),
    #[error(transparent)]
    ReadPrompt(std::io::Error),
}
