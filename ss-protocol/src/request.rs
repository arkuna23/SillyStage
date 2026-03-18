use crate::api::{
    ApiCreateParams, ApiDeleteParams, ApiGetParams, ApiListModelsParams, ApiListParams,
    ApiUpdateParams,
};
use crate::api_group::{
    ApiGroupCreateParams, ApiGroupDeleteParams, ApiGroupGetParams, ApiGroupListParams,
    ApiGroupUpdateParams,
};
use crate::character::CharacterCardContent;
use crate::config::{ConfigGetGlobalParams, SessionGetConfigParams, SessionUpdateConfigParams};
use crate::data_package::{
    DataPackageExportPrepareParams, DataPackageImportCommitParams, DataPackageImportPrepareParams,
};
use crate::lorebook::{
    LorebookCreateParams, LorebookDeleteParams, LorebookEntryCreateParams,
    LorebookEntryDeleteParams, LorebookEntryGetParams, LorebookEntryListParams,
    LorebookEntryUpdateParams, LorebookGetParams, LorebookListParams, LorebookUpdateParams,
};
use crate::player_profile::{
    PlayerProfileCreateParams, PlayerProfileDeleteParams, PlayerProfileGetParams,
    PlayerProfileListParams, PlayerProfileUpdateParams,
};
use crate::preset::{
    PresetCreateParams, PresetDeleteParams, PresetEntryCreateParams, PresetEntryDeleteParams,
    PresetEntryUpdateParams, PresetGetParams, PresetListParams, PresetUpdateParams,
};
use crate::reply_suggestion::SuggestRepliesParams;
use crate::schema::{
    SchemaCreateParams, SchemaDeleteParams, SchemaGetParams, SchemaListParams, SchemaUpdateParams,
};
use crate::session_character::{
    DeleteSessionCharacterParams, EnterSessionCharacterSceneParams, GetSessionCharacterParams,
    LeaveSessionCharacterSceneParams, ListSessionCharactersParams, UpdateSessionCharacterParams,
};
use crate::session_message::{
    CreateSessionMessageParams, DeleteSessionMessageParams, GetSessionMessageParams,
    ListSessionMessagesParams, UpdateSessionMessageParams,
};
use crate::session_variable::{GetSessionVariablesParams, UpdateSessionVariablesParams};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use story::{CommonVariableDefinition, StoryGraph};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RequestMethod {
    #[serde(rename = "api.create")]
    ApiCreate,
    #[serde(rename = "api.get")]
    ApiGet,
    #[serde(rename = "api.list")]
    ApiList,
    #[serde(rename = "api.list_models")]
    ApiListModels,
    #[serde(rename = "api.update")]
    ApiUpdate,
    #[serde(rename = "api.delete")]
    ApiDelete,
    #[serde(rename = "api_group.create")]
    ApiGroupCreate,
    #[serde(rename = "api_group.get")]
    ApiGroupGet,
    #[serde(rename = "api_group.list")]
    ApiGroupList,
    #[serde(rename = "api_group.update")]
    ApiGroupUpdate,
    #[serde(rename = "api_group.delete")]
    ApiGroupDelete,
    #[serde(rename = "preset.create")]
    PresetCreate,
    #[serde(rename = "preset.get")]
    PresetGet,
    #[serde(rename = "preset.list")]
    PresetList,
    #[serde(rename = "preset.update")]
    PresetUpdate,
    #[serde(rename = "preset.delete")]
    PresetDelete,
    #[serde(rename = "preset_entry.create")]
    PresetEntryCreate,
    #[serde(rename = "preset_entry.update")]
    PresetEntryUpdate,
    #[serde(rename = "preset_entry.delete")]
    PresetEntryDelete,
    #[serde(rename = "schema.create")]
    SchemaCreate,
    #[serde(rename = "schema.get")]
    SchemaGet,
    #[serde(rename = "schema.list")]
    SchemaList,
    #[serde(rename = "schema.update")]
    SchemaUpdate,
    #[serde(rename = "schema.delete")]
    SchemaDelete,
    #[serde(rename = "lorebook.create")]
    LorebookCreate,
    #[serde(rename = "lorebook.get")]
    LorebookGet,
    #[serde(rename = "lorebook.list")]
    LorebookList,
    #[serde(rename = "lorebook.update")]
    LorebookUpdate,
    #[serde(rename = "lorebook.delete")]
    LorebookDelete,
    #[serde(rename = "lorebook_entry.create")]
    LorebookEntryCreate,
    #[serde(rename = "lorebook_entry.get")]
    LorebookEntryGet,
    #[serde(rename = "lorebook_entry.list")]
    LorebookEntryList,
    #[serde(rename = "lorebook_entry.update")]
    LorebookEntryUpdate,
    #[serde(rename = "lorebook_entry.delete")]
    LorebookEntryDelete,
    #[serde(rename = "player_profile.create")]
    PlayerProfileCreate,
    #[serde(rename = "player_profile.get")]
    PlayerProfileGet,
    #[serde(rename = "player_profile.list")]
    PlayerProfileList,
    #[serde(rename = "player_profile.update")]
    PlayerProfileUpdate,
    #[serde(rename = "player_profile.delete")]
    PlayerProfileDelete,
    #[serde(rename = "character.create")]
    CharacterCreate,
    #[serde(rename = "character.get")]
    CharacterGet,
    #[serde(rename = "character.update")]
    CharacterUpdate,
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
    #[serde(rename = "story.update")]
    StoryUpdate,
    #[serde(rename = "story.update_graph")]
    StoryUpdateGraph,
    #[serde(rename = "story.list")]
    StoryList,
    #[serde(rename = "story.delete")]
    StoryDelete,
    #[serde(rename = "story_draft.start")]
    StoryDraftStart,
    #[serde(rename = "story_draft.get")]
    StoryDraftGet,
    #[serde(rename = "story_draft.list")]
    StoryDraftList,
    #[serde(rename = "story_draft.update_graph")]
    StoryDraftUpdateGraph,
    #[serde(rename = "story_draft.continue")]
    StoryDraftContinue,
    #[serde(rename = "story_draft.finalize")]
    StoryDraftFinalize,
    #[serde(rename = "story_draft.delete")]
    StoryDraftDelete,
    #[serde(rename = "story.start_session")]
    StoryStartSession,
    #[serde(rename = "session.get")]
    SessionGet,
    #[serde(rename = "session.update")]
    SessionUpdate,
    #[serde(rename = "session.list")]
    SessionList,
    #[serde(rename = "session.delete")]
    SessionDelete,
    #[serde(rename = "session_message.create")]
    SessionMessageCreate,
    #[serde(rename = "session_message.get")]
    SessionMessageGet,
    #[serde(rename = "session_message.list")]
    SessionMessageList,
    #[serde(rename = "session_message.update")]
    SessionMessageUpdate,
    #[serde(rename = "session_message.delete")]
    SessionMessageDelete,
    #[serde(rename = "session_character.get")]
    SessionCharacterGet,
    #[serde(rename = "session_character.list")]
    SessionCharacterList,
    #[serde(rename = "session_character.update")]
    SessionCharacterUpdate,
    #[serde(rename = "session_character.delete")]
    SessionCharacterDelete,
    #[serde(rename = "session_character.enter_scene")]
    SessionCharacterEnterScene,
    #[serde(rename = "session_character.leave_scene")]
    SessionCharacterLeaveScene,
    #[serde(rename = "session.run_turn")]
    SessionRunTurn,
    #[serde(rename = "session.get_variables")]
    SessionGetVariables,
    #[serde(rename = "session.update_variables")]
    SessionUpdateVariables,
    #[serde(rename = "session.suggest_replies")]
    SessionSuggestReplies,
    #[serde(rename = "session.set_player_profile")]
    SessionSetPlayerProfile,
    #[serde(rename = "session.update_player_description")]
    SessionUpdatePlayerDescription,
    #[serde(rename = "session.get_runtime_snapshot")]
    SessionGetRuntimeSnapshot,
    #[serde(rename = "config.get_global")]
    ConfigGetGlobal,
    #[serde(rename = "session.get_config")]
    SessionGetConfig,
    #[serde(rename = "session.update_config")]
    SessionUpdateConfig,
    #[serde(rename = "dashboard.get")]
    DashboardGet,
    #[serde(rename = "data_package.export_prepare")]
    DataPackageExportPrepare,
    #[serde(rename = "data_package.import_prepare")]
    DataPackageImportPrepare,
    #[serde(rename = "data_package.import_commit")]
    DataPackageImportCommit,
}

