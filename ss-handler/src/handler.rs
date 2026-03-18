mod api;
mod api_group;
mod config;
mod dashboard;
mod data_package;
mod lorebook;
mod player_profile;
mod preset;
mod schema;
mod session;
mod story;
mod upload;

use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use engine::{EngineManager, LlmApiRegistry};
use futures_core::Stream;
use protocol::{JsonRpcRequestMessage, JsonRpcResponseMessage, RequestParams, ServerEventMessage};
use store::{InMemoryStore, Store};

use crate::error::HandlerError;

pub use upload::BinaryAsset;

pub type HandlerEventStream = Pin<Box<dyn Stream<Item = ServerEventMessage> + Send>>;

pub enum HandlerReply {
    Unary(JsonRpcResponseMessage),
    Stream {
        ack: JsonRpcResponseMessage,
        events: HandlerEventStream,
    },
}

pub struct Handler {
    store: Arc<dyn Store>,
    manager: EngineManager,
    id_generator: IdGenerator,
    data_packages: data_package::TempDataPackages,
}

impl Handler {
    pub async fn new(
        store: Arc<dyn Store>,
        registry: LlmApiRegistry,
    ) -> Result<Self, HandlerError> {
        let manager = EngineManager::new(Arc::clone(&store), registry).await?;

        Ok(Self {
            store,
            manager,
            id_generator: IdGenerator::default(),
            data_packages: data_package::TempDataPackages::default(),
        })
    }

    pub async fn with_in_memory_store(registry: LlmApiRegistry) -> Result<Self, HandlerError> {
        Self::new(Arc::new(InMemoryStore::new()), registry).await
    }

