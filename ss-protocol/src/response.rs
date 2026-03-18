use engine::{EngineTurnResult, RuntimeSnapshot};
use serde::{Deserialize, Serialize};
use story::{CommonVariableDefinition, StoryGraph};

use crate::api::{ApiDeletedPayload, ApiModelsListedPayload, ApiPayload, ApisListedPayload};
use crate::api_group::{ApiGroupDeletedPayload, ApiGroupPayload, ApiGroupsListedPayload};
use crate::character::{CharacterCardContent, CharacterCardSummaryPayload, CharacterCoverMimeType};
use crate::config::{GlobalConfigPayload, SessionConfigPayload};
use crate::data_package::{
    DataPackageExportPreparedPayload, DataPackageImportCommittedPayload,
    DataPackageImportPreparedPayload,
};
use crate::lorebook::{
    LorebookDeletedPayload, LorebookEntriesListedPayload, LorebookEntryDeletedPayload,
    LorebookEntryPayload, LorebookPayload, LorebooksListedPayload,
};
use crate::player_profile::{
    PlayerProfileDeletedPayload, PlayerProfilePayload, PlayerProfilesListedPayload,
};
use crate::preset::{
    PresetDeletedPayload, PresetEntryDeletedPayload, PresetEntryPayload, PresetPayload,
    PresetsListedPayload,
};
use crate::reply_suggestion::SuggestedRepliesPayload;
use crate::schema::{SchemaDeletedPayload, SchemaPayload, SchemasListedPayload};
use crate::session_character::{
    SessionCharacterDeletedPayload, SessionCharacterPayload, SessionCharactersListedPayload,
};
use crate::session_message::{
    SessionMessageDeletedPayload, SessionMessagePayload, SessionMessagesListedPayload,
};
use crate::session_variable::SessionVariablesPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseResult {
    Api(Box<ApiPayload>),
    ApisListed(ApisListedPayload),
    ApiModelsListed(ApiModelsListedPayload),
    ApiDeleted(ApiDeletedPayload),
    ApiGroup(Box<ApiGroupPayload>),
    ApiGroupsListed(ApiGroupsListedPayload),
    ApiGroupDeleted(ApiGroupDeletedPayload),
    Preset(Box<PresetPayload>),
    PresetsListed(PresetsListedPayload),
    PresetDeleted(PresetDeletedPayload),
    PresetEntry(Box<PresetEntryPayload>),
    PresetEntryDeleted(PresetEntryDeletedPayload),
    Schema(Box<SchemaPayload>),
    SchemasListed(SchemasListedPayload),
    SchemaDeleted(SchemaDeletedPayload),
    Lorebook(Box<LorebookPayload>),
    LorebooksListed(LorebooksListedPayload),
    LorebookDeleted(LorebookDeletedPayload),
    LorebookEntry(Box<LorebookEntryPayload>),
    LorebookEntriesListed(LorebookEntriesListedPayload),
    LorebookEntryDeleted(LorebookEntryDeletedPayload),
    PlayerProfile(Box<PlayerProfilePayload>),
    PlayerProfilesListed(PlayerProfilesListedPayload),
    PlayerProfileDeleted(PlayerProfileDeletedPayload),
    CharacterCreated(CharacterCreatedPayload),
    Character(Box<CharacterSchemaPayload>),
    CharactersListed(CharactersListedPayload),
    CharacterDeleted(CharacterDeletedPayload),
    StoryResourcesCreated(Box<StoryResourcesPayload>),
    StoryResources(Box<StoryResourcesPayload>),
    StoryResourcesListed(StoryResourcesListedPayload),
    StoryResourcesUpdated(Box<StoryResourcesPayload>),
    StoryResourcesDeleted(StoryResourcesDeletedPayload),
    StoryPlanned(StoryPlannedPayload),
    StoryGenerated(Box<StoryGeneratedPayload>),
    Story(Box<StoryDetailPayload>),
    StoriesListed(StoriesListedPayload),
    StoryDeleted(StoryDeletedPayload),
    StoryDraft(Box<StoryDraftDetailPayload>),
    StoryDraftsListed(StoryDraftsListedPayload),
    StoryDraftDeleted(StoryDraftDeletedPayload),
    SessionStarted(Box<SessionStartedPayload>),
    Session(Box<SessionDetailPayload>),
    SessionsListed(SessionsListedPayload),
    SessionDeleted(SessionDeletedPayload),
    SuggestedReplies(SuggestedRepliesPayload),
    SessionCharacter(Box<SessionCharacterPayload>),
    SessionCharactersListed(SessionCharactersListedPayload),
    SessionCharacterDeleted(SessionCharacterDeletedPayload),
    SessionMessage(Box<SessionMessagePayload>),
    SessionMessagesListed(SessionMessagesListedPayload),
    SessionMessageDeleted(SessionMessageDeletedPayload),
    SessionVariables(Box<SessionVariablesPayload>),
    GlobalConfig(GlobalConfigPayload),
    SessionConfig(SessionConfigPayload),
    Dashboard(Box<DashboardPayload>),
    TurnStreamAccepted(TurnStreamAcceptedPayload),
    TurnCompleted(Box<TurnCompletedPayload>),
    PlayerDescriptionUpdated(Box<PlayerDescriptionUpdatedPayload>),
    RuntimeSnapshot(Box<RuntimeSnapshotPayload>),
    DataPackageExportPrepared(DataPackageExportPreparedPayload),
    DataPackageImportPrepared(DataPackageImportPreparedPayload),
    DataPackageImportCommitted(DataPackageImportCommittedPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterCreatedPayload {
    pub character_id: String,
    pub character_summary: CharacterCardSummaryPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterSchemaPayload {
    pub character_id: String,
    pub content: CharacterCardContent,
    pub cover_file_name: Option<String>,
    pub cover_mime_type: Option<CharacterCoverMimeType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharactersListedPayload {
    pub characters: Vec<CharacterCardSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CharacterDeletedPayload {
    pub character_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResourcesPayload {
    pub resource_id: String,
    pub story_concept: String,
    pub character_ids: Vec<String>,
    pub player_schema_id_seed: Option<String>,
    pub world_schema_id_seed: Option<String>,
    pub lorebook_ids: Vec<String>,
    pub planned_story: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryResourcesListedPayload {
    pub resources: Vec<StoryResourcesPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryResourcesDeletedPayload {
    pub resource_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryPlannedPayload {
    pub resource_id: String,
    pub story_script: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryGeneratedPayload {
    pub resource_id: String,
    pub story_id: String,
    pub display_name: String,
    pub graph: StoryGraph,
    pub world_schema_id: String,
    pub player_schema_id: String,
    pub introduction: String,
    #[serde(default)]
    pub common_variables: Vec<CommonVariableDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorySummaryPayload {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub world_schema_id: String,
    pub player_schema_id: String,
    pub introduction: String,
    #[serde(default)]
    pub common_variables: Vec<CommonVariableDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryDetailPayload {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub graph: StoryGraph,
    pub world_schema_id: String,
    pub player_schema_id: String,
    pub introduction: String,
    #[serde(default)]
    pub common_variables: Vec<CommonVariableDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoriesListedPayload {
    pub stories: Vec<StorySummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryDeletedPayload {
    pub story_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StoryDraftStatusPayload {
    Building,
    ReadyToFinalize,
    Finalized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryDraftSummaryPayload {
    pub draft_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub api_group_id: String,
    pub preset_id: String,
    pub status: StoryDraftStatusPayload,
    pub next_section_index: usize,
    pub total_sections: usize,
    pub partial_node_count: usize,
    pub final_story_id: Option<String>,
    pub created_at_ms: Option<u64>,
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryDraftDetailPayload {
    pub draft_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub api_group_id: String,
    pub preset_id: String,
    pub planned_story: String,
    pub outline_sections: Vec<String>,
    pub next_section_index: usize,
    pub partial_graph: StoryGraph,
    pub world_schema_id: String,
    pub player_schema_id: String,
    pub introduction: String,
    #[serde(default)]
    pub common_variables: Vec<CommonVariableDefinition>,
    pub section_summaries: Vec<String>,
    pub status: StoryDraftStatusPayload,
    pub final_story_id: Option<String>,
    pub created_at_ms: Option<u64>,
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryDraftsListedPayload {
    pub drafts: Vec<StoryDraftSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoryDraftDeletedPayload {
    pub draft_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStartedPayload {
    pub story_id: String,
    pub display_name: String,
    pub snapshot: RuntimeSnapshot,
    pub player_profile_id: Option<String>,
    pub player_schema_id: String,
    pub api_group_id: String,
    pub preset_id: String,
    pub history: Vec<SessionMessagePayload>,
    pub created_at_ms: Option<u64>,
    pub updated_at_ms: Option<u64>,
    pub character_summaries: Vec<CharacterCardSummaryPayload>,
    pub config: SessionConfigPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryPayload {
    pub session_id: String,
    pub story_id: String,
    pub display_name: String,
    pub player_profile_id: Option<String>,
    pub player_schema_id: String,
    pub api_group_id: String,
    pub preset_id: String,
    pub turn_index: u64,
    pub created_at_ms: Option<u64>,
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailPayload {
    pub session_id: String,
    pub story_id: String,
    pub display_name: String,
    pub player_profile_id: Option<String>,
    pub player_schema_id: String,
    pub api_group_id: String,
    pub preset_id: String,
    pub snapshot: RuntimeSnapshot,
    pub history: Vec<SessionMessagePayload>,
    pub created_at_ms: Option<u64>,
    pub updated_at_ms: Option<u64>,
    pub config: SessionConfigPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsListedPayload {
    pub sessions: Vec<SessionSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionDeletedPayload {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardHealthStatus {
    Ok,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardHealthPayload {
    pub status: DashboardHealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DashboardCountsPayload {
    pub characters_total: usize,
    pub characters_with_cover: usize,
    pub story_resources_total: usize,
    pub stories_total: usize,
    pub sessions_total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStorySummaryPayload {
    pub story_id: String,
    pub display_name: String,
    pub resource_id: String,
    pub introduction: String,
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSessionSummaryPayload {
    pub session_id: String,
    pub story_id: String,
    pub display_name: String,
    pub turn_index: u64,
    pub updated_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPayload {
    pub health: DashboardHealthPayload,
    pub counts: DashboardCountsPayload,
    pub global_config: GlobalConfigPayload,
    pub recent_stories: Vec<DashboardStorySummaryPayload>,
    pub recent_sessions: Vec<DashboardSessionSummaryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TurnStreamAcceptedPayload {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnCompletedPayload {
    pub result: EngineTurnResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerDescriptionUpdatedPayload {
    pub snapshot: RuntimeSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSnapshotPayload {
    pub snapshot: RuntimeSnapshot,
}
