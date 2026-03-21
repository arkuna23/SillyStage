use std::sync::Arc;

use engine::{
    ArchitectPromptMode, PromptAgentKind, PromptPreview, PromptPreviewActorPurpose,
    PromptPreviewEntrySource, PromptPreviewKeeperPhase, PromptPreviewMessageRole,
    PromptPreviewNarratorPurpose, RuntimePromptPreviewOptions, compact_agent_preset_config,
    normalize_agent_preset_config,
};
use protocol::{
    AgentPresetConfigPayload, ArchitectPromptModePayload, JsonRpcResponseMessage,
    PresetAgentIdPayload, PresetCreateParams, PresetDeleteParams, PresetDeletedPayload,
    PresetEntryCreateParams, PresetEntryDeleteParams, PresetEntryDeletedPayload,
    PresetEntryPayload, PresetEntryUpdateParams, PresetGetParams, PresetModuleEntryPayload,
    PresetModuleEntrySummaryPayload, PresetPayload, PresetPreviewRuntimeParams,
    PresetPreviewTemplateParams, PresetPromptModulePayload, PresetPromptModuleSummaryPayload,
    PresetPromptPreviewEntryPayload, PresetPromptPreviewMessagePayload,
    PresetPromptPreviewModulePayload, PresetPromptPreviewPayload, PresetSummaryPayload,
    PresetUpdateParams, PresetsListedPayload, PromptEntryKindPayload, PromptMessageRolePayload,
    PromptModuleIdPayload, PromptPreviewActorPurposePayload, PromptPreviewEntrySourcePayload,
    PromptPreviewKeeperPhasePayload, PromptPreviewKindPayload, PromptPreviewMessageRolePayload,
    PromptPreviewNarratorPurposePayload, ResponseResult,
};
use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PresetAgentConfigs,
    PresetRecord, PromptEntryKind, PromptMessageRole, PromptModuleId, Store,
};

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_preset_create(
        &self,
        request_id: &str,
        params: PresetCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        if self.store.get_preset(&preset_id).await?.is_some() {
            return Err(HandlerError::DuplicatePreset(preset_id));
        }

        let record = PresetRecord {
            preset_id: preset_id.clone(),
            display_name: params.display_name,
            agents: compact_preset_configs_from_payload(params.agents)?,
        };
        self.store.save_preset(record.clone()).await?;
        let expanded = expand_preset_record(&record)?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Preset(Box::new(preset_payload_from_record(&expanded))),
        ))
    }

    pub(crate) async fn handle_preset_get(
        &self,
        request_id: &str,
        params: PresetGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;
        let expanded = expand_preset_record(&record)?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Preset(Box::new(preset_payload_from_record(&expanded))),
        ))
    }

    pub(crate) async fn handle_preset_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut presets = self
            .store
            .list_presets()
            .await?
            .into_iter()
            .map(|record| {
                expand_preset_record(&record)
                    .map(|expanded| preset_summary_payload_from_record(&expanded))
            })
            .collect::<Result<Vec<_>, _>>()?;
        presets.sort_by(|left, right| left.preset_id.cmp(&right.preset_id));

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PresetsListed(PresetsListedPayload { presets }),
        ))
    }

    pub(crate) async fn handle_preset_update(
        &self,
        request_id: &str,
        params: PresetUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let mut record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;

        if let Some(display_name) = params.display_name {
            record.display_name = display_name;
        }
        if let Some(agents) = params.agents {
            record.agents = compact_preset_configs_from_payload(agents)?;
        }
        self.store.save_preset(record.clone()).await?;
        let expanded = expand_preset_record(&record)?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Preset(Box::new(preset_payload_from_record(&expanded))),
        ))
    }

    pub(crate) async fn handle_preset_delete(
        &self,
        request_id: &str,
        params: PresetDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        ensure_preset_not_in_use(self, &preset_id).await?;
        self.store
            .delete_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PresetDeleted(PresetDeletedPayload { preset_id }),
        ))
    }

    pub(crate) async fn handle_preset_entry_create(
        &self,
        request_id: &str,
        params: PresetEntryCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let entry_id = normalize_entry_id(&params.entry_id)?;
        let mut record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;
        record = expand_preset_record(&record)?;

        let kind = prompt_agent_kind_from_payload(params.agent);
        let agent_config = agent_config_mut(&mut record.agents, params.agent);
        if agent_config.modules.iter().any(|candidate| {
            candidate
                .entries
                .iter()
                .any(|entry| entry.entry_id == entry_id)
        }) {
            return Err(HandlerError::DuplicatePresetEntry {
                preset_id,
                agent: agent_label(params.agent).to_owned(),
                entry_id,
            });
        }
        let module_id = module_id_from_payload(params.module_id.clone());
        let module = module_mut(agent_config, &module_id).ok_or_else(|| {
            HandlerError::MissingPresetModule {
                preset_id: preset_id.clone(),
                agent: agent_label(params.agent).to_owned(),
                module_id: module_id.as_str().to_owned(),
            }
        })?;

        let order = params
            .order
            .unwrap_or_else(|| next_custom_entry_order(&module.entries));
        module.entries.push(AgentPromptModuleEntryConfig {
            entry_id: entry_id.clone(),
            display_name: params.display_name.trim().to_owned(),
            kind: PromptEntryKind::CustomText,
            enabled: params.enabled,
            order,
            required: false,
            text: Some(params.text),
            context_key: None,
        });

        *agent_config = normalize_agent_preset_config(kind, agent_config.clone())
            .map_err(|error| HandlerError::InvalidPresetDefinition(error.to_string()))?;
        let entry = find_entry(
            agent_config_ref(&record.agents, params.agent),
            module_id,
            &entry_id,
        )
        .cloned()
        .ok_or_else(|| HandlerError::MissingPresetEntry {
            preset_id: preset_id.clone(),
            agent: agent_label(params.agent).to_owned(),
            entry_id: entry_id.clone(),
        })?;
        self.store
            .save_preset(compact_preset_record(&record)?)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PresetEntry(Box::new(PresetEntryPayload {
                preset_id,
                agent: params.agent,
                module_id: params.module_id,
                entry: entry_payload_from_config(&entry),
            })),
        ))
    }

    pub(crate) async fn handle_preset_entry_update(
        &self,
        request_id: &str,
        params: PresetEntryUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let entry_id = normalize_entry_id(&params.entry_id)?;
        let mut record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;
        record = expand_preset_record(&record)?;

        let kind = prompt_agent_kind_from_payload(params.agent);
        let agent_config = agent_config_mut(&mut record.agents, params.agent);
        let module_id = module_id_from_payload(params.module_id.clone());
        let module = module_mut(agent_config, &module_id).ok_or_else(|| {
            HandlerError::MissingPresetModule {
                preset_id: preset_id.clone(),
                agent: agent_label(params.agent).to_owned(),
                module_id: module_id.as_str().to_owned(),
            }
        })?;
        let entry = module
            .entries
            .iter_mut()
            .find(|entry| entry.entry_id == entry_id)
            .ok_or_else(|| HandlerError::MissingPresetEntry {
                preset_id: preset_id.clone(),
                agent: agent_label(params.agent).to_owned(),
                entry_id: entry_id.clone(),
            })?;

        match entry.kind {
            PromptEntryKind::CustomText => {
                if let Some(display_name) = params.display_name {
                    entry.display_name = display_name.trim().to_owned();
                }
                if let Some(text) = params.text {
                    entry.text = Some(text);
                }
            }
            PromptEntryKind::BuiltInText | PromptEntryKind::BuiltInContextRef => {
                if params.display_name.is_some() || params.text.is_some() {
                    return Err(HandlerError::BuiltInPresetEntryImmutable(entry_id));
                }
            }
        }
        if let Some(enabled) = params.enabled {
            entry.enabled = enabled;
        }
        if let Some(order) = params.order {
            entry.order = order;
        }

        *agent_config = normalize_agent_preset_config(kind, agent_config.clone())
            .map_err(|error| HandlerError::InvalidPresetDefinition(error.to_string()))?;
        let entry = find_entry(
            agent_config_ref(&record.agents, params.agent),
            module_id,
            &params.entry_id,
        )
        .cloned()
        .ok_or_else(|| HandlerError::MissingPresetEntry {
            preset_id: preset_id.clone(),
            agent: agent_label(params.agent).to_owned(),
            entry_id: params.entry_id.clone(),
        })?;
        self.store
            .save_preset(compact_preset_record(&record)?)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PresetEntry(Box::new(PresetEntryPayload {
                preset_id,
                agent: params.agent,
                module_id: params.module_id,
                entry: entry_payload_from_config(&entry),
            })),
        ))
    }

    pub(crate) async fn handle_preset_entry_delete(
        &self,
        request_id: &str,
        params: PresetEntryDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let entry_id = normalize_entry_id(&params.entry_id)?;
        let mut record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;
        record = expand_preset_record(&record)?;

        let kind = prompt_agent_kind_from_payload(params.agent);
        let agent_config = agent_config_mut(&mut record.agents, params.agent);
        let module_id = module_id_from_payload(params.module_id.clone());
        let module = module_mut(agent_config, &module_id).ok_or_else(|| {
            HandlerError::MissingPresetModule {
                preset_id: preset_id.clone(),
                agent: agent_label(params.agent).to_owned(),
                module_id: module_id.as_str().to_owned(),
            }
        })?;
        let index = module
            .entries
            .iter()
            .position(|entry| entry.entry_id == entry_id)
            .ok_or_else(|| HandlerError::MissingPresetEntry {
                preset_id: preset_id.clone(),
                agent: agent_label(params.agent).to_owned(),
                entry_id: entry_id.clone(),
            })?;
        if module.entries[index].kind != PromptEntryKind::CustomText {
            return Err(HandlerError::BuiltInPresetEntryDeleteForbidden(entry_id));
        }
        module.entries.remove(index);

        *agent_config = normalize_agent_preset_config(kind, agent_config.clone())
            .map_err(|error| HandlerError::InvalidPresetDefinition(error.to_string()))?;
        self.store
            .save_preset(compact_preset_record(&record)?)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PresetEntryDeleted(PresetEntryDeletedPayload {
                preset_id,
                agent: params.agent,
                module_id: params.module_id,
                entry_id,
            }),
        ))
    }

    pub(crate) async fn handle_preset_preview_template(
        &self,
        request_id: &str,
        params: PresetPreviewTemplateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;
        let expanded = expand_preset_record(&record)?;
        validate_preview_mode(params.agent, params.architect_mode)?;
        if let Some(module_id) = &params.module_id {
            ensure_preview_module_exists(&expanded, params.agent, module_id)?;
        }

        let preview = self
            .manager
            .preview_prompt_template(
                &preset_id,
                prompt_agent_kind_from_payload(params.agent),
                params
                    .module_id
                    .as_ref()
                    .map(|module_id| module_id_from_payload(module_id.clone()))
                    .as_ref(),
                params.architect_mode.map(architect_mode_from_payload),
            )
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::PresetPromptPreview(Box::new(prompt_preview_payload(
                preset_id,
                params.agent,
                params.module_id,
                params.architect_mode,
                PromptPreviewKindPayload::Template,
                preview,
            ))),
        ))
    }

    pub(crate) async fn handle_preset_preview_runtime(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: PresetPreviewRuntimeParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let preset_id = normalize_preset_id(&params.preset_id)?;
        let record = self
            .store
            .get_preset(&preset_id)
            .await?
            .ok_or_else(|| HandlerError::MissingPreset(preset_id.clone()))?;
        let expanded = expand_preset_record(&record)?;
        validate_preview_mode(params.agent, params.architect_mode)?;
        if let Some(module_id) = &params.module_id {
            ensure_preview_module_exists(&expanded, params.agent, module_id)?;
        }

        let agent = prompt_agent_kind_from_payload(params.agent);
        let module_id = params.module_id.clone().map(module_id_from_payload);
        let architect_mode = params.architect_mode.map(architect_mode_from_payload);
        let preview = match agent {
            PromptAgentKind::Planner => {
                let resource_id = params.resource_id.as_deref().ok_or_else(|| {
                    HandlerError::InvalidPromptPreview(
                        "planner runtime preview requires resource_id".to_owned(),
                    )
                })?;
                self.manager
                    .preview_prompt_runtime_for_resource(
                        &preset_id,
                        agent,
                        module_id.as_ref(),
                        architect_mode,
                        resource_id,
                    )
                    .await?
            }
            PromptAgentKind::Architect => match architect_mode.ok_or_else(|| {
                HandlerError::InvalidPromptPreview(
                    "architect prompt preview requires architect_mode".to_owned(),
                )
            })? {
                ArchitectPromptMode::Graph => {
                    let resource_id = params.resource_id.as_deref().ok_or_else(|| {
                        HandlerError::InvalidPromptPreview(
                            "architect graph runtime preview requires resource_id".to_owned(),
                        )
                    })?;
                    self.manager
                        .preview_prompt_runtime_for_resource(
                            &preset_id,
                            agent,
                            module_id.as_ref(),
                            Some(ArchitectPromptMode::Graph),
                            resource_id,
                        )
                        .await?
                }
                ArchitectPromptMode::DraftInit | ArchitectPromptMode::DraftContinue => {
                    let draft_id = params.draft_id.as_deref().ok_or_else(|| {
                        HandlerError::InvalidPromptPreview(
                            "architect draft runtime preview requires draft_id".to_owned(),
                        )
                    })?;
                    self.manager
                        .preview_prompt_runtime_for_draft(
                            &preset_id,
                            module_id.as_ref(),
                            architect_mode.expect("validated architect mode should exist"),
                            draft_id,
                        )
                        .await?
                }
            },
            PromptAgentKind::Director
            | PromptAgentKind::Actor
            | PromptAgentKind::Narrator
            | PromptAgentKind::Keeper
            | PromptAgentKind::Replyer => {
                let session_id = session_id
                    .as_deref()
                    .ok_or(HandlerError::MissingSessionId)?
                    .to_owned();
                self.manager
                    .preview_prompt_runtime_for_session(
                        &preset_id,
                        agent,
                        module_id.as_ref(),
                        &session_id,
                        runtime_preview_options_from_params(&params),
                    )
                    .await?
            }
        };

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            session_id,
            ResponseResult::PresetPromptPreview(Box::new(prompt_preview_payload(
                preset_id,
                params.agent,
                params.module_id,
                params.architect_mode,
                PromptPreviewKindPayload::Runtime,
                preview,
            ))),
        ))
    }
}

