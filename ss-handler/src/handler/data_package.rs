use std::collections::{BTreeSet, HashMap};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use protocol::{
    DATA_PACKAGE_ARCHIVE_CONTENT_TYPE, DataPackageArchive, DataPackageCharacterEntry,
    DataPackageExportPrepareParams, DataPackageExportPreparedPayload,
    DataPackageImportCommitParams, DataPackageImportCommittedPayload,
    DataPackageImportPrepareParams, DataPackageImportPreparedPayload, JsonRpcResponseMessage,
    ResourceFilePayload, ResourceFileRefPayload, ResponseResult,
};
use store::{
    BlobRecord, CharacterCardDefinition, CharacterCardRecord, PresetRecord, SchemaRecord,
    StoryRecord, StoryResourcesRecord,
};
use story::runtime_graph::{GraphBuildError, RuntimeStoryGraph};
use story::{validate_common_variables, validate_graph_state_conventions};

use crate::error::HandlerError;

use super::{BinaryAsset, Handler};

pub(super) const PACKAGE_EXPORT_RESOURCE_PREFIX: &str = "package_export:";
pub(super) const PACKAGE_IMPORT_RESOURCE_PREFIX: &str = "package_import:";
pub(super) const PACKAGE_ARCHIVE_FILE_ID: &str = "archive";

const TEMP_DATA_PACKAGE_TTL_MS: u64 = 30 * 60 * 1_000;

#[derive(Default)]
pub(super) struct TempDataPackages {
    exports: Mutex<HashMap<String, PreparedExportSlot>>,
    imports: Mutex<HashMap<String, PreparedImportSlot>>,
}

#[derive(Debug, Clone)]
struct PreparedExportSlot {
    file_name: String,
    bytes: Vec<u8>,
    expires_at_ms: u64,
}

#[derive(Debug, Clone)]
struct PreparedImportSlot {
    file_name: Option<String>,
    bytes: Option<Vec<u8>>,
    expires_at_ms: u64,
}

#[derive(Debug, Default)]
struct DataPackageSelection {
    preset_ids: BTreeSet<String>,
    schema_ids: BTreeSet<String>,
    lorebook_ids: BTreeSet<String>,
    player_profile_ids: BTreeSet<String>,
    character_ids: BTreeSet<String>,
    story_resource_ids: BTreeSet<String>,
    story_ids: BTreeSet<String>,
}

#[derive(Debug, Default)]
struct AppliedImportResources {
    story_ids: Vec<String>,
    story_resource_ids: Vec<String>,
    character_ids: Vec<String>,
    blob_ids: Vec<String>,
    player_profile_ids: Vec<String>,
    lorebook_ids: Vec<String>,
    preset_ids: Vec<String>,
    schema_ids: Vec<String>,
}

