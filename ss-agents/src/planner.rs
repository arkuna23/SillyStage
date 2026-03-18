use std::sync::Arc;

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use crate::prompt::{PromptProfile, render_character_summaries, render_prompt_entries};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct PlannerRequest<'a> {
    pub story_concept: &'a str,
    pub available_characters: &'a [CharacterCard],
    pub lorebook_base: Option<String>,
    pub lorebook_matched: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerResponse {
    pub story_script: String,
    pub output: llm::ChatResponse,
}

pub struct Planner {
    client: Arc<dyn LlmApi>,
    model: String,
    prompt_profile: PromptProfile,
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
            prompt_profile: PromptProfile::default(),
            temperature,
            max_tokens,
        })
    }

    pub fn with_prompt_profile(mut self, prompt_profile: PromptProfile) -> Self {
        self.prompt_profile = prompt_profile;
        self
    }

    pub async fn plan(&self, request: PlannerRequest<'_>) -> Result<PlannerResponse, PlannerError> {
        let (stable_prompt, dynamic_prompt) = self.build_user_prompts(&request)?;
        let output = self
            .client
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(self.model.clone())
                    .system_message(self.prompt_profile.system_prompt.clone())
                    .user_message(stable_prompt)
                    .user_message(dynamic_prompt);
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

    fn build_user_prompts(
        &self,
        request: &PlannerRequest<'_>,
    ) -> Result<(String, String), PlannerError> {
        let character_summaries: Vec<CharacterCardSummaryRef<'_>> = request
            .available_characters
            .iter()
            .map(|card| card.summary_ref(None))
            .collect();
        let stable_prompt =
            render_prompt_entries(&self.prompt_profile.stable_entries, |key| match key {
                "story_concept" => Some(request.story_concept.to_owned()),
                "lorebook_base" => request.lorebook_base.as_deref().map(str::to_owned),
                "available_characters" => {
                    Some(render_character_summaries(&character_summaries, None))
                }
                _ => None,
            });
        let dynamic_prompt =
            render_prompt_entries(&self.prompt_profile.dynamic_entries, |key| match key {
                "lorebook_matched" => request.lorebook_matched.as_deref().map(str::to_owned),
                _ => None,
            });

        Ok((stable_prompt, dynamic_prompt))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error(transparent)]
    SerializeCharacters(serde_json::Error),
}