async fn ensure_preset_not_in_use(handler: &Handler, preset_id: &str) -> Result<(), HandlerError> {
    if handler
        .store
        .list_story_drafts()
        .await?
        .into_iter()
        .any(|draft| draft.preset_id == preset_id)
    {
        return Err(HandlerError::PresetInUse(preset_id.to_owned()));
    }

    if handler
        .store
        .list_sessions()
        .await?
        .into_iter()
        .any(|session| session.binding.preset_id == preset_id)
    {
        return Err(HandlerError::PresetInUse(preset_id.to_owned()));
    }

    Ok(())
}

fn normalize_preset_id(preset_id: &str) -> Result<String, HandlerError> {
    let trimmed = preset_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyPresetId);
    }
    Ok(trimmed.to_owned())
}

fn normalize_entry_id(entry_id: &str) -> Result<String, HandlerError> {
    let trimmed = entry_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyPresetEntryId);
    }
    Ok(trimmed.to_owned())
}

pub(super) async fn migrate_preset_storage(store: &Arc<dyn Store>) -> Result<(), HandlerError> {
    for record in store.list_presets().await? {
        let compacted = compact_preset_record(&record)?;
        if compacted.agents != record.agents {
            store.save_preset(compacted).await?;
        }
    }
    Ok(())
}