#[derive(Debug, Clone)]
pub enum RequestParams {
    ApiCreate(ApiCreateParams),
    ApiGet(ApiGetParams),
    ApiList(ApiListParams),
    ApiListModels(ApiListModelsParams),
    ApiUpdate(ApiUpdateParams),
    ApiDelete(ApiDeleteParams),
    ApiGroupCreate(ApiGroupCreateParams),
    ApiGroupGet(ApiGroupGetParams),
    ApiGroupList(ApiGroupListParams),
    ApiGroupUpdate(ApiGroupUpdateParams),
    ApiGroupDelete(ApiGroupDeleteParams),
    PresetCreate(PresetCreateParams),
    PresetGet(PresetGetParams),
    PresetList(PresetListParams),
    PresetUpdate(PresetUpdateParams),
    PresetDelete(PresetDeleteParams),
    PresetEntryCreate(PresetEntryCreateParams),
    PresetEntryUpdate(PresetEntryUpdateParams),
    PresetEntryDelete(PresetEntryDeleteParams),
    SchemaCreate(SchemaCreateParams),
    SchemaGet(SchemaGetParams),
    SchemaList(SchemaListParams),
    SchemaUpdate(SchemaUpdateParams),
    SchemaDelete(SchemaDeleteParams),
    LorebookCreate(LorebookCreateParams),
    LorebookGet(LorebookGetParams),
    LorebookList(LorebookListParams),
    LorebookUpdate(LorebookUpdateParams),
    LorebookDelete(LorebookDeleteParams),
    LorebookEntryCreate(LorebookEntryCreateParams),
    LorebookEntryGet(LorebookEntryGetParams),
    LorebookEntryList(LorebookEntryListParams),
    LorebookEntryUpdate(LorebookEntryUpdateParams),
    LorebookEntryDelete(LorebookEntryDeleteParams),
    PlayerProfileCreate(PlayerProfileCreateParams),
    PlayerProfileGet(PlayerProfileGetParams),
    PlayerProfileList(PlayerProfileListParams),
    PlayerProfileUpdate(PlayerProfileUpdateParams),
    PlayerProfileDelete(PlayerProfileDeleteParams),
    CharacterCreate(CharacterCreateParams),
    CharacterGet(CharacterGetParams),
    CharacterUpdate(CharacterUpdateParams),
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
    StoryUpdate(UpdateStoryParams),
    StoryUpdateGraph(UpdateStoryGraphParams),
    StoryList(ListStoriesParams),
    StoryDelete(DeleteStoryParams),
    StoryDraftStart(StartStoryDraftParams),
    StoryDraftGet(GetStoryDraftParams),
    StoryDraftList(ListStoryDraftsParams),
    StoryDraftUpdateGraph(UpdateStoryDraftGraphParams),
    StoryDraftContinue(ContinueStoryDraftParams),
    StoryDraftFinalize(FinalizeStoryDraftParams),
    StoryDraftDelete(DeleteStoryDraftParams),
    StoryStartSession(StartSessionFromStoryParams),
    SessionGet(GetSessionParams),
    SessionUpdate(UpdateSessionParams),
    SessionList(ListSessionsParams),
    SessionDelete(DeleteSessionParams),
    SessionMessageCreate(CreateSessionMessageParams),
    SessionMessageGet(GetSessionMessageParams),
    SessionMessageList(ListSessionMessagesParams),
    SessionMessageUpdate(UpdateSessionMessageParams),
    SessionMessageDelete(DeleteSessionMessageParams),
    SessionCharacterGet(GetSessionCharacterParams),
    SessionCharacterList(ListSessionCharactersParams),
    SessionCharacterUpdate(UpdateSessionCharacterParams),
    SessionCharacterDelete(DeleteSessionCharacterParams),
    SessionCharacterEnterScene(EnterSessionCharacterSceneParams),
    SessionCharacterLeaveScene(LeaveSessionCharacterSceneParams),
    SessionRunTurn(RunTurnParams),
    SessionGetVariables(GetSessionVariablesParams),
    SessionUpdateVariables(UpdateSessionVariablesParams),
    SessionSuggestReplies(SuggestRepliesParams),
    SessionSetPlayerProfile(SetPlayerProfileParams),
    SessionUpdatePlayerDescription(UpdatePlayerDescriptionParams),
    SessionGetRuntimeSnapshot(GetRuntimeSnapshotParams),
    ConfigGetGlobal(ConfigGetGlobalParams),
    SessionGetConfig(SessionGetConfigParams),
    SessionUpdateConfig(SessionUpdateConfigParams),
    DashboardGet(DashboardGetParams),
    DataPackageExportPrepare(DataPackageExportPrepareParams),
    DataPackageImportPrepare(DataPackageImportPrepareParams),
    DataPackageImportCommit(DataPackageImportCommitParams),
}

