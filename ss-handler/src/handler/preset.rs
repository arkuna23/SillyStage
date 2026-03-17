use protocol::{
    JsonRpcResponseMessage, PresetCreateParams, PresetDeleteParams, PresetDeletedPayload,
    PresetGetParams, PresetPayload, PresetSummaryPayload, PresetUpdateParams, PresetsListedPayload,
    ResponseResult,
};
use store::{AgentPresetConfig, AgentPromptEntryConfig, PresetAgentConfigs, PresetRecord};

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
            agents: preset_configs_from_payload(params.agents),
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
            record.agents = preset_configs_from_payload(agents);
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

fn preset_configs_from_payload(payload: protocol::PresetAgentPayloads) -> PresetAgentConfigs {
    PresetAgentConfigs {
        planner: preset_config_from_payload(payload.planner),
        architect: preset_config_from_payload(payload.architect),
        director: preset_config_from_payload(payload.director),
        actor: preset_config_from_payload(payload.actor),
        narrator: preset_config_from_payload(payload.narrator),
        keeper: preset_config_from_payload(payload.keeper),
        replyer: preset_config_from_payload(payload.replyer),
    }
}

fn preset_config_from_payload(payload: protocol::AgentPresetConfigPayload) -> AgentPresetConfig {
    AgentPresetConfig {
        temperature: payload.temperature,
        max_tokens: payload.max_tokens,
        extra: payload.extra,
        prompt_entries: payload
            .prompt_entries
            .into_iter()
            .map(prompt_entry_config_from_payload)
            .collect(),
    }
}

fn preset_payload_from_record(record: &PresetRecord) -> PresetPayload {
    PresetPayload {
        preset_id: record.preset_id.clone(),
        display_name: record.display_name.clone(),
        agents: protocol::PresetAgentPayloads {
            planner: preset_payload_from_config(&record.agents.planner),
            architect: preset_payload_from_config(&record.agents.architect),
            director: preset_payload_from_config(&record.agents.director),
            actor: preset_payload_from_config(&record.agents.actor),
            narrator: preset_payload_from_config(&record.agents.narrator),
            keeper: preset_payload_from_config(&record.agents.keeper),
            replyer: preset_payload_from_config(&record.agents.replyer),
        },
    }
}

fn preset_payload_from_config(config: &AgentPresetConfig) -> protocol::AgentPresetConfigPayload {
    protocol::AgentPresetConfigPayload {
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        extra: config.extra.clone(),
        prompt_entries: config
            .prompt_entries
            .iter()
            .map(prompt_entry_payload_from_config)
            .collect(),
    }
}

fn prompt_entry_config_from_payload(
    payload: protocol::PresetPromptEntryPayload,
) -> AgentPromptEntryConfig {
    AgentPromptEntryConfig {
        entry_id: payload.entry_id,
        title: payload.title,
        content: payload.content,
        enabled: payload.enabled,
    }
}

fn prompt_entry_payload_from_config(
    config: &AgentPromptEntryConfig,
) -> protocol::PresetPromptEntryPayload {
    protocol::PresetPromptEntryPayload {
        entry_id: config.entry_id.clone(),
        title: config.title.clone(),
        content: config.content.clone(),
        enabled: config.enabled,
    }
}

fn preset_summary_payload_from_record(record: &PresetRecord) -> PresetSummaryPayload {
    PresetSummaryPayload {
        preset_id: record.preset_id.clone(),
        display_name: record.display_name.clone(),
        agents: protocol::PresetAgentSummaryPayloads {
            planner: preset_summary_payload_from_config(&record.agents.planner),
            architect: preset_summary_payload_from_config(&record.agents.architect),
            director: preset_summary_payload_from_config(&record.agents.director),
            actor: preset_summary_payload_from_config(&record.agents.actor),
            narrator: preset_summary_payload_from_config(&record.agents.narrator),
            keeper: preset_summary_payload_from_config(&record.agents.keeper),
            replyer: preset_summary_payload_from_config(&record.agents.replyer),
        },
    }
}

fn preset_summary_payload_from_config(
    config: &AgentPresetConfig,
) -> protocol::AgentPresetConfigSummaryPayload {
    protocol::AgentPresetConfigSummaryPayload {
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        extra: config.extra.clone(),
        prompt_entry_count: config.prompt_entries.len(),
        prompt_entries: config
            .prompt_entries
            .iter()
            .map(|entry| protocol::PresetPromptEntrySummaryPayload {
                entry_id: entry.entry_id.clone(),
                title: entry.title.clone(),
                enabled: entry.enabled,
            })
            .collect(),
    }
}