fn compact_preset_configs_from_payload(
    payload: protocol::PresetAgentPayloads,
) -> Result<PresetAgentConfigs, HandlerError> {
    Ok(PresetAgentConfigs {
        planner: compact_preset_config_from_payload(payload.planner, PromptAgentKind::Planner)?,
        architect: compact_preset_config_from_payload(
            payload.architect,
            PromptAgentKind::Architect,
        )?,
        director: compact_preset_config_from_payload(payload.director, PromptAgentKind::Director)?,
        actor: compact_preset_config_from_payload(payload.actor, PromptAgentKind::Actor)?,
        narrator: compact_preset_config_from_payload(payload.narrator, PromptAgentKind::Narrator)?,
        keeper: compact_preset_config_from_payload(payload.keeper, PromptAgentKind::Keeper)?,
        replyer: compact_preset_config_from_payload(payload.replyer, PromptAgentKind::Replyer)?,
    })
}

fn compact_preset_config_from_payload(
    payload: AgentPresetConfigPayload,
    agent: PromptAgentKind,
) -> Result<AgentPresetConfig, HandlerError> {
    compact_agent_preset_config(
        agent,
        AgentPresetConfig {
            temperature: payload.temperature,
            max_tokens: payload.max_tokens,
            director_shared_history_limit: payload.director_shared_history_limit,
            actor_shared_history_limit: payload.actor_shared_history_limit,
            actor_private_memory_limit: payload.actor_private_memory_limit,
            narrator_shared_history_limit: payload.narrator_shared_history_limit,
            replyer_session_history_limit: payload.replyer_session_history_limit,
            extra: payload.extra,
            modules: payload
                .modules
                .into_iter()
                .map(module_config_from_payload)
                .collect(),
        },
    )
    .map_err(|error| HandlerError::InvalidPresetDefinition(error.to_string()))
}

