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
    #[serde(rename = "story_resources.create")]
    StoryResourcesCreate,
    #[serde(rename = "story_resources.update")]
    StoryResourcesUpdate,
    #[serde(rename = "story.generate_plan")]
    StoryGeneratePlan,
    #[serde(rename = "story.generate")]
    StoryGenerate,
    #[serde(rename = "story.start_session")]
    StoryStartSession,
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
    StoryResourcesCreate(CreateStoryResourcesParams),
    StoryResourcesUpdate(UpdateStoryResourcesParams),
    StoryGeneratePlan(GenerateStoryPlanParams),
    StoryGenerate(GenerateStoryParams),
    StoryStartSession(StartSessionFromStoryParams),
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
            Self::StoryResourcesCreate(_) => RequestMethod::StoryResourcesCreate,
            Self::StoryResourcesUpdate(_) => RequestMethod::StoryResourcesUpdate,
            Self::StoryGeneratePlan(_) => RequestMethod::StoryGeneratePlan,
            Self::StoryGenerate(_) => RequestMethod::StoryGenerate,
            Self::StoryStartSession(_) => RequestMethod::StoryStartSession,
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
            Self::StoryResourcesCreate(params) => serde_json::to_value(params),
            Self::StoryResourcesUpdate(params) => serde_json::to_value(params),
            Self::StoryGeneratePlan(params) => serde_json::to_value(params),
            Self::StoryGenerate(params) => serde_json::to_value(params),
            Self::StoryStartSession(params) => serde_json::to_value(params),
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
            RequestMethod::StoryResourcesCreate => {
                serde_json::from_value(value).map(Self::StoryResourcesCreate)
            }
            RequestMethod::StoryResourcesUpdate => {
                serde_json::from_value(value).map(Self::StoryResourcesUpdate)
            }
            RequestMethod::StoryGeneratePlan => {
                serde_json::from_value(value).map(Self::StoryGeneratePlan)
            }
            RequestMethod::StoryGenerate => serde_json::from_value(value).map(Self::StoryGenerate),
            RequestMethod::StoryStartSession => {
                serde_json::from_value(value).map(Self::StoryStartSession)
            }
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
    pub architect_api_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StartSessionFromStoryParams {
    pub story_id: String,
    pub player_description: String,
    #[serde(default)]
    pub config_mode: SessionConfigMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_api_ids: Option<AgentApiIds>,
}

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
