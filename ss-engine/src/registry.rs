use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use llm::{LlmApi, LlmError, OpenAiClient, OpenAiConfig};
use store::{AgentPresetConfig, ApiRecord, LlmProvider, PresetRecord};

use crate::engine::{AgentModelConfig, RuntimeAgentConfigs, StoryGenerationAgentConfigs};

#[derive(Clone)]
pub struct RegisteredApi {
    pub client: Arc<dyn LlmApi>,
    pub model: String,
}

pub struct RuntimeApiRecords<'a> {
    pub director: &'a ApiRecord,
    pub actor: &'a ApiRecord,
    pub narrator: &'a ApiRecord,
    pub keeper: &'a ApiRecord,
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
        planner_api: &ApiRecord,
        architect_api: &ApiRecord,
        planner_preset: &AgentPresetConfig,
        architect_preset: &AgentPresetConfig,
    ) -> Result<StoryGenerationAgentConfigs, RegistryError> {
        Ok(StoryGenerationAgentConfigs {
            planner: self.build_agent_model_config(planner_api, planner_preset)?,
            architect: self.build_agent_model_config(architect_api, architect_preset)?,
        })
    }

    pub fn build_runtime_configs(
        &self,
        apis: RuntimeApiRecords<'_>,
        preset: &PresetRecord,
    ) -> Result<RuntimeAgentConfigs, RegistryError> {
        Ok(RuntimeAgentConfigs {
            director: self.build_agent_model_config(apis.director, &preset.agents.director)?,
            actor: self.build_agent_model_config(apis.actor, &preset.agents.actor)?,
            narrator: self.build_agent_model_config(apis.narrator, &preset.agents.narrator)?,
            keeper: self.build_agent_model_config(apis.keeper, &preset.agents.keeper)?,
        })
    }

    pub fn build_replyer_config(
        &self,
        api: &ApiRecord,
        preset: &AgentPresetConfig,
    ) -> Result<AgentModelConfig, RegistryError> {
        self.build_agent_model_config(api, preset)
    }

    pub async fn list_models(
        &self,
        provider: LlmProvider,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<String>, RegistryError> {
        let client: Arc<dyn LlmApi> = match provider {
            LlmProvider::OpenAi => {
                let config = OpenAiConfig::builder()
                    .api_key(api_key)
                    .base_url(base_url)
                    .default_model("model-probe")
                    .build()?;
                Arc::new(OpenAiClient::new(config)?)
            }
        };

        client.list_models().await.map_err(RegistryError::Llm)
    }

    fn build_agent_model_config(
        &self,
        api: &ApiRecord,
        preset: &AgentPresetConfig,
    ) -> Result<AgentModelConfig, RegistryError> {
        if let Ok(registered) = self.resolve(&api.api_id) {
            return Ok(AgentModelConfig::new(registered.client, registered.model)
                .with_temperature(preset.temperature)
                .with_max_tokens(preset.max_tokens));
        }

        let client: Arc<dyn LlmApi> = match api.provider {
            LlmProvider::OpenAi => {
                let config = OpenAiConfig::builder()
                    .api_key(&api.api_key)
                    .base_url(&api.base_url)
                    .default_model(&api.model)
                    .build()?;
                Arc::new(OpenAiClient::new(config)?)
            }
        };

        Ok(AgentModelConfig::new(client, &api.model)
            .with_temperature(preset.temperature)
            .with_max_tokens(preset.max_tokens))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("unknown llm api id: {0}")]
    UnknownApiId(String),
    #[error("failed to build llm api client: {0}")]
    Llm(#[from] LlmError),
}