impl Handler {
    pub(crate) async fn handle_data_package_export_prepare(
        &self,
        request_id: &str,
        params: DataPackageExportPrepareParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let include_dependencies = params.include_dependencies;
        let mut selection = DataPackageSelection::from_params(params);
        if selection.is_empty() {
            return Err(HandlerError::EmptyDataPackageSelection);
        }
        if include_dependencies {
            self.expand_export_dependencies(&mut selection).await?;
        }

        let archive = self.build_export_archive(&selection).await?;
        let contents = archive.contents();
        let bytes = archive.to_zip_bytes()?;
        let export_id = self.id_generator.next("package-export");
        let file_name = format!("sillystage-data-package-{export_id}.zip");

        self.data_packages
            .insert_export(export_id.clone(), file_name.clone(), bytes);

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::DataPackageExportPrepared(DataPackageExportPreparedPayload {
                export_id: export_id.clone(),
                archive: ResourceFileRefPayload {
                    resource_id: package_export_resource_id(&export_id),
                    file_id: PACKAGE_ARCHIVE_FILE_ID.to_owned(),
                },
                contents,
            }),
        ))
    }

    pub(crate) async fn handle_data_package_import_prepare(
        &self,
        request_id: &str,
        _params: DataPackageImportPrepareParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let import_id = self.id_generator.next("package-import");
        self.data_packages.insert_import(import_id.clone());

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::DataPackageImportPrepared(DataPackageImportPreparedPayload {
                import_id: import_id.clone(),
                archive: ResourceFileRefPayload {
                    resource_id: package_import_resource_id(&import_id),
                    file_id: PACKAGE_ARCHIVE_FILE_ID.to_owned(),
                },
            }),
        ))
    }

    pub(crate) async fn handle_data_package_import_commit(
        &self,
        request_id: &str,
        params: DataPackageImportCommitParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let bytes = self
            .data_packages
            .import_bytes(&params.import_id)
            .ok_or_else(|| HandlerError::MissingDataPackageImport(params.import_id.clone()))?
            .ok_or_else(|| {
                HandlerError::InvalidDataPackage(format!(
                    "data package import '{}' does not have an uploaded archive",
                    params.import_id
                ))
            })?;
        let archive = DataPackageArchive::from_zip_bytes(&bytes)?;
        self.validate_import_archive(&archive).await?;
        self.apply_import_archive(&archive).await?;
        self.data_packages.remove_import(&params.import_id);

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::DataPackageImportCommitted(DataPackageImportCommittedPayload {
                import_id: params.import_id,
                contents: archive.contents(),
            }),
        ))
    }

    pub(super) async fn upload_package_import_archive(
        &self,
        import_id: &str,
        resource_id: &str,
        file_name: Option<String>,
        bytes: Vec<u8>,
    ) -> Result<ResourceFilePayload, HandlerError> {
        if bytes.is_empty() {
            return Err(HandlerError::InvalidDataPackage(
                "data package archive bytes must not be empty".to_owned(),
            ));
        }

        let file_name = normalize_file_name(file_name)
            .or_else(|| Some(format!("sillystage-data-package-{import_id}.zip")));
        self.data_packages
            .store_import_archive(import_id, file_name.clone(), bytes.len() as u64, bytes)
            .ok_or_else(|| HandlerError::MissingDataPackageImport(import_id.to_owned()))?;

        Ok(ResourceFilePayload {
            resource_id: resource_id.to_owned(),
            file_id: PACKAGE_ARCHIVE_FILE_ID.to_owned(),
            file_name,
            content_type: DATA_PACKAGE_ARCHIVE_CONTENT_TYPE.to_owned(),
            size_bytes: self
                .data_packages
                .import_size(import_id)
                .ok_or_else(|| HandlerError::MissingDataPackageImport(import_id.to_owned()))?,
        })
    }

    pub(super) async fn download_package_export_archive(
        &self,
        export_id: &str,
    ) -> Result<BinaryAsset, HandlerError> {
        let slot = self
            .data_packages
            .export_slot(export_id)
            .ok_or_else(|| HandlerError::MissingDataPackageExport(export_id.to_owned()))?;

        Ok(BinaryAsset {
            file_name: Some(slot.file_name),
            content_type: DATA_PACKAGE_ARCHIVE_CONTENT_TYPE.to_owned(),
            bytes: slot.bytes,
        })
    }

    async fn expand_export_dependencies(
        &self,
        selection: &mut DataPackageSelection,
    ) -> Result<(), HandlerError> {
        for story_id in selection.story_ids.clone() {
            let story = self.load_story_record(&story_id).await?;
            selection.story_resource_ids.insert(story.resource_id);
            selection.schema_ids.insert(story.world_schema_id);
            selection.schema_ids.insert(story.player_schema_id);
        }

        for resource_id in selection.story_resource_ids.clone() {
            let resource = self.load_story_resources_record(&resource_id).await?;
            selection.character_ids.extend(resource.character_ids);
            selection.lorebook_ids.extend(resource.lorebook_ids);
            if let Some(schema_id) = resource.player_schema_id_seed {
                selection.schema_ids.insert(schema_id);
            }
            if let Some(schema_id) = resource.world_schema_id_seed {
                selection.schema_ids.insert(schema_id);
            }
        }

        for character_id in selection.character_ids.clone() {
            let character = self.load_character_record(&character_id).await?;
            selection.schema_ids.insert(character.content.schema_id);
        }

        Ok(())
    }

    async fn build_export_archive(
        &self,
        selection: &DataPackageSelection,
    ) -> Result<DataPackageArchive, HandlerError> {
        let mut presets = Vec::with_capacity(selection.preset_ids.len());
        for preset_id in &selection.preset_ids {
            presets.push(self.load_preset_record(preset_id).await?);
        }

        let mut schemas = Vec::with_capacity(selection.schema_ids.len());
        for schema_id in &selection.schema_ids {
            schemas.push(self.load_schema_record(schema_id).await?);
        }

        let mut lorebooks = Vec::with_capacity(selection.lorebook_ids.len());
        for lorebook_id in &selection.lorebook_ids {
            let record = self
                .store
                .get_lorebook(lorebook_id)
                .await?
                .ok_or_else(|| HandlerError::MissingLorebook(lorebook_id.clone()))?;
            lorebooks.push(record);
        }

        let mut player_profiles = Vec::with_capacity(selection.player_profile_ids.len());
        for player_profile_id in &selection.player_profile_ids {
            let record = self
                .store
                .get_player_profile(player_profile_id)
                .await?
                .ok_or_else(|| HandlerError::MissingPlayerProfile(player_profile_id.clone()))?;
            player_profiles.push(record);
        }

        let mut characters = Vec::with_capacity(selection.character_ids.len());
        for character_id in &selection.character_ids {
            characters.push(self.build_character_entry(character_id).await?);
        }

        let mut story_resources = Vec::with_capacity(selection.story_resource_ids.len());
        for resource_id in &selection.story_resource_ids {
            story_resources.push(self.load_story_resources_record(resource_id).await?);
        }

        let mut stories = Vec::with_capacity(selection.story_ids.len());
        for story_id in &selection.story_ids {
            stories.push(self.load_story_record(story_id).await?);
        }

        Ok(DataPackageArchive::new(
            now_timestamp_ms(),
            presets,
            schemas,
            lorebooks,
            player_profiles,
            characters,
            story_resources,
            stories,
        ))
    }

    async fn validate_import_archive(
        &self,
        archive: &DataPackageArchive,
    ) -> Result<(), HandlerError> {
        let contents = archive.contents();
        if contents.presets.count
            + contents.schemas.count
            + contents.lorebooks.count
            + contents.player_profiles.count
            + contents.characters.count
            + contents.story_resources.count
            + contents.stories.count
            == 0
        {
            return Err(HandlerError::InvalidDataPackage(
                "archive does not contain any resources".to_owned(),
            ));
        }

        self.ensure_import_conflicts(archive).await?;

        for preset in &archive.presets {
            ensure_non_empty_id("preset", &preset.preset_id)?;
        }
        for schema in &archive.schemas {
            ensure_non_empty_id("schema", &schema.schema_id)?;
        }
        for lorebook in &archive.lorebooks {
            ensure_non_empty_id("lorebook", &lorebook.lorebook_id)?;
        }
        for player_profile in &archive.player_profiles {
            ensure_non_empty_id("player profile", &player_profile.player_profile_id)?;
        }
        for character in &archive.characters {
            ensure_non_empty_id("character", &character.character_id)?;
            self.ensure_schema_available_for_import(
                archive,
                &character.content.schema_id,
                &format!("character '{}'", character.character_id),
            )
            .await?;
        }
        for resource in &archive.story_resources {
            ensure_non_empty_id("story resources", &resource.resource_id)?;
            if resource.character_ids.is_empty() {
                return Err(HandlerError::InvalidDataPackage(format!(
                    "story resources '{}' must contain at least one character",
                    resource.resource_id
                )));
            }
            for character_id in &resource.character_ids {
                self.ensure_character_available_for_import(
                    archive,
                    character_id,
                    &format!("story resources '{}'", resource.resource_id),
                )
                .await?;
            }
            for lorebook_id in &resource.lorebook_ids {
                self.ensure_lorebook_available_for_import(
                    archive,
                    lorebook_id,
                    &format!("story resources '{}'", resource.resource_id),
                )
                .await?;
            }
            if let Some(schema_id) = &resource.player_schema_id_seed {
                self.ensure_schema_available_for_import(
                    archive,
                    schema_id,
                    &format!("story resources '{}'", resource.resource_id),
                )
                .await?;
            }
            if let Some(schema_id) = &resource.world_schema_id_seed {
                self.ensure_schema_available_for_import(
                    archive,
                    schema_id,
                    &format!("story resources '{}'", resource.resource_id),
                )
                .await?;
            }
        }
        for story in &archive.stories {
            ensure_non_empty_id("story", &story.story_id)?;
            self.ensure_story_resources_available_for_import(
                archive,
                &story.resource_id,
                &format!("story '{}'", story.story_id),
            )
            .await?;
            self.ensure_schema_available_for_import(
                archive,
                &story.world_schema_id,
                &format!("story '{}'", story.story_id),
            )
            .await?;
            self.ensure_schema_available_for_import(
                archive,
                &story.player_schema_id,
                &format!("story '{}'", story.story_id),
            )
            .await?;
            validate_story_graph(&story.graph)?;
            self.validate_story_common_variables_for_import(archive, story)
                .await?;
        }

        Ok(())
    }

    async fn apply_import_archive(&self, archive: &DataPackageArchive) -> Result<(), HandlerError> {
        let mut applied = AppliedImportResources::default();
        let apply_result = async {
            for schema in &archive.schemas {
                self.store.save_schema(schema.clone()).await?;
                applied.schema_ids.push(schema.schema_id.clone());
            }
            for preset in &archive.presets {
                self.store.save_preset(preset.clone()).await?;
                applied.preset_ids.push(preset.preset_id.clone());
            }
            for lorebook in &archive.lorebooks {
                self.store.save_lorebook(lorebook.clone()).await?;
                applied.lorebook_ids.push(lorebook.lorebook_id.clone());
            }
            for player_profile in &archive.player_profiles {
                self.store
                    .save_player_profile(player_profile.clone())
                    .await?;
                applied
                    .player_profile_ids
                    .push(player_profile.player_profile_id.clone());
            }
            for character in &archive.characters {
                let record = self
                    .save_imported_character(character, &mut applied.blob_ids)
                    .await?;
                self.store.save_character(record.clone()).await?;
                applied.character_ids.push(record.character_id);
            }
            for resource in &archive.story_resources {
                self.store.save_story_resources(resource.clone()).await?;
                applied
                    .story_resource_ids
                    .push(resource.resource_id.clone());
            }
            for story in &archive.stories {
                self.store.save_story(story.clone()).await?;
                applied.story_ids.push(story.story_id.clone());
            }
            Ok::<(), HandlerError>(())
        }
        .await;

        if let Err(error) = apply_result {
            self.rollback_import(applied).await;
            return Err(error);
        }

        Ok(())
    }

    async fn save_imported_character(
        &self,
        character: &DataPackageCharacterEntry,
        blob_ids: &mut Vec<String>,
    ) -> Result<CharacterCardRecord, HandlerError> {
        let (cover_blob_id, cover_file_name, cover_mime_type) = match (
            character.cover_bytes.clone(),
            character.cover_file_name.clone(),
            character.cover_content_type.clone(),
        ) {
            (Some(bytes), Some(file_name), Some(content_type)) => {
                let blob = BlobRecord {
                    blob_id: self.id_generator.next("blob"),
                    file_name: Some(file_name.clone()),
                    content_type: normalize_content_type(content_type.clone()),
                    bytes,
                };
                self.store.save_blob(blob.clone()).await?;
                blob_ids.push(blob.blob_id.clone());
                (
                    Some(blob.blob_id),
                    Some(file_name),
                    Some(normalize_content_type(content_type)),
                )
            }
            (None, None, None) => (None, None, None),
            _ => {
                return Err(HandlerError::InvalidDataPackage(format!(
                    "character '{}' cover metadata is incomplete",
                    character.character_id
                )));
            }
        };

        Ok(CharacterCardRecord {
            character_id: character.character_id.clone(),
            content: CharacterCardDefinition {
                id: character.content.id.clone(),
                name: character.content.name.clone(),
                personality: character.content.personality.clone(),
                style: character.content.style.clone(),
                schema_id: character.content.schema_id.clone(),
                system_prompt: character.content.system_prompt.clone(),
            },
            cover_blob_id,
            cover_file_name,
            cover_mime_type,
        })
    }

    async fn rollback_import(&self, applied: AppliedImportResources) {
        for story_id in applied.story_ids.into_iter().rev() {
            let _ = self.store.delete_story(&story_id).await;
        }
        for resource_id in applied.story_resource_ids.into_iter().rev() {
            let _ = self.store.delete_story_resources(&resource_id).await;
        }
        for character_id in applied.character_ids.into_iter().rev() {
            let _ = self.store.delete_character(&character_id).await;
        }
        for blob_id in applied.blob_ids.into_iter().rev() {
            let _ = self.store.delete_blob(&blob_id).await;
        }
        for player_profile_id in applied.player_profile_ids.into_iter().rev() {
            let _ = self.store.delete_player_profile(&player_profile_id).await;
        }
        for lorebook_id in applied.lorebook_ids.into_iter().rev() {
            let _ = self.store.delete_lorebook(&lorebook_id).await;
        }
        for preset_id in applied.preset_ids.into_iter().rev() {
            let _ = self.store.delete_preset(&preset_id).await;
        }
        for schema_id in applied.schema_ids.into_iter().rev() {
            let _ = self.store.delete_schema(&schema_id).await;
        }
    }

    async fn ensure_import_conflicts(
        &self,
        archive: &DataPackageArchive,
    ) -> Result<(), HandlerError> {
        for preset in &archive.presets {
            if self.store.get_preset(&preset.preset_id).await?.is_some() {
                return Err(HandlerError::DuplicatePreset(preset.preset_id.clone()));
            }
        }
        for schema in &archive.schemas {
            if self.store.get_schema(&schema.schema_id).await?.is_some() {
                return Err(HandlerError::DuplicateSchema(schema.schema_id.clone()));
            }
        }
        for lorebook in &archive.lorebooks {
            if self
                .store
                .get_lorebook(&lorebook.lorebook_id)
                .await?
                .is_some()
            {
                return Err(HandlerError::DuplicateLorebook(
                    lorebook.lorebook_id.clone(),
                ));
            }
        }
        for player_profile in &archive.player_profiles {
            if self
                .store
                .get_player_profile(&player_profile.player_profile_id)
                .await?
                .is_some()
            {
                return Err(HandlerError::DuplicatePlayerProfile(
                    player_profile.player_profile_id.clone(),
                ));
            }
        }
        for character in &archive.characters {
            if self
                .store
                .get_character(&character.character_id)
                .await?
                .is_some()
            {
                return Err(HandlerError::DuplicateCharacter(
                    character.character_id.clone(),
                ));
            }
        }
        for resource in &archive.story_resources {
            if self
                .store
                .get_story_resources(&resource.resource_id)
                .await?
                .is_some()
            {
                return Err(HandlerError::DuplicateStoryResources(
                    resource.resource_id.clone(),
                ));
            }
        }
        for story in &archive.stories {
            if self.store.get_story(&story.story_id).await?.is_some() {
                return Err(HandlerError::DuplicateStory(story.story_id.clone()));
            }
        }

        Ok(())
    }

    async fn ensure_schema_available_for_import(
        &self,
        archive: &DataPackageArchive,
        schema_id: &str,
        owner: &str,
    ) -> Result<(), HandlerError> {
        if archive
            .schemas
            .iter()
            .any(|record| record.schema_id == schema_id)
            || self.store.get_schema(schema_id).await?.is_some()
        {
            return Ok(());
        }

        Err(HandlerError::InvalidDataPackage(format!(
            "{owner} references missing schema '{schema_id}'"
        )))
    }

    async fn ensure_lorebook_available_for_import(
        &self,
        archive: &DataPackageArchive,
        lorebook_id: &str,
        owner: &str,
    ) -> Result<(), HandlerError> {
        if archive
            .lorebooks
            .iter()
            .any(|record| record.lorebook_id == lorebook_id)
            || self.store.get_lorebook(lorebook_id).await?.is_some()
        {
            return Ok(());
        }

        Err(HandlerError::InvalidDataPackage(format!(
            "{owner} references missing lorebook '{lorebook_id}'"
        )))
    }

    async fn ensure_character_available_for_import(
        &self,
        archive: &DataPackageArchive,
        character_id: &str,
        owner: &str,
    ) -> Result<(), HandlerError> {
        if archive
            .characters
            .iter()
            .any(|record| record.character_id == character_id)
            || self.store.get_character(character_id).await?.is_some()
        {
            return Ok(());
        }

        Err(HandlerError::InvalidDataPackage(format!(
            "{owner} references missing character '{character_id}'"
        )))
    }

    async fn ensure_story_resources_available_for_import(
        &self,
        archive: &DataPackageArchive,
        resource_id: &str,
        owner: &str,
    ) -> Result<(), HandlerError> {
        if archive
            .story_resources
            .iter()
            .any(|record| record.resource_id == resource_id)
            || self.store.get_story_resources(resource_id).await?.is_some()
        {
            return Ok(());
        }

        Err(HandlerError::InvalidDataPackage(format!(
            "{owner} references missing story resources '{resource_id}'"
        )))
    }

    async fn validate_story_common_variables_for_import(
        &self,
        archive: &DataPackageArchive,
        story: &StoryRecord,
    ) -> Result<(), HandlerError> {
        let resource = self
            .resolve_story_resources_for_import(archive, &story.resource_id)
            .await?;
        let world_schema = self
            .resolve_schema_for_import(archive, &story.world_schema_id)
            .await?;
        let player_schema = self
            .resolve_schema_for_import(archive, &story.player_schema_id)
            .await?;
        let mut character_fields = HashMap::new();

        for character_id in &resource.character_ids {
            let character_schema_id = self
                .resolve_character_schema_id_for_import(archive, character_id)
                .await?;
            let schema = self
                .resolve_schema_for_import(archive, &character_schema_id)
                .await?;
            character_fields.insert(character_id.clone(), schema.fields);
        }

        validate_common_variables(
            &story.common_variables,
            &resource.character_ids,
            &world_schema.fields,
            &player_schema.fields,
            &character_fields,
        )
        .map_err(HandlerError::InvalidCommonVariable)
    }

    async fn resolve_story_resources_for_import(
        &self,
        archive: &DataPackageArchive,
        resource_id: &str,
    ) -> Result<StoryResourcesRecord, HandlerError> {
        if let Some(record) = archive
            .story_resources
            .iter()
            .find(|record| record.resource_id == resource_id)
        {
            return Ok(record.clone());
        }

        self.store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| {
                HandlerError::InvalidDataPackage(format!("missing story resources '{resource_id}'"))
            })
    }

    async fn resolve_schema_for_import(
        &self,
        archive: &DataPackageArchive,
        schema_id: &str,
    ) -> Result<SchemaRecord, HandlerError> {
        if let Some(record) = archive
            .schemas
            .iter()
            .find(|record| record.schema_id == schema_id)
        {
            return Ok(record.clone());
        }

        self.store.get_schema(schema_id).await?.ok_or_else(|| {
            HandlerError::InvalidDataPackage(format!("missing schema '{schema_id}'"))
        })
    }

    async fn resolve_character_schema_id_for_import(
        &self,
        archive: &DataPackageArchive,
        character_id: &str,
    ) -> Result<String, HandlerError> {
        if let Some(character) = archive
            .characters
            .iter()
            .find(|record| record.character_id == character_id)
        {
            return Ok(character.content.schema_id.clone());
        }

        let character = self
            .store
            .get_character(character_id)
            .await?
            .ok_or_else(|| {
                HandlerError::InvalidDataPackage(format!("missing character '{character_id}'"))
            })?;
        Ok(character.content.schema_id)
    }

    async fn load_preset_record(&self, preset_id: &str) -> Result<PresetRecord, HandlerError> {
        self.store
            .get_preset(preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.to_owned()))
    }

    async fn load_schema_record(&self, schema_id: &str) -> Result<SchemaRecord, HandlerError> {
        self.store
            .get_schema(schema_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSchema(schema_id.to_owned()))
    }

    async fn load_character_record(
        &self,
        character_id: &str,
    ) -> Result<CharacterCardRecord, HandlerError> {
        self.store
            .get_character(character_id)
            .await?
            .ok_or_else(|| HandlerError::MissingCharacter(character_id.to_owned()))
    }

    async fn load_story_resources_record(
        &self,
        resource_id: &str,
    ) -> Result<StoryResourcesRecord, HandlerError> {
        self.store
            .get_story_resources(resource_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStoryResources(resource_id.to_owned()))
    }

    async fn load_story_record(&self, story_id: &str) -> Result<StoryRecord, HandlerError> {
        self.store
            .get_story(story_id)
            .await?
            .ok_or_else(|| HandlerError::MissingStory(story_id.to_owned()))
    }

    async fn build_character_entry(
        &self,
        character_id: &str,
    ) -> Result<DataPackageCharacterEntry, HandlerError> {
        let record = self.load_character_record(character_id).await?;
        let (cover_file_name, cover_content_type, cover_bytes) = match record
            .cover_blob_id
            .as_deref()
        {
            Some(blob_id) => {
                let blob = self
                    .store
                    .get_blob(blob_id)
                    .await?
                    .ok_or_else(|| HandlerError::MissingBlob(blob_id.to_owned()))?;
                let file_name = record
                    .cover_file_name
                    .clone()
                    .or(blob.file_name.clone())
                    .ok_or_else(|| HandlerError::MissingCharacterCover(character_id.to_owned()))?;
                let content_type = record
                    .cover_mime_type
                    .clone()
                    .unwrap_or(blob.content_type.clone());
                (Some(file_name), Some(content_type), Some(blob.bytes))
            }
            None => (None, None, None),
        };

        Ok(DataPackageCharacterEntry {
            character_id: record.character_id.clone(),
            content: protocol::CharacterCardContent {
                id: record.content.id,
                name: record.content.name,
                personality: record.content.personality,
                style: record.content.style,
                schema_id: record.content.schema_id,
                system_prompt: record.content.system_prompt,
            },
            cover_file_name,
            cover_content_type,
            cover_bytes,
        })
    }
}

