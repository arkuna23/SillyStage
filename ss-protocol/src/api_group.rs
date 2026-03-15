use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiGroupBindingsInput {
    pub planner_api_id: String,
    pub architect_api_id: String,
    pub director_api_id: String,
    pub actor_api_id: String,
    pub narrator_api_id: String,
    pub keeper_api_id: String,
    pub replyer_api_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiGroupBindingsPayload {
    pub planner_api_id: String,
    pub architect_api_id: String,
    pub director_api_id: String,
    pub actor_api_id: String,
    pub narrator_api_id: String,
    pub keeper_api_id: String,
    pub replyer_api_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiGroupCreateParams {
    pub api_group_id: String,
    pub display_name: String,
    pub bindings: ApiGroupBindingsInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiGroupGetParams {
    pub api_group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ApiGroupListParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiGroupUpdateParams {
    pub api_group_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bindings: Option<ApiGroupBindingsInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ApiGroupDeleteParams {
    pub api_group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiGroupPayload {
    pub api_group_id: String,
    pub display_name: String,
    pub bindings: ApiGroupBindingsPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiGroupsListedPayload {
    pub api_groups: Vec<ApiGroupPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiGroupDeletedPayload {
    pub api_group_id: String,
}
