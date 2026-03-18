use agents::actor::CharacterCard;
use state::{PlayerStateSchema, WorldStateSchema};
use store::{
    ApiGroupRecord, ApiRecord, CharacterCardRecord, LorebookEntryRecord, PresetRecord,
    SchemaRecord, SessionBindingConfig, SessionCharacterRecord, StoryRecord, StoryResourcesRecord,
};

use crate::{RuntimeError, RuntimeState, StoryResources};

use super::util::non_empty_planned_story;
use super::{EngineManager, ManagerError, ResolvedApiGroup};

impl EngineManager {
    pub async fn get_global_config(&self) -> Result<Option<SessionBindingConfig>, ManagerError> {
        self.resolve_first_available_binding().await
    }

    pub async fn list_models(
        &self,
        provider: store::LlmProvider,
        base_url: &str,
        api_key: &str,
    ) -> Result<Vec<String>, ManagerError> {
        self.registry
            .list_models(provider, base_url, api_key)
            .await
            .map_err(ManagerError::Registry)
    }

    pub(super) async fn build_engine_story_resources(
        &self,
        resource: &StoryResourcesRecord,
    ) -> Result<StoryResources, ManagerError> {
        if resource.character_ids.is_empty() {
            return Err(ManagerError::EmptyCharacterIds);
        }

        let mut cards = Vec::with_capacity(resource.character_ids.len());
        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| ManagerError::MissingCharacter(character_id.clone()))?;
            cards.push(self.resolve_character_card(&character).await?);
        }

        let player_state_schema_seed = match &resource.player_schema_id_seed {
            Some(schema_id) => Some(self.resolve_player_schema(schema_id).await?),
            None => None,
        };
        let lorebook_entries = self
            .resolve_story_resource_lorebook_entries(resource)
            .await?;

        let mut story_resources = StoryResources::new(
            resource.resource_id.clone(),
            resource.story_concept.clone(),
            cards,
            player_state_schema_seed,
        )?;
        story_resources = story_resources.with_lorebook_entries(lorebook_entries);

        if let Some(planned_story) = non_empty_planned_story(resource.planned_story.as_deref()) {
            story_resources = story_resources.with_planned_story(planned_story);
        }
        if let Some(world_schema_id_seed) = &resource.world_schema_id_seed {
            story_resources = story_resources.with_world_state_schema_seed(
                self.resolve_world_schema(world_schema_id_seed).await?,
            );
        }

        Ok(story_resources)
    }

    pub(super) async fn build_runtime_state_from_story(
        &self,
        story: &StoryRecord,
        player_name: Option<String>,
        player_description: String,
    ) -> Result<RuntimeState, ManagerError> {
        let resource = self
            .store
            .get_story_resources(&story.resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(story.resource_id.clone()))?;
        let lorebook_entries = self
            .resolve_story_resource_lorebook_entries(&resource)
            .await?;
        let characters = self.load_story_characters(&story.resource_id).await?;
        let mut resolved_characters = Vec::with_capacity(characters.len());
        for character in &characters {
            resolved_characters.push(self.resolve_character_card(character).await?);
        }

        let mut runtime_state = RuntimeState::from_story_graph(
            &story.story_id,
            story.graph.clone(),
            resolved_characters,
            player_description,
            self.resolve_player_schema(&story.player_schema_id).await?,
        )
        .map_err(ManagerError::from)?
        .with_lorebook_entries(lorebook_entries);
        runtime_state.set_player_name(player_name);
        Ok(runtime_state)
    }

    pub(super) async fn build_runtime_state_from_session(
        &self,
        story: &StoryRecord,
        session: &store::SessionRecord,
    ) -> Result<RuntimeState, ManagerError> {
        let (player_name, _player_description) = self
            .resolve_player_identity(session.player_profile_id.as_deref())
            .await?;
        let resource = self
            .store
            .get_story_resources(&story.resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(story.resource_id.clone()))?;
        let lorebook_entries = self
            .resolve_story_resource_lorebook_entries(&resource)
            .await?;
        let characters = self.load_story_characters(&story.resource_id).await?;
        let session_characters = self
            .store
            .list_session_characters(&session.session_id)
            .await?;
        let mut resolved_characters =
            Vec::with_capacity(characters.len().saturating_add(session_characters.len()));
        for character in &characters {
            resolved_characters.push(self.resolve_character_card(character).await?);
        }
        for character in &session_characters {
            resolved_characters.push(self.resolve_session_character_card(character));
        }

        let mut runtime_state = RuntimeState::from_snapshot(
            &story.story_id,
            story::runtime_graph::RuntimeStoryGraph::from_story_graph(story.graph.clone())
                .map_err(RuntimeError::GraphBuild)?,
            resolved_characters,
            self.resolve_player_schema(&session.player_schema_id)
                .await?,
            session.snapshot.clone(),
        )
        .map_err(ManagerError::from)?
        .with_lorebook_entries(lorebook_entries);
        for character in &session_characters {
            runtime_state.register_existing_session_character(&character.session_character_id)?;
        }
        runtime_state.set_player_name(player_name);
        Ok(runtime_state)
    }

    pub(super) async fn resolve_player_identity(
        &self,
        player_profile_id: Option<&str>,
    ) -> Result<(Option<String>, String), ManagerError> {
        match player_profile_id {
            Some(player_profile_id) => {
                let profile = self
                    .store
                    .get_player_profile(player_profile_id)
                    .await?
                    .ok_or_else(|| {
                        ManagerError::MissingPlayerProfile(player_profile_id.to_owned())
                    })?;
                Ok((Some(profile.display_name), profile.description))
            }
            None => Ok((None, String::new())),
        }
    }

    async fn load_story_characters(
        &self,
        resource_id: &str,
    ) -> Result<Vec<CharacterCardRecord>, ManagerError> {
        let resource = self
            .store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| ManagerError::MissingStoryResources(resource_id.to_owned()))?;
        let mut characters = Vec::with_capacity(resource.character_ids.len());
        for character_id in &resource.character_ids {
            let character = self
                .store
                .get_character(character_id)
                .await?
                .ok_or_else(|| ManagerError::MissingCharacter(character_id.clone()))?;
            characters.push(character);
        }
        Ok(characters)
    }

    async fn resolve_story_resource_lorebook_entries(
        &self,
        resource: &StoryResourcesRecord,
    ) -> Result<Vec<LorebookEntryRecord>, ManagerError> {
        let mut entries = Vec::new();
        for lorebook_id in &resource.lorebook_ids {
            let lorebook = self
                .store
                .get_lorebook(lorebook_id)
                .await?
                .ok_or_else(|| ManagerError::MissingLorebook(lorebook_id.clone()))?;
            entries.extend(lorebook.entries);
        }
        Ok(entries)
    }

    async fn resolve_schema_record(&self, schema_id: &str) -> Result<SchemaRecord, ManagerError> {
        self.store
            .get_schema(schema_id)
            .await?
            .ok_or_else(|| ManagerError::MissingSchema(schema_id.to_owned()))
    }

    pub(super) async fn resolve_world_schema(
        &self,
        schema_id: &str,
    ) -> Result<WorldStateSchema, ManagerError> {
        let schema = self.resolve_schema_record(schema_id).await?;
        Ok(WorldStateSchema {
            fields: schema.fields,
        })
    }

    pub(super) async fn resolve_player_schema(
        &self,
        schema_id: &str,
    ) -> Result<PlayerStateSchema, ManagerError> {
        let schema = self.resolve_schema_record(schema_id).await?;
        Ok(PlayerStateSchema {
            fields: schema.fields,
        })
    }

    async fn resolve_character_card(
        &self,
        record: &CharacterCardRecord,
    ) -> Result<CharacterCard, ManagerError> {
        let schema = self
            .resolve_schema_record(&record.content.schema_id)
            .await?;
        Ok(CharacterCard {
            id: record.content.id.clone(),
            name: record.content.name.clone(),
            personality: record.content.personality.clone(),
            style: record.content.style.clone(),
            state_schema: schema.fields,
            system_prompt: record.content.system_prompt.clone(),
        })
    }

    pub(super) fn resolve_session_character_card(
        &self,
        record: &SessionCharacterRecord,
    ) -> CharacterCard {
        CharacterCard {
            id: record.session_character_id.clone(),
            name: record.display_name.clone(),
            personality: record.personality.clone(),
            style: record.style.clone(),
            state_schema: Default::default(),
            system_prompt: record.system_prompt.clone(),
        }
    }

    async fn resolve_first_available_binding(
        &self,
    ) -> Result<Option<SessionBindingConfig>, ManagerError> {
        let mut api_groups = self.store.list_api_groups().await?;
        let mut presets = self.store.list_presets().await?;
        api_groups.sort_by(|left, right| left.api_group_id.cmp(&right.api_group_id));
        presets.sort_by(|left, right| left.preset_id.cmp(&right.preset_id));

        match (api_groups.first(), presets.first()) {
            (Some(api_group), Some(preset)) => Ok(Some(SessionBindingConfig {
                api_group_id: api_group.api_group_id.clone(),
                preset_id: preset.preset_id.clone(),
            })),
            _ => Ok(None),
        }
    }

    pub(super) async fn resolve_api_group(
        &self,
        api_group_id: &str,
    ) -> Result<ApiGroupRecord, ManagerError> {
        self.store
            .get_api_group(api_group_id)
            .await?
            .ok_or_else(|| ManagerError::MissingApiGroup(api_group_id.to_owned()))
    }

    async fn resolve_api(&self, api_id: &str) -> Result<ApiRecord, ManagerError> {
        self.store
            .get_api(api_id)
            .await?
            .ok_or_else(|| ManagerError::MissingApi(api_id.to_owned()))
    }

    pub(super) async fn resolve_api_group_bindings(
        &self,
        api_group: &ApiGroupRecord,
    ) -> Result<ResolvedApiGroup, ManagerError> {
        Ok(ResolvedApiGroup {
            planner: self.resolve_api(&api_group.agents.planner_api_id).await?,
            architect: self.resolve_api(&api_group.agents.architect_api_id).await?,
            director: self.resolve_api(&api_group.agents.director_api_id).await?,
            actor: self.resolve_api(&api_group.agents.actor_api_id).await?,
            narrator: self.resolve_api(&api_group.agents.narrator_api_id).await?,
            keeper: self.resolve_api(&api_group.agents.keeper_api_id).await?,
            replyer: self.resolve_api(&api_group.agents.replyer_api_id).await?,
        })
    }

    pub(super) async fn resolve_preset(
        &self,
        preset_id: &str,
    ) -> Result<PresetRecord, ManagerError> {
        self.store
            .get_preset(preset_id)
            .await?
            .ok_or_else(|| ManagerError::MissingPreset(preset_id.to_owned()))
    }

    pub(super) async fn resolve_api_group_and_preset(
        &self,
        api_group_id: Option<&str>,
        preset_id: Option<&str>,
    ) -> Result<(ApiGroupRecord, PresetRecord, SessionBindingConfig), ManagerError> {
        let binding = match (api_group_id, preset_id) {
            (Some(api_group_id), Some(preset_id)) => SessionBindingConfig {
                api_group_id: api_group_id.to_owned(),
                preset_id: preset_id.to_owned(),
            },
            (Some(api_group_id), None) => {
                let mut presets = self.store.list_presets().await?;
                presets.sort_by(|left, right| left.preset_id.cmp(&right.preset_id));
                let preset = presets
                    .first()
                    .ok_or(ManagerError::LlmConfigNotInitialized)?;
                SessionBindingConfig {
                    api_group_id: api_group_id.to_owned(),
                    preset_id: preset.preset_id.clone(),
                }
            }
            (None, Some(preset_id)) => {
                let mut api_groups = self.store.list_api_groups().await?;
                api_groups.sort_by(|left, right| left.api_group_id.cmp(&right.api_group_id));
                let api_group = api_groups
                    .first()
                    .ok_or(ManagerError::LlmConfigNotInitialized)?;
                SessionBindingConfig {
                    api_group_id: api_group.api_group_id.clone(),
                    preset_id: preset_id.to_owned(),
                }
            }
            (None, None) => self
                .resolve_first_available_binding()
                .await?
                .ok_or(ManagerError::LlmConfigNotInitialized)?,
        };

        let api_group = self.resolve_api_group(&binding.api_group_id).await?;
        let preset = self.resolve_preset(&binding.preset_id).await?;
        Ok((api_group, preset, binding))
    }
}
