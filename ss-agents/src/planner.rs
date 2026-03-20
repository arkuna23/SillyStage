use std::sync::Arc;

use crate::actor::{CharacterCard, CharacterCardSummaryRef};
use crate::prompt::{PromptProfile, render_character_summaries, render_prompt_modules};
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
        let (system_prompt, user_prompt) = self.build_prompts(&request)?;
        let output = self
            .client
            .chat({
                let mut builder = ChatRequest::builder()
                    .model(self.model.clone())
                    .system_message(system_prompt)
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

    fn build_prompts(
        &self,
        request: &PlannerRequest<'_>,
    ) -> Result<(String, String), PlannerError> {
        let character_summaries: Vec<CharacterCardSummaryRef<'_>> = request
            .available_characters
            .iter()
            .map(|card| card.summary_ref(None))
            .collect();
        let system_prompt =
            render_prompt_modules(&self.prompt_profile.system_modules, |key| match key {
                "story_concept" => Some(request.story_concept.to_owned()),
                "lorebook_base" => request.lorebook_base.as_deref().map(str::to_owned),
                "available_characters" => {
                    Some(render_character_summaries(&character_summaries, None))
                }
                "lorebook_matched" => request.lorebook_matched.as_deref().map(str::to_owned),
                _ => None,
            });
        let system_prompt = if system_prompt.is_empty() {
            self.prompt_profile.system_prompt.clone()
        } else if self.prompt_profile.system_prompt.is_empty() {
            system_prompt
        } else {
            format!("{}\n\n{}", self.prompt_profile.system_prompt, system_prompt)
        };
        let user_prompt =
            render_prompt_modules(&self.prompt_profile.user_modules, |key| match key {
                "story_concept" => Some(request.story_concept.to_owned()),
                "lorebook_base" => request.lorebook_base.as_deref().map(str::to_owned),
                "available_characters" => {
                    Some(render_character_summaries(&character_summaries, None))
                }
                "lorebook_matched" => request.lorebook_matched.as_deref().map(str::to_owned),
                _ => None,
            });

        Ok((system_prompt, user_prompt))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error(transparent)]
    SerializeCharacters(serde_json::Error),
}
