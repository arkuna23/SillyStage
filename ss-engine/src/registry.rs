use std::collections::HashMap;
use std::sync::Arc;

use llm::LlmApi;
use store::AgentApiIds;

use crate::engine::{AgentModelConfig, RuntimeAgentConfigs, StoryGenerationAgentConfigs};

#[derive(Clone)]
pub struct RegisteredApi {
    pub client: Arc<dyn LlmApi>,
    pub model: String,
}

impl RegisteredApi {
    pub fn new(client: Arc<dyn LlmApi>, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }
}

#[derive(Clone, Default)]
pub struct LlmApiRegistry {
    apis: HashMap<String, RegisteredApi>,
}

impl LlmApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        mut self,
        api_id: impl Into<String>,
        client: Arc<dyn LlmApi>,
        model: impl Into<String>,
    ) -> Self {
        self.apis
            .insert(api_id.into(), RegisteredApi::new(client, model));
        self
    }

    pub fn resolve(&self, api_id: &str) -> Result<&RegisteredApi, RegistryError> {
        self.apis
            .get(api_id)
            .ok_or_else(|| RegistryError::UnknownApiId(api_id.to_owned()))
    }

    pub fn build_story_generation_configs(
        &self,
        api_ids: &AgentApiIds,
    ) -> Result<StoryGenerationAgentConfigs, RegistryError> {
        Ok(StoryGenerationAgentConfigs {
            planner: self.resolve_story_agent(&api_ids.planner_api_id)?,
            architect: self.resolve_story_agent(&api_ids.architect_api_id)?,
        })
    }

    pub fn build_runtime_configs(
        &self,
        api_ids: &AgentApiIds,
    ) -> Result<RuntimeAgentConfigs, RegistryError> {
        Ok(RuntimeAgentConfigs {
            director: self.resolve_runtime_agent(&api_ids.director_api_id)?,
            actor: self.resolve_runtime_agent(&api_ids.actor_api_id)?,
            narrator: self.resolve_runtime_agent(&api_ids.narrator_api_id)?,
            keeper: self.resolve_runtime_agent(&api_ids.keeper_api_id)?,
        })
    }

    fn resolve_story_agent(&self, api_id: &str) -> Result<AgentModelConfig, RegistryError> {
        let api = self.resolve(api_id)?;
        Ok(AgentModelConfig::new(Arc::clone(&api.client), api.model.clone()))
    }

    fn resolve_runtime_agent(&self, api_id: &str) -> Result<AgentModelConfig, RegistryError> {
        let api = self.resolve(api_id)?;
        Ok(AgentModelConfig::new(Arc::clone(&api.client), api.model.clone()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("unknown llm api id: {0}")]
    UnknownApiId(String),
}