impl RequestParams {
    pub const fn method(&self) -> RequestMethod {
        match self {
            Self::ApiCreate(_) => RequestMethod::ApiCreate,
            Self::ApiGet(_) => RequestMethod::ApiGet,
            Self::ApiList(_) => RequestMethod::ApiList,
            Self::ApiListModels(_) => RequestMethod::ApiListModels,
            Self::ApiUpdate(_) => RequestMethod::ApiUpdate,
            Self::ApiDelete(_) => RequestMethod::ApiDelete,
            Self::ApiGroupCreate(_) => RequestMethod::ApiGroupCreate,
            Self::ApiGroupGet(_) => RequestMethod::ApiGroupGet,
            Self::ApiGroupList(_) => RequestMethod::ApiGroupList,
            Self::ApiGroupUpdate(_) => RequestMethod::ApiGroupUpdate,
            Self::ApiGroupDelete(_) => RequestMethod::ApiGroupDelete,
            Self::PresetCreate(_) => RequestMethod::PresetCreate,
            Self::PresetGet(_) => RequestMethod::PresetGet,
            Self::PresetList(_) => RequestMethod::PresetList,
            Self::PresetUpdate(_) => RequestMethod::PresetUpdate,
            Self::PresetDelete(_) => RequestMethod::PresetDelete,
            Self::PresetEntryCreate(_) => RequestMethod::PresetEntryCreate,
            Self::PresetEntryUpdate(_) => RequestMethod::PresetEntryUpdate,
            Self::PresetEntryDelete(_) => RequestMethod::PresetEntryDelete,
            Self::SchemaCreate(_) => RequestMethod::SchemaCreate,
            Self::SchemaGet(_) => RequestMethod::SchemaGet,
            Self::SchemaList(_) => RequestMethod::SchemaList,
            Self::SchemaUpdate(_) => RequestMethod::SchemaUpdate,
            Self::SchemaDelete(_) => RequestMethod::SchemaDelete,
            Self::LorebookCreate(_) => RequestMethod::LorebookCreate,
            Self::LorebookGet(_) => RequestMethod::LorebookGet,
            Self::LorebookList(_) => RequestMethod::LorebookList,
            Self::LorebookUpdate(_) => RequestMethod::LorebookUpdate,
            Self::LorebookDelete(_) => RequestMethod::LorebookDelete,
            Self::LorebookEntryCreate(_) => RequestMethod::LorebookEntryCreate,
            Self::LorebookEntryGet(_) => RequestMethod::LorebookEntryGet,
            Self::LorebookEntryList(_) => RequestMethod::LorebookEntryList,
            Self::LorebookEntryUpdate(_) => RequestMethod::LorebookEntryUpdate,
            Self::LorebookEntryDelete(_) => RequestMethod::LorebookEntryDelete,
            Self::PlayerProfileCreate(_) => RequestMethod::PlayerProfileCreate,
            Self::PlayerProfileGet(_) => RequestMethod::PlayerProfileGet,
            Self::PlayerProfileList(_) => RequestMethod::PlayerProfileList,
            Self::PlayerProfileUpdate(_) => RequestMethod::PlayerProfileUpdate,
            Self::PlayerProfileDelete(_) => RequestMethod::PlayerProfileDelete,
            Self::CharacterCreate(_) => RequestMethod::CharacterCreate,
            Self::CharacterGet(_) => RequestMethod::CharacterGet,
            Self::CharacterUpdate(_) => RequestMethod::CharacterUpdate,
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
            Self::StoryUpdate(_) => RequestMethod::StoryUpdate,
            Self::StoryUpdateGraph(_) => RequestMethod::StoryUpdateGraph,
            Self::StoryList(_) => RequestMethod::StoryList,
            Self::StoryDelete(_) => RequestMethod::StoryDelete,
            Self::StoryDraftStart(_) => RequestMethod::StoryDraftStart,
            Self::StoryDraftGet(_) => RequestMethod::StoryDraftGet,
            Self::StoryDraftList(_) => RequestMethod::StoryDraftList,
            Self::StoryDraftUpdateGraph(_) => RequestMethod::StoryDraftUpdateGraph,
            Self::StoryDraftContinue(_) => RequestMethod::StoryDraftContinue,
            Self::StoryDraftFinalize(_) => RequestMethod::StoryDraftFinalize,
            Self::StoryDraftDelete(_) => RequestMethod::StoryDraftDelete,
            Self::StoryStartSession(_) => RequestMethod::StoryStartSession,
            Self::SessionGet(_) => RequestMethod::SessionGet,
            Self::SessionUpdate(_) => RequestMethod::SessionUpdate,
            Self::SessionList(_) => RequestMethod::SessionList,
            Self::SessionDelete(_) => RequestMethod::SessionDelete,
            Self::SessionMessageCreate(_) => RequestMethod::SessionMessageCreate,
            Self::SessionMessageGet(_) => RequestMethod::SessionMessageGet,
            Self::SessionMessageList(_) => RequestMethod::SessionMessageList,
            Self::SessionMessageUpdate(_) => RequestMethod::SessionMessageUpdate,
            Self::SessionMessageDelete(_) => RequestMethod::SessionMessageDelete,
            Self::SessionCharacterGet(_) => RequestMethod::SessionCharacterGet,
            Self::SessionCharacterList(_) => RequestMethod::SessionCharacterList,
            Self::SessionCharacterUpdate(_) => RequestMethod::SessionCharacterUpdate,
            Self::SessionCharacterDelete(_) => RequestMethod::SessionCharacterDelete,
            Self::SessionCharacterEnterScene(_) => RequestMethod::SessionCharacterEnterScene,
            Self::SessionCharacterLeaveScene(_) => RequestMethod::SessionCharacterLeaveScene,
            Self::SessionRunTurn(_) => RequestMethod::SessionRunTurn,
            Self::SessionGetVariables(_) => RequestMethod::SessionGetVariables,
            Self::SessionUpdateVariables(_) => RequestMethod::SessionUpdateVariables,
            Self::SessionSuggestReplies(_) => RequestMethod::SessionSuggestReplies,
            Self::SessionSetPlayerProfile(_) => RequestMethod::SessionSetPlayerProfile,
            Self::SessionUpdatePlayerDescription(_) => {
                RequestMethod::SessionUpdatePlayerDescription
            }
            Self::SessionGetRuntimeSnapshot(_) => RequestMethod::SessionGetRuntimeSnapshot,
            Self::ConfigGetGlobal(_) => RequestMethod::ConfigGetGlobal,
            Self::SessionGetConfig(_) => RequestMethod::SessionGetConfig,
            Self::SessionUpdateConfig(_) => RequestMethod::SessionUpdateConfig,
            Self::DashboardGet(_) => RequestMethod::DashboardGet,
            Self::DataPackageExportPrepare(_) => RequestMethod::DataPackageExportPrepare,
            Self::DataPackageImportPrepare(_) => RequestMethod::DataPackageImportPrepare,
            Self::DataPackageImportCommit(_) => RequestMethod::DataPackageImportCommit,
        }
    }