    pub async fn handle(&self, request: JsonRpcRequestMessage) -> HandlerReply {
        let request_id = request.id.clone();
        let session_id = request.session_id.clone();

        let result = match request.params {
            RequestParams::ApiCreate(params) => self.handle_api_create(&request_id, params).await,
            RequestParams::ApiGet(params) => self.handle_api_get(&request_id, params).await,
            RequestParams::ApiList(_) => self.handle_api_list(&request_id).await,
            RequestParams::ApiListModels(params) => {
                self.handle_api_list_models(&request_id, params).await
            }
            RequestParams::ApiUpdate(params) => self.handle_api_update(&request_id, params).await,
            RequestParams::ApiDelete(params) => self.handle_api_delete(&request_id, params).await,
            RequestParams::ApiGroupCreate(params) => {
                self.handle_api_group_create(&request_id, params).await
            }
            RequestParams::ApiGroupGet(params) => {
                self.handle_api_group_get(&request_id, params).await
            }
            RequestParams::ApiGroupList(_) => self.handle_api_group_list(&request_id).await,
            RequestParams::ApiGroupUpdate(params) => {
                self.handle_api_group_update(&request_id, params).await
            }
            RequestParams::ApiGroupDelete(params) => {
                self.handle_api_group_delete(&request_id, params).await
            }
            RequestParams::PresetCreate(params) => {
                self.handle_preset_create(&request_id, params).await
            }
            RequestParams::PresetGet(params) => self.handle_preset_get(&request_id, params).await,
            RequestParams::PresetList(_) => self.handle_preset_list(&request_id).await,
            RequestParams::PresetUpdate(params) => {
                self.handle_preset_update(&request_id, params).await
            }
            RequestParams::PresetDelete(params) => {
                self.handle_preset_delete(&request_id, params).await
            }
            RequestParams::PresetEntryCreate(params) => {
                self.handle_preset_entry_create(&request_id, params).await
            }
            RequestParams::PresetEntryUpdate(params) => {
                self.handle_preset_entry_update(&request_id, params).await
            }
            RequestParams::PresetEntryDelete(params) => {
                self.handle_preset_entry_delete(&request_id, params).await
            }
            RequestParams::SchemaCreate(params) => {
                self.handle_schema_create(&request_id, params).await
            }
            RequestParams::SchemaGet(params) => self.handle_schema_get(&request_id, params).await,
            RequestParams::SchemaList(_) => self.handle_schema_list(&request_id).await,
            RequestParams::SchemaUpdate(params) => {
                self.handle_schema_update(&request_id, params).await
            }
            RequestParams::SchemaDelete(params) => {
                self.handle_schema_delete(&request_id, params).await
            }
            RequestParams::LorebookCreate(params) => {
                self.handle_lorebook_create(&request_id, params).await
            }
            RequestParams::LorebookGet(params) => {
                self.handle_lorebook_get(&request_id, params).await
            }
            RequestParams::LorebookList(_) => self.handle_lorebook_list(&request_id).await,
            RequestParams::LorebookUpdate(params) => {
                self.handle_lorebook_update(&request_id, params).await
            }
            RequestParams::LorebookDelete(params) => {
                self.handle_lorebook_delete(&request_id, params).await
            }
            RequestParams::LorebookEntryCreate(params) => {
                self.handle_lorebook_entry_create(&request_id, params).await
            }
            RequestParams::LorebookEntryGet(params) => {
                self.handle_lorebook_entry_get(&request_id, params).await
            }
            RequestParams::LorebookEntryList(params) => {
                self.handle_lorebook_entry_list(&request_id, params).await
            }
            RequestParams::LorebookEntryUpdate(params) => {
                self.handle_lorebook_entry_update(&request_id, params).await
            }
            RequestParams::LorebookEntryDelete(params) => {
                self.handle_lorebook_entry_delete(&request_id, params).await
            }
            RequestParams::PlayerProfileCreate(params) => {
                self.handle_player_profile_create(&request_id, params).await
            }
            RequestParams::PlayerProfileGet(params) => {
                self.handle_player_profile_get(&request_id, params).await
            }
            RequestParams::PlayerProfileList(_) => {
                self.handle_player_profile_list(&request_id).await
            }
            RequestParams::PlayerProfileUpdate(params) => {
                self.handle_player_profile_update(&request_id, params).await
            }
            RequestParams::PlayerProfileDelete(params) => {
                self.handle_player_profile_delete(&request_id, params).await
            }
            RequestParams::CharacterCreate(params) => {
                self.handle_character_create(&request_id, params).await
            }
            RequestParams::CharacterGet(params) => {
                self.handle_character_get(&request_id, params).await
            }
            RequestParams::CharacterUpdate(params) => {
                self.handle_character_update(&request_id, params).await
            }
            RequestParams::CharacterList(_) => self.handle_character_list(&request_id).await,
            RequestParams::CharacterDelete(params) => {
                self.handle_character_delete(&request_id, params).await
            }
            RequestParams::StoryResourcesCreate(params) => {
                self.handle_story_resources_create(&request_id, params)
                    .await
            }
            RequestParams::StoryResourcesGet(params) => {
                self.handle_story_resources_get(&request_id, params).await
            }
            RequestParams::StoryResourcesList(_) => {
                self.handle_story_resources_list(&request_id).await
            }
            RequestParams::StoryResourcesUpdate(params) => {
                self.handle_story_resources_update(&request_id, params)
                    .await
            }
            RequestParams::StoryResourcesDelete(params) => {
                self.handle_story_resources_delete(&request_id, params)
                    .await
            }
            RequestParams::StoryGeneratePlan(params) => {
                self.handle_story_generate_plan(&request_id, params).await
            }
            RequestParams::StoryGenerate(params) => {
                self.handle_story_generate(&request_id, params).await
            }
            RequestParams::StoryGet(params) => self.handle_story_get(&request_id, params).await,
            RequestParams::StoryUpdate(params) => {
                self.handle_story_update(&request_id, params).await
            }
            RequestParams::StoryUpdateGraph(params) => {
                self.handle_story_update_graph(&request_id, params).await
            }
            RequestParams::StoryList(_) => self.handle_story_list(&request_id).await,
            RequestParams::StoryDelete(params) => {
                self.handle_story_delete(&request_id, params).await
            }
            RequestParams::StoryDraftStart(params) => {
                self.handle_story_draft_start(&request_id, params).await
            }
            RequestParams::StoryDraftGet(params) => {
                self.handle_story_draft_get(&request_id, params).await
            }
            RequestParams::StoryDraftList(_) => self.handle_story_draft_list(&request_id).await,
            RequestParams::StoryDraftUpdateGraph(params) => {
                self.handle_story_draft_update_graph(&request_id, params)
                    .await
            }
            RequestParams::StoryDraftContinue(params) => {
                self.handle_story_draft_continue(&request_id, params).await
            }
            RequestParams::StoryDraftFinalize(params) => {
                self.handle_story_draft_finalize(&request_id, params).await
            }
            RequestParams::StoryDraftDelete(params) => {
                self.handle_story_draft_delete(&request_id, params).await
            }
            RequestParams::StoryStartSession(params) => {
                self.handle_story_start_session(&request_id, params).await
            }
            RequestParams::SessionGet(_) => {
                self.handle_session_get(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionUpdate(params) => {
                self.handle_session_update(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionList(_) => self.handle_session_list(&request_id).await,
            RequestParams::SessionDelete(_) => {
                self.handle_session_delete(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionMessageCreate(params) => {
                self.handle_session_message_create(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionMessageGet(params) => {
                self.handle_session_message_get(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionMessageList(params) => {
                self.handle_session_message_list(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionMessageUpdate(params) => {
                self.handle_session_message_update(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionMessageDelete(params) => {
                self.handle_session_message_delete(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionCharacterGet(params) => {
                self.handle_session_character_get(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionCharacterList(params) => {
                self.handle_session_character_list(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionCharacterUpdate(params) => {
                self.handle_session_character_update(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionCharacterDelete(params) => {
                self.handle_session_character_delete(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionCharacterEnterScene(params) => {
                self.handle_session_character_enter_scene(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionCharacterLeaveScene(params) => {
                self.handle_session_character_leave_scene(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionRunTurn(params) => {
                return self
                    .handle_session_run_turn(request_id, session_id, params)
                    .await;
            }
            RequestParams::SessionGetVariables(_) => {
                self.handle_session_get_variables(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionUpdateVariables(params) => {
                self.handle_session_update_variables(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionSuggestReplies(params) => {
                self.handle_session_suggest_replies(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionSetPlayerProfile(params) => {
                self.handle_session_set_player_profile(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::SessionUpdatePlayerDescription(params) => {
                self.handle_session_update_player_description(
                    &request_id,
                    session_id.clone(),
                    params,
                )
                .await
            }
            RequestParams::SessionGetRuntimeSnapshot(_) => {
                self.handle_session_get_runtime_snapshot(&request_id, session_id.clone())
                    .await
            }
            RequestParams::ConfigGetGlobal(_) => self.handle_config_get_global(&request_id).await,
            RequestParams::SessionGetConfig(_) => {
                self.handle_session_get_config(&request_id, session_id.clone())
                    .await
            }
            RequestParams::SessionUpdateConfig(params) => {
                self.handle_session_update_config(&request_id, session_id.clone(), params)
                    .await
            }
            RequestParams::DashboardGet(_) => self.handle_dashboard_get(&request_id).await,
            RequestParams::DataPackageExportPrepare(params) => {
                self.handle_data_package_export_prepare(&request_id, params)
                    .await
            }
            RequestParams::DataPackageImportPrepare(params) => {
                self.handle_data_package_import_prepare(&request_id, params)
                    .await
            }
            RequestParams::DataPackageImportCommit(params) => {
                self.handle_data_package_import_commit(&request_id, params)
                    .await
            }
        };

        match result {
            Ok(response) => HandlerReply::Unary(response),
            Err(error) => HandlerReply::Unary(JsonRpcResponseMessage::err(
                request_id,
                session_id,
                error.to_error_payload(),
            )),
        }
    }
}

#[derive(Default)]
struct IdGenerator {
    next: AtomicU64,
}

impl IdGenerator {
    fn next(&self, prefix: &str) -> String {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{id}")
    }
}

fn require_session_id(session_id: Option<String>) -> Result<String, HandlerError> {
    session_id.ok_or(HandlerError::MissingSessionId)
}