fn expand_preset_record(record: &PresetRecord) -> Result<PresetRecord, HandlerError> {
    Ok(PresetRecord {
        preset_id: record.preset_id.clone(),
        display_name: record.display_name.clone(),
        agents: expand_preset_configs(&record.agents)?,
    })
}

pub(super) fn compact_preset_record(record: &PresetRecord) -> Result<PresetRecord, HandlerError> {
    Ok(PresetRecord {
        preset_id: record.preset_id.clone(),
        display_name: record.display_name.clone(),
        agents: compact_preset_configs(&record.agents)?,
    })
}

fn expand_preset_configs(configs: &PresetAgentConfigs) -> Result<PresetAgentConfigs, HandlerError> {
    Ok(PresetAgentConfigs {
        planner: expand_preset_config(&configs.planner, PromptAgentKind::Planner)?,
        architect: expand_preset_config(&configs.architect, PromptAgentKind::Architect)?,
        director: expand_preset_config(&configs.director, PromptAgentKind::Director)?,
        actor: expand_preset_config(&configs.actor, PromptAgentKind::Actor)?,
        narrator: expand_preset_config(&configs.narrator, PromptAgentKind::Narrator)?,
        keeper: expand_preset_config(&configs.keeper, PromptAgentKind::Keeper)?,
        replyer: expand_preset_config(&configs.replyer, PromptAgentKind::Replyer)?,
    })
}

