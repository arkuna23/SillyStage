use engine::{AgentApiIdOverrides, AgentApiIds, SessionConfigMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GlobalConfigPayload {
    pub api_ids: Option<AgentApiIds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionConfigPayload {
    pub mode: SessionConfigMode,
    pub session_api_ids: Option<AgentApiIds>,
    pub effective_api_ids: AgentApiIds,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ConfigGetGlobalParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConfigUpdateGlobalParams {
    pub api_overrides: AgentApiIdOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct SessionGetConfigParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionUpdateConfigParams {
    pub mode: SessionConfigMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_api_ids: Option<AgentApiIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_overrides: Option<AgentApiIdOverrides>,
}
