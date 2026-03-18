use engine::{PromptAgentKind, normalize_agent_preset_config};
use protocol::{
    AgentPresetConfigPayload, JsonRpcResponseMessage, PresetAgentIdPayload, PresetCreateParams,
    PresetDeleteParams, PresetDeletedPayload, PresetEntryCreateParams, PresetEntryDeleteParams,
    PresetEntryDeletedPayload, PresetEntryPayload, PresetEntryUpdateParams, PresetGetParams,
    PresetModuleEntryPayload, PresetModuleEntrySummaryPayload, PresetPayload,
    PresetPromptModulePayload, PresetPromptModuleSummaryPayload, PresetSummaryPayload,
    PresetUpdateParams, PresetsListedPayload, PromptEntryKindPayload, PromptModuleIdPayload,
    ResponseResult,
};
use store::{
    AgentPresetConfig, AgentPromptModuleConfig, AgentPromptModuleEntryConfig, PresetAgentConfigs,
    PresetRecord, PromptEntryKind, PromptModuleId,
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
            agents: preset_configs_from_payload(params.agents)?,
        };
        self.store.save_preset(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Preset(Box::new(preset_payload_from_record(&record))),
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

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Preset(Box::new(preset_payload_from_record(&record))),
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
            .map(|record| preset_summary_payload_from_record(&record))
            .collect::<Vec<_>>();
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
            record.agents = preset_configs_from_payload(agents)?;
        }
        self.store.save_preset(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Preset(Box::new(preset_payload_from_record(&record))),
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

        let kind = prompt_agent_kind_from_payload(params.agent);
        let agent_config = agent_config_mut(&mut record.agents, params.agent);
        let module_id = module_id_from_payload(params.module_id);
        let module = module_mut(agent_config, module_id);
        if module
            .entries
            .iter()
            .any(|entry| entry.entry_id == entry_id)
        {
            return Err(HandlerError::DuplicatePresetEntry {
                preset_id,
                agent: agent_label(params.agent).to_owned(),
                entry_id,
            });
        }

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
        self.store.save_preset(record).await?;

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

        let kind = prompt_agent_kind_from_payload(params.agent);
        let agent_config = agent_config_mut(&mut record.agents, params.agent);
        let module_id = module_id_from_payload(params.module_id);
        let module = module_mut(agent_config, module_id);
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
        self.store.save_preset(record).await?;

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

        let kind = prompt_agent_kind_from_payload(params.agent);
        let agent_config = agent_config_mut(&mut record.agents, params.agent);
        let module_id = module_id_from_payload(params.module_id);
        let module = module_mut(agent_config, module_id);
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
        self.store.save_preset(record).await?;

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

fn preset_configs_from_payload(
    payload: protocol::PresetAgentPayloads,
) -> Result<PresetAgentConfigs, HandlerError> {
    Ok(PresetAgentConfigs {
        planner: preset_config_from_payload(payload.planner, PromptAgentKind::Planner)?,
        architect: preset_config_from_payload(payload.architect, PromptAgentKind::Architect)?,
        director: preset_config_from_payload(payload.director, PromptAgentKind::Director)?,
        actor: preset_config_from_payload(payload.actor, PromptAgentKind::Actor)?,
        narrator: preset_config_from_payload(payload.narrator, PromptAgentKind::Narrator)?,
        keeper: preset_config_from_payload(payload.keeper, PromptAgentKind::Keeper)?,
        replyer: preset_config_from_payload(payload.replyer, PromptAgentKind::Replyer)?,
    })
}

fn preset_config_from_payload(
    payload: AgentPresetConfigPayload,
    agent: PromptAgentKind,
) -> Result<AgentPresetConfig, HandlerError> {
    normalize_agent_preset_config(
        agent,
        AgentPresetConfig {
            temperature: payload.temperature,
            max_tokens: payload.max_tokens,
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

fn module_config_from_payload(payload: PresetPromptModulePayload) -> AgentPromptModuleConfig {
    AgentPromptModuleConfig {
        module_id: module_id_from_payload(payload.module_id),
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
        module_id: module_id_to_payload(config.module_id),
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
                module_id: module_id_to_payload(module.module_id),
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

fn module_mut(
    config: &mut AgentPresetConfig,
    module_id: PromptModuleId,
) -> &mut AgentPromptModuleConfig {
    if let Some(index) = config
        .modules
        .iter()
        .position(|module| module.module_id == module_id)
    {
        return &mut config.modules[index];
    }
    config.modules.push(AgentPromptModuleConfig {
        module_id,
        entries: Vec::new(),
    });
    config
        .modules
        .last_mut()
        .expect("module list should contain the appended module")
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
    }
}

fn module_id_to_payload(module_id: PromptModuleId) -> PromptModuleIdPayload {
    match module_id {
        PromptModuleId::Role => PromptModuleIdPayload::Role,
        PromptModuleId::Task => PromptModuleIdPayload::Task,
        PromptModuleId::StaticContext => PromptModuleIdPayload::StaticContext,
        PromptModuleId::DynamicContext => PromptModuleIdPayload::DynamicContext,
        PromptModuleId::Output => PromptModuleIdPayload::Output,
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