fn compact_preset_configs(
    configs: &PresetAgentConfigs,
) -> Result<PresetAgentConfigs, HandlerError> {
    Ok(PresetAgentConfigs {
        planner: compact_preset_config(&configs.planner, PromptAgentKind::Planner)?,
        architect: compact_preset_config(&configs.architect, PromptAgentKind::Architect)?,
        director: compact_preset_config(&configs.director, PromptAgentKind::Director)?,
        actor: compact_preset_config(&configs.actor, PromptAgentKind::Actor)?,
        narrator: compact_preset_config(&configs.narrator, PromptAgentKind::Narrator)?,
        keeper: compact_preset_config(&configs.keeper, PromptAgentKind::Keeper)?,
        replyer: compact_preset_config(&configs.replyer, PromptAgentKind::Replyer)?,
    })
}

fn expand_preset_config(
    config: &AgentPresetConfig,
    agent: PromptAgentKind,
) -> Result<AgentPresetConfig, HandlerError> {
    normalize_agent_preset_config(agent, config.clone())
        .map_err(|error| HandlerError::InvalidPresetDefinition(error.to_string()))
}

fn compact_preset_config(
    config: &AgentPresetConfig,
    agent: PromptAgentKind,
) -> Result<AgentPresetConfig, HandlerError> {
    compact_agent_preset_config(agent, config.clone())
        .map_err(|error| HandlerError::InvalidPresetDefinition(error.to_string()))
}

fn module_config_from_payload(payload: PresetPromptModulePayload) -> AgentPromptModuleConfig {
    AgentPromptModuleConfig {
        module_id: module_id_from_payload(payload.module_id),
        display_name: payload.display_name,
        message_role: message_role_from_payload(payload.message_role),
        order: payload.order,
        entries: payload
            .entries
            .into_iter()
            .map(entry_config_from_payload)
            .collect(),
    }
}

fn entry_config_from_payload(payload: PresetModuleEntryPayload) -> AgentPromptModuleEntryConfig {
    AgentPromptModuleEntryConfig {
        entry_id: payload.entry_id,
        display_name: payload.display_name,
        kind: entry_kind_from_payload(payload.kind),
        enabled: payload.enabled,
        order: payload.order,
        required: payload.required,
        text: payload.text,
        context_key: payload.context_key,
    }
}

fn validate_preview_mode(
    agent: PresetAgentIdPayload,
    architect_mode: Option<ArchitectPromptModePayload>,
) -> Result<(), HandlerError> {
    if matches!(agent, PresetAgentIdPayload::Architect) {
        if architect_mode.is_none() {
            return Err(HandlerError::InvalidPromptPreview(
                "architect prompt preview requires architect_mode".to_owned(),
            ));
        }
    } else if architect_mode.is_some() {
        return Err(HandlerError::InvalidPromptPreview(format!(
            "architect_mode is only valid for architect previews, got agent '{}'",
            agent_label(agent)
        )));
    }

    Ok(())
}

fn ensure_preview_module_exists(
    record: &PresetRecord,
    agent: PresetAgentIdPayload,
    module_id: &PromptModuleIdPayload,
) -> Result<(), HandlerError> {
    let module_id_store = module_id_from_payload(module_id.clone());
    let agent_config = agent_config_ref(&record.agents, agent);
    if agent_config
        .modules
        .iter()
        .any(|module| module.module_id == module_id_store)
    {
        Ok(())
    } else {
        Err(HandlerError::MissingPresetModule {
            preset_id: record.preset_id.clone(),
            agent: agent_label(agent).to_owned(),
            module_id: module_id_store.as_str().to_owned(),
        })
    }
}

fn runtime_preview_options_from_params(
    params: &PresetPreviewRuntimeParams,
) -> RuntimePromptPreviewOptions {
    RuntimePromptPreviewOptions {
        character_id: params.character_id.clone(),
        actor_purpose: params.actor_purpose.map(actor_purpose_from_payload),
        narrator_purpose: params.narrator_purpose.map(narrator_purpose_from_payload),
        keeper_phase: params.keeper_phase.map(keeper_phase_from_payload),
        previous_node_id: params.previous_node_id.clone(),
        player_input: params.player_input.clone(),
        reply_limit: params.reply_limit.map(|value| value as usize),
    }
}

