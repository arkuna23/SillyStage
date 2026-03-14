use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use llm::{LlmApi, OpenAiClient, OpenAiConfig};
use store::{AgentApiIds, LlmApiRecord, LlmProvider};

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
    apis: Arc<RwLock<HashMap<String, RegisteredApi>>>,
}

impl LlmApiRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        self,
        api_id: impl Into<String>,
        client: Arc<dyn LlmApi>,
        model: impl Into<String>,
    ) -> Self {
        self.apis
            .write()
            .expect("llm api registry write lock should not be poisoned")
            .insert(api_id.into(), RegisteredApi::new(client, model));
        self
    }

    pub fn upsert_record(&self, record: &LlmApiRecord) -> Result<(), RegistryError> {
        let registered = match record.provider {
            LlmProvider::OpenAi => {
                let config = OpenAiConfig::builder()
                    .api_key(&record.api_key)
                    .base_url(&record.base_url)
                    .default_model(&record.model)
                    .build()?;
                RegisteredApi::new(Arc::new(OpenAiClient::new(config)?), &record.model)
            }
        };

        self.apis
            .write()
            .expect("llm api registry write lock should not be poisoned")
            .insert(record.api_id.clone(), registered);
        Ok(())
    }

    pub fn remove(&self, api_id: &str) {
        self.apis
            .write()
            .expect("llm api registry write lock should not be poisoned")
            .remove(api_id);
    }

    pub fn resolve(&self, api_id: &str) -> Result<RegisteredApi, RegistryError> {
        self.apis
            .read()
            .expect("llm api registry read lock should not be poisoned")
            .get(api_id)
            .cloned()
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
        Ok(AgentModelConfig::new(api.client, api.model))
    }

    fn resolve_runtime_agent(&self, api_id: &str) -> Result<AgentModelConfig, RegistryError> {
        let api = self.resolve(api_id)?;
        Ok(AgentModelConfig::new(api.client, api.model))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("unknown llm api id: {0}")]
    UnknownApiId(String),
    #[error("failed to build llm api client: {0}")]
    Llm(#[from] llm::LlmError),
}
