use engine::{AgentApiIdOverrides, AgentApiIds, SessionConfigMode};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use state::{PlayerStateSchema, WorldStateSchema};

use crate::config::{
    ConfigGetGlobalParams, ConfigUpdateGlobalParams, SessionGetConfigParams,
    SessionUpdateConfigParams,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestMethod {
    #[serde(rename = "upload.init")]
    UploadInit,
    #[serde(rename = "upload.chunk")]
    UploadChunk,
    #[serde(rename = "upload.complete")]
    UploadComplete,
    #[serde(rename = "character.get")]
    CharacterGet,
    #[serde(rename = "character.list")]
    CharacterList,
    #[serde(rename = "character.delete")]
    CharacterDelete,
    #[serde(rename = "story_resources.create")]
    StoryResourcesCreate,
    #[serde(rename = "story_resources.get")]
    StoryResourcesGet,
    #[serde(rename = "story_resources.list")]
    StoryResourcesList,
    #[serde(rename = "story_resources.update")]
    StoryResourcesUpdate,
    #[serde(rename = "story_resources.delete")]
    StoryResourcesDelete,
    #[serde(rename = "story.generate_plan")]
    StoryGeneratePlan,
    #[serde(rename = "story.generate")]
    StoryGenerate,
    #[serde(rename = "story.get")]
    StoryGet,
    #[serde(rename = "story.list")]
    StoryList,
    #[serde(rename = "story.delete")]
    StoryDelete,
    #[serde(rename = "story.start_session")]
    StoryStartSession,
    #[serde(rename = "session.get")]
    SessionGet,
    #[serde(rename = "session.list")]
    SessionList,
    #[serde(rename = "session.delete")]
    SessionDelete,
    #[serde(rename = "session.run_turn")]
    SessionRunTurn,
    #[serde(rename = "session.update_player_description")]
    SessionUpdatePlayerDescription,
    #[serde(rename = "session.get_runtime_snapshot")]
    SessionGetRuntimeSnapshot,
    #[serde(rename = "config.get_global")]
    ConfigGetGlobal,
    #[serde(rename = "config.update_global")]
    ConfigUpdateGlobal,
    #[serde(rename = "session.get_config")]
    SessionGetConfig,
    #[serde(rename = "session.update_config")]
    SessionUpdateConfig,
}

#[derive(Debug, Clone)]
pub enum RequestParams {
    UploadInit(UploadInitParams),
    UploadChunk(UploadChunkParams),
    UploadComplete(UploadCompleteParams),
    CharacterGet(CharacterGetParams),
    CharacterList(CharacterListParams),
    CharacterDelete(CharacterDeleteParams),
    StoryResourcesCreate(CreateStoryResourcesParams),
    StoryResourcesGet(GetStoryResourcesParams),
    StoryResourcesList(ListStoryResourcesParams),
    StoryResourcesUpdate(UpdateStoryResourcesParams),
    StoryResourcesDelete(DeleteStoryResourcesParams),
    StoryGeneratePlan(GenerateStoryPlanParams),
    StoryGenerate(GenerateStoryParams),
    StoryGet(GetStoryParams),
    StoryList(ListStoriesParams),
    StoryDelete(DeleteStoryParams),
    StoryStartSession(StartSessionFromStoryParams),
    SessionGet(GetSessionParams),
    SessionList(ListSessionsParams),
    SessionDelete(DeleteSessionParams),
    SessionRunTurn(RunTurnParams),
    SessionUpdatePlayerDescription(UpdatePlayerDescriptionParams),
    SessionGetRuntimeSnapshot(GetRuntimeSnapshotParams),
    ConfigGetGlobal(ConfigGetGlobalParams),
    ConfigUpdateGlobal(ConfigUpdateGlobalParams),
    SessionGetConfig(SessionGetConfigParams),
    SessionUpdateConfig(SessionUpdateConfigParams),
}

impl RequestParams {
    pub const fn method(&self) -> RequestMethod {
        match self {
            Self::UploadInit(_) => RequestMethod::UploadInit,
            Self::UploadChunk(_) => RequestMethod::UploadChunk,
            Self::UploadComplete(_) => RequestMethod::UploadComplete,
            Self::CharacterGet(_) => RequestMethod::CharacterGet,
            Self::CharacterList(_) => RequestMethod::CharacterList,
            Self::CharacterDelete(_) => RequestMethod::CharacterDelete,
            Self::StoryResourcesCreate(_) => RequestMethod::StoryResourcesCreate,
            Self::StoryResourcesGet(_) => RequestMethod::StoryResourcesGet,
            Self::StoryResourcesList(_) => RequestMethod::StoryResourcesList,
            Self::StoryResourcesUpdate(_) => RequestMethod::StoryResourcesUpdate,
            Self::StoryResourcesDelete(_) => RequestMethod::StoryResourcesDelete,
            Self::StoryGeneratePlan(_) => RequestMethod::StoryGeneratePlan,
            Self::StoryGenerate(_) => RequestMethod::StoryGenerate,
            Self::StoryGet(_) => RequestMethod::StoryGet,
            Self::StoryList(_) => RequestMethod::StoryList,
            Self::StoryDelete(_) => RequestMethod::StoryDelete,
            Self::StoryStartSession(_) => RequestMethod::StoryStartSession,
            Self::SessionGet(_) => RequestMethod::SessionGet,
            Self::SessionList(_) => RequestMethod::SessionList,
            Self::SessionDelete(_) => RequestMethod::SessionDelete,
            Self::SessionRunTurn(_) => RequestMethod::SessionRunTurn,
            Self::SessionUpdatePlayerDescription(_) => {
                RequestMethod::SessionUpdatePlayerDescription
            }
            Self::SessionGetRuntimeSnapshot(_) => RequestMethod::SessionGetRuntimeSnapshot,
            Self::ConfigGetGlobal(_) => RequestMethod::ConfigGetGlobal,
            Self::ConfigUpdateGlobal(_) => RequestMethod::ConfigUpdateGlobal,
            Self::SessionGetConfig(_) => RequestMethod::SessionGetConfig,
            Self::SessionUpdateConfig(_) => RequestMethod::SessionUpdateConfig,
        }
    }

    pub(crate) fn to_value(&self) -> Result<Value, serde_json::Error> {
        match self {
            Self::UploadInit(params) => serde_json::to_value(params),
            Self::UploadChunk(params) => serde_json::to_value(params),
            Self::UploadComplete(params) => serde_json::to_value(params),
            Self::CharacterGet(params) => serde_json::to_value(params),
            Self::CharacterList(params) => serde_json::to_value(params),
            Self::CharacterDelete(params) => serde_json::to_value(params),
            Self::StoryResourcesCreate(params) => serde_json::to_value(params),
            Self::StoryResourcesGet(params) => serde_json::to_value(params),
            Self::StoryResourcesList(params) => serde_json::to_value(params),
            Self::StoryResourcesUpdate(params) => serde_json::to_value(params),
            Self::StoryResourcesDelete(params) => serde_json::to_value(params),
            Self::StoryGeneratePlan(params) => serde_json::to_value(params),
            Self::StoryGenerate(params) => serde_json::to_value(params),
            Self::StoryGet(params) => serde_json::to_value(params),
            Self::StoryList(params) => serde_json::to_value(params),
            Self::StoryDelete(params) => serde_json::to_value(params),
            Self::StoryStartSession(params) => serde_json::to_value(params),
            Self::SessionGet(params) => serde_json::to_value(params),
            Self::SessionList(params) => serde_json::to_value(params),
            Self::SessionDelete(params) => serde_json::to_value(params),
            Self::SessionRunTurn(params) => serde_json::to_value(params),
            Self::SessionUpdatePlayerDescription(params) => serde_json::to_value(params),
            Self::SessionGetRuntimeSnapshot(params) => serde_json::to_value(params),
            Self::ConfigGetGlobal(params) => serde_json::to_value(params),
            Self::ConfigUpdateGlobal(params) => serde_json::to_value(params),
            Self::SessionGetConfig(params) => serde_json::to_value(params),
            Self::SessionUpdateConfig(params) => serde_json::to_value(params),
        }
    }

    pub(crate) fn from_method_and_value(
        method: RequestMethod,
        value: Value,
    ) -> Result<Self, serde_json::Error> {
        match method {
            RequestMethod::UploadInit => serde_json::from_value(value).map(Self::UploadInit),
            RequestMethod::UploadChunk => serde_json::from_value(value).map(Self::UploadChunk),
            RequestMethod::UploadComplete => {
                serde_json::from_value(value).map(Self::UploadComplete)
            }
            RequestMethod::CharacterGet => serde_json::from_value(value).map(Self::CharacterGet),
            RequestMethod::CharacterList => serde_json::from_value(value).map(Self::CharacterList),
            RequestMethod::CharacterDelete => {
                serde_json::from_value(value).map(Self::CharacterDelete)
            }
            RequestMethod::StoryResourcesCreate => {
                serde_json::from_value(value).map(Self::StoryResourcesCreate)
            }
            RequestMethod::StoryResourcesGet => {
                serde_json::from_value(value).map(Self::StoryResourcesGet)
            }
            RequestMethod::StoryResourcesList => {
                serde_json::from_value(value).map(Self::StoryResourcesList)
            }
            RequestMethod::StoryResourcesUpdate => {
                serde_json::from_value(value).map(Self::StoryResourcesUpdate)
            }
            RequestMethod::StoryResourcesDelete => {
                serde_json::from_value(value).map(Self::StoryResourcesDelete)
            }
            RequestMethod::StoryGeneratePlan => {
                serde_json::from_value(value).map(Self::StoryGeneratePlan)
            }
            RequestMethod::StoryGenerate => serde_json::from_value(value).map(Self::StoryGenerate),
            RequestMethod::StoryGet => serde_json::from_value(value).map(Self::StoryGet),
            RequestMethod::StoryList => serde_json::from_value(value).map(Self::StoryList),
            RequestMethod::StoryDelete => serde_json::from_value(value).map(Self::StoryDelete),
            RequestMethod::StoryStartSession => {
                serde_json::from_value(value).map(Self::StoryStartSession)
            }
            RequestMethod::SessionGet => serde_json::from_value(value).map(Self::SessionGet),
            RequestMethod::SessionList => serde_json::from_value(value).map(Self::SessionList),
            RequestMethod::SessionDelete => serde_json::from_value(value).map(Self::SessionDelete),
            RequestMethod::SessionRunTurn => {
                serde_json::from_value(value).map(Self::SessionRunTurn)
            }
            RequestMethod::SessionUpdatePlayerDescription => {
                serde_json::from_value(value).map(Self::SessionUpdatePlayerDescription)
            }
            RequestMethod::SessionGetRuntimeSnapshot => {
                serde_json::from_value(value).map(Self::SessionGetRuntimeSnapshot)
            }
            RequestMethod::ConfigGetGlobal => {
                serde_json::from_value(value).map(Self::ConfigGetGlobal)
            }
            RequestMethod::ConfigUpdateGlobal => {
                serde_json::from_value(value).map(Self::ConfigUpdateGlobal)
            }
            RequestMethod::SessionGetConfig => {
                serde_json::from_value(value).map(Self::SessionGetConfig)
            }
            RequestMethod::SessionUpdateConfig => {
                serde_json::from_value(value).map(Self::SessionUpdateConfig)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CharacterGetParams {
    pub character_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct CharacterListParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CharacterDeleteParams {
    pub character_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UploadTargetKind {
    CharacterCard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UploadInitParams {
    pub target_kind: UploadTargetKind,
    pub file_name: String,
    pub content_type: String,
    pub total_size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UploadChunkParams {
    pub upload_id: String,
    pub chunk_index: u64,
    pub offset: u64,
    pub payload_base64: String,
    pub is_last: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UploadCompleteParams {
    pub upload_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateStoryResourcesParams {
    pub story_concept: String,
    pub character_ids: Vec<String>,
    pub player_state_schema_seed: PlayerStateSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_state_schema_seed: Option<WorldStateSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetStoryResourcesParams {
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ListStoryResourcesParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateStoryResourcesParams {
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub story_concept: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_state_schema_seed: Option<PlayerStateSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_state_schema_seed: Option<WorldStateSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planned_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeleteStoryResourcesParams {
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GenerateStoryPlanParams {
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner_api_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GenerateStoryParams {
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architect_api_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetStoryParams {
    pub story_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ListStoriesParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeleteStoryParams {
    pub story_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StartSessionFromStoryParams {
    pub story_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    pub player_description: String,
    #[serde(default)]
    pub config_mode: SessionConfigMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_api_ids: Option<AgentApiIds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct GetSessionParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ListSessionsParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct DeleteSessionParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunTurnParams {
    pub player_input: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_overrides: Option<AgentApiIdOverrides>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpdatePlayerDescriptionParams {
    pub player_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct GetRuntimeSnapshotParams {}