impl TempDataPackages {
    fn insert_export(&self, export_id: String, file_name: String, bytes: Vec<u8>) {
        let mut exports = self.exports.lock().expect("export slots lock poisoned");
        prune_exports(&mut exports);
        exports.insert(
            export_id,
            PreparedExportSlot {
                file_name,
                bytes,
                expires_at_ms: expires_at_ms(),
            },
        );
    }

    fn export_slot(&self, export_id: &str) -> Option<PreparedExportSlot> {
        let mut exports = self.exports.lock().expect("export slots lock poisoned");
        prune_exports(&mut exports);
        exports.get(export_id).cloned()
    }

    fn insert_import(&self, import_id: String) {
        let mut imports = self.imports.lock().expect("import slots lock poisoned");
        prune_imports(&mut imports);
        imports.insert(
            import_id,
            PreparedImportSlot {
                file_name: None,
                bytes: None,
                expires_at_ms: expires_at_ms(),
            },
        );
    }

    fn store_import_archive(
        &self,
        import_id: &str,
        file_name: Option<String>,
        _size_bytes: u64,
        bytes: Vec<u8>,
    ) -> Option<()> {
        let mut imports = self.imports.lock().expect("import slots lock poisoned");
        prune_imports(&mut imports);
        let slot = imports.get_mut(import_id)?;
        slot.file_name = file_name;
        slot.bytes = Some(bytes);
        slot.expires_at_ms = expires_at_ms();
        Some(())
    }

