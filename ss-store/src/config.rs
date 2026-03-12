use serde::{Deserialize, Serialize};

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