fn prompt_preview_payload(
    preset_id: String,
    agent: PresetAgentIdPayload,
    module_id: Option<PromptModuleIdPayload>,
    architect_mode: Option<ArchitectPromptModePayload>,
    preview_kind: PromptPreviewKindPayload,
    preview: PromptPreview,
) -> PresetPromptPreviewPayload {
    PresetPromptPreviewPayload {
        preset_id,
        agent,
        module_id,
        architect_mode,
        preview_kind,
        message_role: match preview.message_role {
            PromptPreviewMessageRole::System => PromptPreviewMessageRolePayload::System,
            PromptPreviewMessageRole::User => PromptPreviewMessageRolePayload::User,
            PromptPreviewMessageRole::Full => PromptPreviewMessageRolePayload::Full,
        },
        messages: preview
            .messages
            .into_iter()
            .map(|message| PresetPromptPreviewMessagePayload {
                role: message_role_to_payload(message.role),
                modules: message
                    .modules
                    .into_iter()
                    .map(|module| PresetPromptPreviewModulePayload {
                        module_id: module_id_to_payload(module.module_id),
                        display_name: module.display_name,
                        order: module.order,
                        entries: module
                            .entries
                            .into_iter()
                            .map(|entry| PresetPromptPreviewEntryPayload {
                                entry_id: entry.entry_id,
                                display_name: entry.display_name,
                                kind: entry_kind_to_payload(entry.kind),
                                order: entry.order,
                                source: match entry.source {
                                    PromptPreviewEntrySource::Preset => {
                                        PromptPreviewEntrySourcePayload::Preset
                                    }
                                    PromptPreviewEntrySource::Synthetic => {
                                        PromptPreviewEntrySourcePayload::Synthetic
                                    }
                                },
                                compiled_text: entry.compiled_text,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        unresolved_context_keys: preview.unresolved_context_keys,
    }
}

fn architect_mode_from_payload(mode: ArchitectPromptModePayload) -> ArchitectPromptMode {
    match mode {
        ArchitectPromptModePayload::Graph => ArchitectPromptMode::Graph,
        ArchitectPromptModePayload::DraftInit => ArchitectPromptMode::DraftInit,
        ArchitectPromptModePayload::DraftContinue => ArchitectPromptMode::DraftContinue,
    }
}

fn actor_purpose_from_payload(
    purpose: PromptPreviewActorPurposePayload,
) -> PromptPreviewActorPurpose {
    match purpose {
        PromptPreviewActorPurposePayload::AdvanceGoal => PromptPreviewActorPurpose::AdvanceGoal,
        PromptPreviewActorPurposePayload::ReactToPlayer => PromptPreviewActorPurpose::ReactToPlayer,
        PromptPreviewActorPurposePayload::CommentOnScene => {
            PromptPreviewActorPurpose::CommentOnScene
        }
    }
}

fn narrator_purpose_from_payload(
    purpose: PromptPreviewNarratorPurposePayload,
) -> PromptPreviewNarratorPurpose {
    match purpose {
        PromptPreviewNarratorPurposePayload::DescribeTransition => {
            PromptPreviewNarratorPurpose::DescribeTransition
        }
        PromptPreviewNarratorPurposePayload::DescribeScene => {
            PromptPreviewNarratorPurpose::DescribeScene
        }
        PromptPreviewNarratorPurposePayload::DescribeResult => {
            PromptPreviewNarratorPurpose::DescribeResult
        }
    }
}

fn keeper_phase_from_payload(phase: PromptPreviewKeeperPhasePayload) -> PromptPreviewKeeperPhase {
    match phase {
        PromptPreviewKeeperPhasePayload::AfterPlayerInput => {
            PromptPreviewKeeperPhase::AfterPlayerInput
        }
        PromptPreviewKeeperPhasePayload::AfterTurnOutputs => {
            PromptPreviewKeeperPhase::AfterTurnOutputs
        }
    }
}

fn preset_payload_from_record(record: &PresetRecord) -> PresetPayload {
    PresetPayload {
        preset_id: record.preset_id.clone(),
        display_name: record.display_name.clone(),
        agents: protocol::PresetAgentPayloads {
            planner: payload_from_config(&record.agents.planner),
            architect: payload_from_config(&record.agents.architect),
            director: payload_from_config(&record.agents.director),
            actor: payload_from_config(&record.agents.actor),
            narrator: payload_from_config(&record.agents.narrator),
            keeper: payload_from_config(&record.agents.keeper),
            replyer: payload_from_config(&record.agents.replyer),
        },
    }
}

fn payload_from_config(config: &AgentPresetConfig) -> AgentPresetConfigPayload {
    AgentPresetConfigPayload {
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        director_shared_history_limit: config.director_shared_history_limit,
        actor_shared_history_limit: config.actor_shared_history_limit,
        actor_private_memory_limit: config.actor_private_memory_limit,
        narrator_shared_history_limit: config.narrator_shared_history_limit,
        replyer_session_history_limit: config.replyer_session_history_limit,
        extra: config.extra.clone(),
        modules: config
            .modules
            .iter()
            .map(module_payload_from_config)
            .collect(),
    }
}

fn module_payload_from_config(config: &AgentPromptModuleConfig) -> PresetPromptModulePayload {
    PresetPromptModulePayload {
        module_id: module_id_to_payload(config.module_id.clone()),
        display_name: config.display_name.clone(),
        message_role: message_role_to_payload(config.message_role),
        order: config.order,
        entries: config
            .entries
            .iter()
            .map(entry_payload_from_config)
            .collect(),
    }
}

fn entry_payload_from_config(config: &AgentPromptModuleEntryConfig) -> PresetModuleEntryPayload {
    PresetModuleEntryPayload {
        entry_id: config.entry_id.clone(),
        display_name: config.display_name.clone(),
        kind: entry_kind_to_payload(config.kind),
        enabled: config.enabled,
        order: config.order,
        required: config.required,
        text: config.text.clone(),
        context_key: config.context_key.clone(),
    }
}

fn preset_summary_payload_from_record(record: &PresetRecord) -> PresetSummaryPayload {
    PresetSummaryPayload {
        preset_id: record.preset_id.clone(),
        display_name: record.display_name.clone(),
        agents: protocol::PresetAgentSummaryPayloads {
            planner: summary_payload_from_config(&record.agents.planner),
            architect: summary_payload_from_config(&record.agents.architect),
            director: summary_payload_from_config(&record.agents.director),
            actor: summary_payload_from_config(&record.agents.actor),
            narrator: summary_payload_from_config(&record.agents.narrator),
            keeper: summary_payload_from_config(&record.agents.keeper),
            replyer: summary_payload_from_config(&record.agents.replyer),
        },
    }
}

fn summary_payload_from_config(
    config: &AgentPresetConfig,
) -> protocol::AgentPresetConfigSummaryPayload {
    protocol::AgentPresetConfigSummaryPayload {
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        director_shared_history_limit: config.director_shared_history_limit,
        actor_shared_history_limit: config.actor_shared_history_limit,
        actor_private_memory_limit: config.actor_private_memory_limit,
        narrator_shared_history_limit: config.narrator_shared_history_limit,
        replyer_session_history_limit: config.replyer_session_history_limit,
        extra: config.extra.clone(),
        module_count: config.modules.len(),
        entry_count: config
            .modules
            .iter()
            .map(|module| module.entries.len())
            .sum(),
        modules: config
            .modules
            .iter()
            .map(|module| PresetPromptModuleSummaryPayload {
                module_id: module_id_to_payload(module.module_id.clone()),
                display_name: module.display_name.clone(),
                message_role: message_role_to_payload(module.message_role),
                order: module.order,
                entry_count: module.entries.len(),
                entries: module
                    .entries
                    .iter()
                    .map(|entry| PresetModuleEntrySummaryPayload {
                        entry_id: entry.entry_id.clone(),
                        display_name: entry.display_name.clone(),
                        kind: entry_kind_to_payload(entry.kind),
                        enabled: entry.enabled,
                        order: entry.order,
                        required: entry.required,
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn agent_config_mut<'a>(
    configs: &'a mut PresetAgentConfigs,
    agent: PresetAgentIdPayload,
) -> &'a mut AgentPresetConfig {
    match agent {
        PresetAgentIdPayload::Planner => &mut configs.planner,
        PresetAgentIdPayload::Architect => &mut configs.architect,
        PresetAgentIdPayload::Director => &mut configs.director,
        PresetAgentIdPayload::Actor => &mut configs.actor,
        PresetAgentIdPayload::Narrator => &mut configs.narrator,
        PresetAgentIdPayload::Keeper => &mut configs.keeper,
        PresetAgentIdPayload::Replyer => &mut configs.replyer,
    }
}

fn agent_config_ref<'a>(
    configs: &'a PresetAgentConfigs,
    agent: PresetAgentIdPayload,
) -> &'a AgentPresetConfig {
    match agent {
        PresetAgentIdPayload::Planner => &configs.planner,
        PresetAgentIdPayload::Architect => &configs.architect,
        PresetAgentIdPayload::Director => &configs.director,
        PresetAgentIdPayload::Actor => &configs.actor,
        PresetAgentIdPayload::Narrator => &configs.narrator,
        PresetAgentIdPayload::Keeper => &configs.keeper,
        PresetAgentIdPayload::Replyer => &configs.replyer,
    }
}

fn module_mut<'a>(
    config: &'a mut AgentPresetConfig,
    module_id: &PromptModuleId,
) -> Option<&'a mut AgentPromptModuleConfig> {
    config
        .modules
        .iter_mut()
        .find(|module| module.module_id == *module_id)
}

fn find_entry<'a>(
    config: &'a AgentPresetConfig,
    module_id: PromptModuleId,
    entry_id: &str,
) -> Option<&'a AgentPromptModuleEntryConfig> {
    config
        .modules
        .iter()
        .find(|module| module.module_id == module_id)
        .and_then(|module| {
            module
                .entries
                .iter()
                .find(|entry| entry.entry_id == entry_id)
        })
}

fn next_custom_entry_order(entries: &[AgentPromptModuleEntryConfig]) -> i32 {
    entries.iter().map(|entry| entry.order).max().unwrap_or(0) + 10
}

fn prompt_agent_kind_from_payload(agent: PresetAgentIdPayload) -> PromptAgentKind {
    match agent {
        PresetAgentIdPayload::Planner => PromptAgentKind::Planner,
        PresetAgentIdPayload::Architect => PromptAgentKind::Architect,
        PresetAgentIdPayload::Director => PromptAgentKind::Director,
        PresetAgentIdPayload::Actor => PromptAgentKind::Actor,
        PresetAgentIdPayload::Narrator => PromptAgentKind::Narrator,
        PresetAgentIdPayload::Keeper => PromptAgentKind::Keeper,
        PresetAgentIdPayload::Replyer => PromptAgentKind::Replyer,
    }
}

fn agent_label(agent: PresetAgentIdPayload) -> &'static str {
    match agent {
        PresetAgentIdPayload::Planner => "planner",
        PresetAgentIdPayload::Architect => "architect",
        PresetAgentIdPayload::Director => "director",
        PresetAgentIdPayload::Actor => "actor",
        PresetAgentIdPayload::Narrator => "narrator",
        PresetAgentIdPayload::Keeper => "keeper",
        PresetAgentIdPayload::Replyer => "replyer",
    }
}

fn module_id_from_payload(module_id: PromptModuleIdPayload) -> PromptModuleId {
    match module_id {
        PromptModuleIdPayload::Role => PromptModuleId::Role,
        PromptModuleIdPayload::Task => PromptModuleId::Task,
        PromptModuleIdPayload::StaticContext => PromptModuleId::StaticContext,
        PromptModuleIdPayload::DynamicContext => PromptModuleId::DynamicContext,
        PromptModuleIdPayload::Output => PromptModuleId::Output,
        PromptModuleIdPayload::Custom(value) => PromptModuleId::Custom(value),
    }
}

fn module_id_to_payload(module_id: PromptModuleId) -> PromptModuleIdPayload {
    match module_id {
        PromptModuleId::Role => PromptModuleIdPayload::Role,
        PromptModuleId::Task => PromptModuleIdPayload::Task,
        PromptModuleId::StaticContext => PromptModuleIdPayload::StaticContext,
        PromptModuleId::DynamicContext => PromptModuleIdPayload::DynamicContext,
        PromptModuleId::Output => PromptModuleIdPayload::Output,
        PromptModuleId::Custom(value) => PromptModuleIdPayload::Custom(value),
    }
}

fn message_role_from_payload(role: PromptMessageRolePayload) -> PromptMessageRole {
    match role {
        PromptMessageRolePayload::System => PromptMessageRole::System,
        PromptMessageRolePayload::User => PromptMessageRole::User,
    }
}

fn message_role_to_payload(role: PromptMessageRole) -> PromptMessageRolePayload {
    match role {
        PromptMessageRole::System => PromptMessageRolePayload::System,
        PromptMessageRole::User => PromptMessageRolePayload::User,
    }
}

fn entry_kind_from_payload(kind: PromptEntryKindPayload) -> PromptEntryKind {
    match kind {
        PromptEntryKindPayload::BuiltInText => PromptEntryKind::BuiltInText,
        PromptEntryKindPayload::BuiltInContextRef => PromptEntryKind::BuiltInContextRef,
        PromptEntryKindPayload::CustomText => PromptEntryKind::CustomText,
    }
}

fn entry_kind_to_payload(kind: PromptEntryKind) -> PromptEntryKindPayload {
    match kind {
        PromptEntryKind::BuiltInText => PromptEntryKindPayload::BuiltInText,
        PromptEntryKind::BuiltInContextRef => PromptEntryKindPayload::BuiltInContextRef,
        PromptEntryKind::CustomText => PromptEntryKindPayload::CustomText,
    }
}