    fn import_size(&self, import_id: &str) -> Option<u64> {
        let mut imports = self.imports.lock().expect("import slots lock poisoned");
        prune_imports(&mut imports);
        imports
            .get(import_id)
            .and_then(|slot| slot.bytes.as_ref().map(|bytes| bytes.len() as u64))
    }

    fn import_bytes(&self, import_id: &str) -> Option<Option<Vec<u8>>> {
        let mut imports = self.imports.lock().expect("import slots lock poisoned");
        prune_imports(&mut imports);
        imports
            .get(import_id)
            .map(|slot| slot.bytes.as_ref().cloned())
    }

    fn remove_import(&self, import_id: &str) {
        let mut imports = self.imports.lock().expect("import slots lock poisoned");
        prune_imports(&mut imports);
        imports.remove(import_id);
    }
}

impl DataPackageSelection {
    fn from_params(params: DataPackageExportPrepareParams) -> Self {
        Self {
            preset_ids: normalize_ids(params.preset_ids),
            schema_ids: normalize_ids(params.schema_ids),
            lorebook_ids: normalize_ids(params.lorebook_ids),
            player_profile_ids: normalize_ids(params.player_profile_ids),
            character_ids: normalize_ids(params.character_ids),
            story_resource_ids: normalize_ids(params.story_resource_ids),
            story_ids: normalize_ids(params.story_ids),
        }
    }

