use std::collections::HashMap;

use llm::LlmApi;
use serde::{Deserialize, Serialize};

use crate::engine::{AgentModelConfig, RuntimeAgentConfigs, StoryGenerationAgentConfigs};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentApiIds {
    pub planner_api_id: String,
    pub architect_api_id: String,
    pub director_api_id: String,
    pub actor_api_id: String,
    pub narrator_api_id: String,
    pub keeper_api_id: String,
}

impl AgentApiIds {
    pub fn apply_overrides(&self, overrides: &AgentApiIdOverrides) -> Self {
        Self {
            planner_api_id: overrides
                .planner_api_id
                .clone()
                .unwrap_or_else(|| self.planner_api_id.clone()),
            architect_api_id: overrides
                .architect_api_id
                .clone()
                .unwrap_or_else(|| self.architect_api_id.clone()),
            director_api_id: overrides
                .director_api_id
                .clone()
                .unwrap_or_else(|| self.director_api_id.clone()),
            actor_api_id: overrides
                .actor_api_id
                .clone()
                .unwrap_or_else(|| self.actor_api_id.clone()),
            narrator_api_id: overrides
                .narrator_api_id
                .clone()
                .unwrap_or_else(|| self.narrator_api_id.clone()),
            keeper_api_id: overrides
                .keeper_api_id
                .clone()
                .unwrap_or_else(|| self.keeper_api_id.clone()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentApiIdOverrides {
    pub planner_api_id: Option<String>,
    pub architect_api_id: Option<String>,
    pub director_api_id: Option<String>,
    pub actor_api_id: Option<String>,
    pub narrator_api_id: Option<String>,
    pub keeper_api_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionConfigMode {
    #[default]
    UseGlobal,
    UseSession,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionEngineConfig {
    pub mode: SessionConfigMode,
    pub session_api_ids: Option<AgentApiIds>,
}

impl SessionEngineConfig {
    pub fn use_global() -> Self {
        Self {
            mode: SessionConfigMode::UseGlobal,
            session_api_ids: None,
        }
    }

    pub fn use_session(api_ids: AgentApiIds) -> Self {
        Self {
            mode: SessionConfigMode::UseSession,
            session_api_ids: Some(api_ids),
        }
    }
}

#[derive(Clone)]
pub struct RegisteredApi<'a> {
    pub client: &'a dyn LlmApi,
    pub model: String,
}

impl<'a> RegisteredApi<'a> {
    pub fn new(client: &'a dyn LlmApi, model: impl Into<String>) -> Self {
        Self {
            client,
            model: model.into(),
        }
    }
}

#[derive(Clone, Default)]
pub struct LlmApiRegistry<'a> {
    apis: HashMap<String, RegisteredApi<'a>>,
}

impl<'a> LlmApiRegistry<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        mut self,
        api_id: impl Into<String>,
        client: &'a dyn LlmApi,
        model: impl Into<String>,
    ) -> Self {
        self.apis
            .insert(api_id.into(), RegisteredApi::new(client, model));
        self
    }

    pub fn resolve(&self, api_id: &str) -> Result<&RegisteredApi<'a>, RegistryError> {
        self.apis
            .get(api_id)
            .ok_or_else(|| RegistryError::UnknownApiId(api_id.to_owned()))
    }

    pub fn build_story_generation_configs(
        &self,
        api_ids: &AgentApiIds,
    ) -> Result<StoryGenerationAgentConfigs<'a>, RegistryError> {
        Ok(StoryGenerationAgentConfigs {
            planner: self.resolve_story_agent(&api_ids.planner_api_id)?,
            architect: self.resolve_story_agent(&api_ids.architect_api_id)?,
        })
    }

    pub fn build_runtime_configs(
        &self,
        api_ids: &AgentApiIds,
    ) -> Result<RuntimeAgentConfigs<'a>, RegistryError> {
        Ok(RuntimeAgentConfigs {
            director: self.resolve_runtime_agent(&api_ids.director_api_id)?,
            actor: self.resolve_runtime_agent(&api_ids.actor_api_id)?,
            narrator: self.resolve_runtime_agent(&api_ids.narrator_api_id)?,
            keeper: self.resolve_runtime_agent(&api_ids.keeper_api_id)?,
        })
    }

    fn resolve_story_agent(&self, api_id: &str) -> Result<AgentModelConfig<'a>, RegistryError> {
        let api = self.resolve(api_id)?;
        Ok(AgentModelConfig::new(api.client, api.model.clone()))
    }

    fn resolve_runtime_agent(&self, api_id: &str) -> Result<AgentModelConfig<'a>, RegistryError> {
        let api = self.resolve(api_id)?;
        Ok(AgentModelConfig::new(api.client, api.model.clone()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("unknown llm api id: {0}")]
    UnknownApiId(String),
}