    pub(crate) fn to_value(&self) -> Result<Value, serde_json::Error> {
        match self {
            Self::ApiCreate(params) => serde_json::to_value(params),
            Self::ApiGet(params) => serde_json::to_value(params),
            Self::ApiList(params) => serde_json::to_value(params),
            Self::ApiListModels(params) => serde_json::to_value(params),
            Self::ApiUpdate(params) => serde_json::to_value(params),
            Self::ApiDelete(params) => serde_json::to_value(params),
            Self::ApiGroupCreate(params) => serde_json::to_value(params),
            Self::ApiGroupGet(params) => serde_json::to_value(params),
            Self::ApiGroupList(params) => serde_json::to_value(params),
            Self::ApiGroupUpdate(params) => serde_json::to_value(params),
            Self::ApiGroupDelete(params) => serde_json::to_value(params),
            Self::PresetCreate(params) => serde_json::to_value(params),
            Self::PresetGet(params) => serde_json::to_value(params),
            Self::PresetList(params) => serde_json::to_value(params),
            Self::PresetUpdate(params) => serde_json::to_value(params),
            Self::PresetDelete(params) => serde_json::to_value(params),
            Self::PresetEntryCreate(params) => serde_json::to_value(params),
            Self::PresetEntryUpdate(params) => serde_json::to_value(params),
            Self::PresetEntryDelete(params) => serde_json::to_value(params),
            Self::SchemaCreate(params) => serde_json::to_value(params),
            Self::SchemaGet(params) => serde_json::to_value(params),
            Self::SchemaList(params) => serde_json::to_value(params),
            Self::SchemaUpdate(params) => serde_json::to_value(params),
            Self::SchemaDelete(params) => serde_json::to_value(params),
            Self::LorebookCreate(params) => serde_json::to_value(params),
            Self::LorebookGet(params) => serde_json::to_value(params),
            Self::LorebookList(params) => serde_json::to_value(params),
            Self::LorebookUpdate(params) => serde_json::to_value(params),
            Self::LorebookDelete(params) => serde_json::to_value(params),
            Self::LorebookEntryCreate(params) => serde_json::to_value(params),
            Self::LorebookEntryGet(params) => serde_json::to_value(params),
            Self::LorebookEntryList(params) => serde_json::to_value(params),
            Self::LorebookEntryUpdate(params) => serde_json::to_value(params),
            Self::LorebookEntryDelete(params) => serde_json::to_value(params),
            Self::PlayerProfileCreate(params) => serde_json::to_value(params),
            Self::PlayerProfileGet(params) => serde_json::to_value(params),
            Self::PlayerProfileList(params) => serde_json::to_value(params),
            Self::PlayerProfileUpdate(params) => serde_json::to_value(params),
            Self::PlayerProfileDelete(params) => serde_json::to_value(params),
            Self::CharacterCreate(params) => serde_json::to_value(params),
            Self::CharacterGet(params) => serde_json::to_value(params),
            Self::CharacterUpdate(params) => serde_json::to_value(params),
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
            Self::StoryUpdate(params) => serde_json::to_value(params),
            Self::StoryUpdateGraph(params) => serde_json::to_value(params),
            Self::StoryList(params) => serde_json::to_value(params),
            Self::StoryDelete(params) => serde_json::to_value(params),
            Self::StoryDraftStart(params) => serde_json::to_value(params),
            Self::StoryDraftGet(params) => serde_json::to_value(params),
            Self::StoryDraftList(params) => serde_json::to_value(params),
            Self::StoryDraftUpdateGraph(params) => serde_json::to_value(params),
            Self::StoryDraftContinue(params) => serde_json::to_value(params),
            Self::StoryDraftFinalize(params) => serde_json::to_value(params),
            Self::StoryDraftDelete(params) => serde_json::to_value(params),
            Self::StoryStartSession(params) => serde_json::to_value(params),
            Self::SessionGet(params) => serde_json::to_value(params),
            Self::SessionUpdate(params) => serde_json::to_value(params),
            Self::SessionList(params) => serde_json::to_value(params),
            Self::SessionDelete(params) => serde_json::to_value(params),
            Self::SessionMessageCreate(params) => serde_json::to_value(params),
            Self::SessionMessageGet(params) => serde_json::to_value(params),
            Self::SessionMessageList(params) => serde_json::to_value(params),
            Self::SessionMessageUpdate(params) => serde_json::to_value(params),
            Self::SessionMessageDelete(params) => serde_json::to_value(params),
            Self::SessionCharacterGet(params) => serde_json::to_value(params),
            Self::SessionCharacterList(params) => serde_json::to_value(params),
            Self::SessionCharacterUpdate(params) => serde_json::to_value(params),
            Self::SessionCharacterDelete(params) => serde_json::to_value(params),
            Self::SessionCharacterEnterScene(params) => serde_json::to_value(params),
            Self::SessionCharacterLeaveScene(params) => serde_json::to_value(params),
            Self::SessionRunTurn(params) => serde_json::to_value(params),
            Self::SessionGetVariables(params) => serde_json::to_value(params),
            Self::SessionUpdateVariables(params) => serde_json::to_value(params),
            Self::SessionSuggestReplies(params) => serde_json::to_value(params),
            Self::SessionSetPlayerProfile(params) => serde_json::to_value(params),
            Self::SessionUpdatePlayerDescription(params) => serde_json::to_value(params),
            Self::SessionGetRuntimeSnapshot(params) => serde_json::to_value(params),
            Self::ConfigGetGlobal(params) => serde_json::to_value(params),
            Self::SessionGetConfig(params) => serde_json::to_value(params),
            Self::SessionUpdateConfig(params) => serde_json::to_value(params),
            Self::DashboardGet(params) => serde_json::to_value(params),
            Self::DataPackageExportPrepare(params) => serde_json::to_value(params),
            Self::DataPackageImportPrepare(params) => serde_json::to_value(params),
            Self::DataPackageImportCommit(params) => serde_json::to_value(params),
        }
    }