    fn is_empty(&self) -> bool {
        self.preset_ids.is_empty()
            && self.schema_ids.is_empty()
            && self.lorebook_ids.is_empty()
            && self.player_profile_ids.is_empty()
            && self.character_ids.is_empty()
            && self.story_resource_ids.is_empty()
            && self.story_ids.is_empty()
    }
}

fn package_export_resource_id(export_id: &str) -> String {
    format!("{PACKAGE_EXPORT_RESOURCE_PREFIX}{export_id}")
}

fn package_import_resource_id(import_id: &str) -> String {
    format!("{PACKAGE_IMPORT_RESOURCE_PREFIX}{import_id}")
}

fn normalize_ids(values: Vec<String>) -> BTreeSet<String> {
    values
        .into_iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            }
        })
        .collect()
}

fn ensure_non_empty_id(label: &str, value: &str) -> Result<(), HandlerError> {
    if value.trim().is_empty() {
        return Err(HandlerError::InvalidDataPackage(format!(
            "{label} id must not be empty"
        )));
    }
    Ok(())
}

fn validate_story_graph(graph: &story::StoryGraph) -> Result<(), HandlerError> {
    RuntimeStoryGraph::from_story_graph(graph.clone()).map_err(|error| {
        HandlerError::InvalidDataPackage(match error {
            GraphBuildError::MissingStartNode(node_id) => {
                format!("story graph start node '{node_id}' does not exist")
            }
            GraphBuildError::MissingTargetNode { from, to } => {
                format!("story graph transition from '{from}' points to missing node '{to}'")
            }
            GraphBuildError::DuplicateNodeId(node_id) => {
                format!("story graph contains duplicate node id '{node_id}'")
            }
        })
    })?;
    validate_graph_state_conventions(graph)
        .map_err(|error| HandlerError::InvalidDataPackage(error.to_string()))?;
    Ok(())
}

fn prune_exports(exports: &mut HashMap<String, PreparedExportSlot>) {
    let now = now_timestamp_ms();
    exports.retain(|_, slot| slot.expires_at_ms > now);
}

fn prune_imports(imports: &mut HashMap<String, PreparedImportSlot>) {
    let now = now_timestamp_ms();
    imports.retain(|_, slot| slot.expires_at_ms > now);
}

fn expires_at_ms() -> u64 {
    now_timestamp_ms() + TEMP_DATA_PACKAGE_TTL_MS
}

fn now_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_millis() as u64
}

fn normalize_file_name(file_name: Option<String>) -> Option<String> {
    file_name.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn normalize_content_type(content_type: String) -> String {
    let trimmed = content_type.trim();
    if trimmed.is_empty() {
        "application/octet-stream".to_owned()
    } else {
        trimmed.to_owned()
    }
}