    pub(crate) fn from_method_and_value(
        method: RequestMethod,
        value: Value,
    ) -> Result<Self, serde_json::Error> {
        match method {
            RequestMethod::ApiCreate => serde_json::from_value(value).map(Self::ApiCreate),
            RequestMethod::ApiGet => serde_json::from_value(value).map(Self::ApiGet),
            RequestMethod::ApiList => serde_json::from_value(value).map(Self::ApiList),
            RequestMethod::ApiListModels => serde_json::from_value(value).map(Self::ApiListModels),
            RequestMethod::ApiUpdate => serde_json::from_value(value).map(Self::ApiUpdate),
            RequestMethod::ApiDelete => serde_json::from_value(value).map(Self::ApiDelete),
            RequestMethod::ApiGroupCreate => {
                serde_json::from_value(value).map(Self::ApiGroupCreate)
            }
            RequestMethod::ApiGroupGet => serde_json::from_value(value).map(Self::ApiGroupGet),
            RequestMethod::ApiGroupList => serde_json::from_value(value).map(Self::ApiGroupList),
            RequestMethod::ApiGroupUpdate => {
                serde_json::from_value(value).map(Self::ApiGroupUpdate)
            }
            RequestMethod::ApiGroupDelete => {
                serde_json::from_value(value).map(Self::ApiGroupDelete)
            }
            RequestMethod::PresetCreate => serde_json::from_value(value).map(Self::PresetCreate),
            RequestMethod::PresetGet => serde_json::from_value(value).map(Self::PresetGet),
            RequestMethod::PresetList => serde_json::from_value(value).map(Self::PresetList),
            RequestMethod::PresetUpdate => serde_json::from_value(value).map(Self::PresetUpdate),
            RequestMethod::PresetDelete => serde_json::from_value(value).map(Self::PresetDelete),
            RequestMethod::PresetEntryCreate => {
                serde_json::from_value(value).map(Self::PresetEntryCreate)
            }
            RequestMethod::PresetEntryUpdate => {
                serde_json::from_value(value).map(Self::PresetEntryUpdate)
            }
            RequestMethod::PresetEntryDelete => {
                serde_json::from_value(value).map(Self::PresetEntryDelete)
            }
            RequestMethod::SchemaCreate => serde_json::from_value(value).map(Self::SchemaCreate),
            RequestMethod::SchemaGet => serde_json::from_value(value).map(Self::SchemaGet),
            RequestMethod::SchemaList => serde_json::from_value(value).map(Self::SchemaList),
            RequestMethod::SchemaUpdate => serde_json::from_value(value).map(Self::SchemaUpdate),
            RequestMethod::SchemaDelete => serde_json::from_value(value).map(Self::SchemaDelete),
            RequestMethod::LorebookCreate => {
                serde_json::from_value(value).map(Self::LorebookCreate)
            }
            RequestMethod::LorebookGet => serde_json::from_value(value).map(Self::LorebookGet),
            RequestMethod::LorebookList => serde_json::from_value(value).map(Self::LorebookList),
            RequestMethod::LorebookUpdate => {
                serde_json::from_value(value).map(Self::LorebookUpdate)
            }
            RequestMethod::LorebookDelete => {
                serde_json::from_value(value).map(Self::LorebookDelete)
            }
            RequestMethod::LorebookEntryCreate => {
                serde_json::from_value(value).map(Self::LorebookEntryCreate)
            }
            RequestMethod::LorebookEntryGet => {
                serde_json::from_value(value).map(Self::LorebookEntryGet)
            }
            RequestMethod::LorebookEntryList => {
                serde_json::from_value(value).map(Self::LorebookEntryList)
            }
            RequestMethod::LorebookEntryUpdate => {
                serde_json::from_value(value).map(Self::LorebookEntryUpdate)
            }
            RequestMethod::LorebookEntryDelete => {
                serde_json::from_value(value).map(Self::LorebookEntryDelete)
            }
            RequestMethod::PlayerProfileCreate => {
                serde_json::from_value(value).map(Self::PlayerProfileCreate)
            }
            RequestMethod::PlayerProfileGet => {
                serde_json::from_value(value).map(Self::PlayerProfileGet)
            }
            RequestMethod::PlayerProfileList => {
                serde_json::from_value(value).map(Self::PlayerProfileList)
            }
            RequestMethod::PlayerProfileUpdate => {
                serde_json::from_value(value).map(Self::PlayerProfileUpdate)
            }
            RequestMethod::PlayerProfileDelete => {
                serde_json::from_value(value).map(Self::PlayerProfileDelete)
            }
            RequestMethod::CharacterCreate => {
                serde_json::from_value(value).map(Self::CharacterCreate)
            }
            RequestMethod::CharacterGet => serde_json::from_value(value).map(Self::CharacterGet),
            RequestMethod::CharacterUpdate => {
                serde_json::from_value(value).map(Self::CharacterUpdate)
            }
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
            RequestMethod::StoryUpdate => serde_json::from_value(value).map(Self::StoryUpdate),
            RequestMethod::StoryUpdateGraph => {
                serde_json::from_value(value).map(Self::StoryUpdateGraph)
            }
            RequestMethod::StoryList => serde_json::from_value(value).map(Self::StoryList),
            RequestMethod::StoryDelete => serde_json::from_value(value).map(Self::StoryDelete),
            RequestMethod::StoryDraftStart => {
                serde_json::from_value(value).map(Self::StoryDraftStart)
            }
            RequestMethod::StoryDraftGet => serde_json::from_value(value).map(Self::StoryDraftGet),
            RequestMethod::StoryDraftList => {
                serde_json::from_value(value).map(Self::StoryDraftList)
            }
            RequestMethod::StoryDraftUpdateGraph => {
                serde_json::from_value(value).map(Self::StoryDraftUpdateGraph)
            }
            RequestMethod::StoryDraftContinue => {
                serde_json::from_value(value).map(Self::StoryDraftContinue)
            }
            RequestMethod::StoryDraftFinalize => {
                serde_json::from_value(value).map(Self::StoryDraftFinalize)
            }
            RequestMethod::StoryDraftDelete => {
                serde_json::from_value(value).map(Self::StoryDraftDelete)
            }
            RequestMethod::StoryStartSession => {
                serde_json::from_value(value).map(Self::StoryStartSession)
            }
            RequestMethod::SessionGet => serde_json::from_value(value).map(Self::SessionGet),
            RequestMethod::SessionUpdate => serde_json::from_value(value).map(Self::SessionUpdate),
            RequestMethod::SessionList => serde_json::from_value(value).map(Self::SessionList),
            RequestMethod::SessionDelete => serde_json::from_value(value).map(Self::SessionDelete),
            RequestMethod::SessionMessageCreate => {
                serde_json::from_value(value).map(Self::SessionMessageCreate)
            }
            RequestMethod::SessionMessageGet => {
                serde_json::from_value(value).map(Self::SessionMessageGet)
            }
            RequestMethod::SessionMessageList => {
                serde_json::from_value(value).map(Self::SessionMessageList)
            }
            RequestMethod::SessionMessageUpdate => {
                serde_json::from_value(value).map(Self::SessionMessageUpdate)
            }
            RequestMethod::SessionMessageDelete => {
                serde_json::from_value(value).map(Self::SessionMessageDelete)
            }
            RequestMethod::SessionCharacterGet => {
                serde_json::from_value(value).map(Self::SessionCharacterGet)
            }
            RequestMethod::SessionCharacterList => {
                serde_json::from_value(value).map(Self::SessionCharacterList)
            }
            RequestMethod::SessionCharacterUpdate => {
                serde_json::from_value(value).map(Self::SessionCharacterUpdate)
            }
            RequestMethod::SessionCharacterDelete => {
                serde_json::from_value(value).map(Self::SessionCharacterDelete)
            }
            RequestMethod::SessionCharacterEnterScene => {
                serde_json::from_value(value).map(Self::SessionCharacterEnterScene)
            }
            RequestMethod::SessionCharacterLeaveScene => {
                serde_json::from_value(value).map(Self::SessionCharacterLeaveScene)
            }
            RequestMethod::SessionRunTurn => {
                serde_json::from_value(value).map(Self::SessionRunTurn)
            }
            RequestMethod::SessionGetVariables => {
                serde_json::from_value(value).map(Self::SessionGetVariables)
            }
            RequestMethod::SessionUpdateVariables => {
                serde_json::from_value(value).map(Self::SessionUpdateVariables)
            }
            RequestMethod::SessionSuggestReplies => {
                serde_json::from_value(value).map(Self::SessionSuggestReplies)
            }
            RequestMethod::SessionSetPlayerProfile => {
                serde_json::from_value(value).map(Self::SessionSetPlayerProfile)
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
            RequestMethod::SessionGetConfig => {
                serde_json::from_value(value).map(Self::SessionGetConfig)
            }
            RequestMethod::SessionUpdateConfig => {
                serde_json::from_value(value).map(Self::SessionUpdateConfig)
            }
            RequestMethod::DashboardGet => serde_json::from_value(value).map(Self::DashboardGet),
            RequestMethod::DataPackageExportPrepare => {
                serde_json::from_value(value).map(Self::DataPackageExportPrepare)
            }
            RequestMethod::DataPackageImportPrepare => {
                serde_json::from_value(value).map(Self::DataPackageImportPrepare)
            }
            RequestMethod::DataPackageImportCommit => {
                serde_json::from_value(value).map(Self::DataPackageImportCommit)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct DashboardGetParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CharacterGetParams {
    pub character_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterCreateParams {
    pub content: CharacterCardContent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CharacterUpdateParams {
    pub character_id: String,
    pub content: CharacterCardContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct CharacterListParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CharacterDeleteParams {
    pub character_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateStoryResourcesParams {
    pub story_concept: String,
    pub character_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_schema_id_seed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_schema_id_seed: Option<String>,
    #[serde(default)]
    pub lorebook_ids: Vec<String>,
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
    pub player_schema_id_seed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_schema_id_seed: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lorebook_ids: Option<Vec<String>>,
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
    pub api_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GenerateStoryParams {
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_variables: Option<Vec<CommonVariableDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetStoryParams {
    pub story_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct UpdateStoryParams {
    pub story_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_variables: Option<Vec<CommonVariableDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateStoryGraphParams {
    pub story_id: String,
    pub graph: StoryGraph,
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
pub struct StartStoryDraftParams {
    pub resource_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_variables: Option<Vec<CommonVariableDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GetStoryDraftParams {
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct ListStoryDraftsParams {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateStoryDraftGraphParams {
    pub draft_id: String,
    pub partial_graph: StoryGraph,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContinueStoryDraftParams {
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FinalizeStoryDraftParams {
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DeleteStoryDraftParams {
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct StartSessionFromStoryParams {
    pub story_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct GetSessionParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct UpdateSessionParams {
    pub display_name: String,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SetPlayerProfileParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_profile_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UpdatePlayerDescriptionParams {
    pub player_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct GetRuntimeSnapshotParams {}
