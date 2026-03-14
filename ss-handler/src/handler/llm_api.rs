use protocol::{
    DefaultLlmConfigPayload, DefaultLlmConfigStatePayload, DefaultLlmConfigUpdateParams,
    JsonRpcResponseMessage, LlmApiCreateParams, LlmApiDeleteParams, LlmApiDeletedPayload,
    LlmApiGetParams, LlmApiPayload, LlmApiUpdateParams, LlmApisListedPayload, ResponseResult,
};
use store::{AgentApiIds, DefaultLlmConfigRecord, LlmApiRecord, SessionConfigMode};

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_llm_api_create(
        &self,
        request_id: &str,
        params: LlmApiCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;

        if self.store.get_llm_api(&api_id).await?.is_some() {
            return Err(HandlerError::DuplicateLlmApi(api_id));
        }

        let default_config = self.resolve_effective_default_llm_config().await?;
        let record = build_llm_api_record(api_id, params, default_config)?;
        self.manager.upsert_llm_api_record(&record)?;
        self.store.save_llm_api(record.clone()).await?;
        let _ = self
            .manager
            .initialize_global_config_if_missing(&record.api_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LlmApi(llm_api_payload_from_record(&record)),
        ))
    }

    pub(crate) async fn handle_llm_api_get(
        &self,
        request_id: &str,
        params: LlmApiGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        let record = self
            .store
            .get_llm_api(&api_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLlmApi(api_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LlmApi(llm_api_payload_from_record(&record)),
        ))
    }

    pub(crate) async fn handle_llm_api_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut apis = self
            .store
            .list_llm_apis()
            .await?
            .into_iter()
            .map(|record| llm_api_payload_from_record(&record))
            .collect::<Vec<_>>();
        apis.sort_by(|left, right| left.api_id.cmp(&right.api_id));

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LlmApisListed(LlmApisListedPayload { apis }),
        ))
    }

    pub(crate) async fn handle_llm_api_update(
        &self,
        request_id: &str,
        params: LlmApiUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        let mut record = self
            .store
            .get_llm_api(&api_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLlmApi(api_id.clone()))?;

        if let Some(provider) = params.provider {
            record.provider = provider;
        }
        if let Some(base_url) = params.base_url {
            record.base_url = base_url;
        }
        if let Some(api_key) = params.api_key {
            record.api_key = api_key;
        }
        if let Some(model) = params.model {
            record.model = model;
        }
        if let Some(temperature) = params.temperature {
            record.temperature = Some(temperature);
        }
        if let Some(max_tokens) = params.max_tokens {
            record.max_tokens = Some(max_tokens);
        }

        self.manager.upsert_llm_api_record(&record)?;
        self.store.save_llm_api(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LlmApi(llm_api_payload_from_record(&record)),
        ))
    }

    pub(crate) async fn handle_llm_api_delete(
        &self,
        request_id: &str,
        params: LlmApiDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        let record = self
            .store
            .get_llm_api(&api_id)
            .await?
            .ok_or_else(|| HandlerError::MissingLlmApi(api_id.clone()))?;

        ensure_llm_api_not_in_use(self, &record.api_id).await?;

        self.store.delete_llm_api(&record.api_id).await?;
        self.manager.remove_llm_api_record(&record.api_id);

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::LlmApiDeleted(LlmApiDeletedPayload {
                api_id: record.api_id,
            }),
        ))
    }

    pub(crate) async fn handle_default_llm_config_get(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let saved = self.store.get_default_llm_config().await?;
        let effective = self.resolve_effective_default_llm_config().await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::DefaultLlmConfig(DefaultLlmConfigStatePayload {
                saved: saved.as_ref().map(default_llm_config_payload_from_record),
                effective: effective
                    .as_ref()
                    .map(default_llm_config_payload_from_record),
            }),
        ))
    }

    pub(crate) async fn handle_default_llm_config_update(
        &self,
        request_id: &str,
        params: DefaultLlmConfigUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let saved = DefaultLlmConfigRecord {
            provider: params.provider,
            base_url: params.base_url,
            api_key: params.api_key,
            model: params.model,
            temperature: params.temperature,
            max_tokens: params.max_tokens,
        };
        self.store.set_default_llm_config(saved.clone()).await?;
        let effective = self.resolve_effective_default_llm_config().await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::DefaultLlmConfig(DefaultLlmConfigStatePayload {
                saved: Some(default_llm_config_payload_from_record(&saved)),
                effective: effective
                    .as_ref()
                    .map(default_llm_config_payload_from_record),
            }),
        ))
    }
}

async fn ensure_llm_api_not_in_use(handler: &Handler, api_id: &str) -> Result<(), HandlerError> {
    let global = handler.manager.get_global_config().await?;
    if global
        .as_ref()
        .is_some_and(|global| agent_api_ids_contains(global, api_id))
    {
        return Err(HandlerError::LlmApiInUse(api_id.to_owned()));
    }

    for session in handler.store.list_sessions().await? {
        if session.config.mode != SessionConfigMode::UseSession {
            continue;
        }

        if let Some(session_api_ids) = session.config.session_api_ids.as_ref()
            && agent_api_ids_contains(session_api_ids, api_id)
        {
            return Err(HandlerError::LlmApiInUse(api_id.to_owned()));
        }
    }

    Ok(())
}

fn agent_api_ids_contains(api_ids: &AgentApiIds, api_id: &str) -> bool {
    api_ids.planner_api_id == api_id
        || api_ids.architect_api_id == api_id
        || api_ids.director_api_id == api_id
        || api_ids.actor_api_id == api_id
        || api_ids.narrator_api_id == api_id
        || api_ids.keeper_api_id == api_id
        || api_ids.replyer_api_id == api_id
}

fn llm_api_payload_from_record(record: &LlmApiRecord) -> LlmApiPayload {
    LlmApiPayload {
        api_id: record.api_id.clone(),
        provider: record.provider,
        base_url: record.base_url.clone(),
        model: record.model.clone(),
        temperature: record.temperature,
        max_tokens: record.max_tokens,
        has_api_key: !record.api_key.is_empty(),
        api_key_masked: mask_api_key(&record.api_key),
    }
}

fn default_llm_config_payload_from_record(
    record: &DefaultLlmConfigRecord,
) -> DefaultLlmConfigPayload {
    DefaultLlmConfigPayload {
        provider: record.provider,
        base_url: record.base_url.clone(),
        model: record.model.clone(),
        temperature: record.temperature,
        max_tokens: record.max_tokens,
        has_api_key: !record.api_key.is_empty(),
        api_key_masked: mask_api_key(&record.api_key),
    }
}

fn mask_api_key(api_key: &str) -> Option<String> {
    if api_key.is_empty() {
        return None;
    }

    let chars = api_key.chars().collect::<Vec<_>>();
    if chars.len() <= 4 {
        return Some("****".to_owned());
    }

    let prefix = chars.iter().take(2).collect::<String>();
    let suffix = chars
        .iter()
        .rev()
        .take(2)
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    Some(format!("{prefix}****{suffix}"))
}

fn normalize_api_id(api_id: &str) -> Result<String, HandlerError> {
    let trimmed = api_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyLlmApiId);
    }
    Ok(trimmed.to_owned())
}

fn build_llm_api_record(
    api_id: String,
    params: LlmApiCreateParams,
    default_config: Option<DefaultLlmConfigRecord>,
) -> Result<LlmApiRecord, HandlerError> {
    let provider = params
        .provider
        .or(default_config.as_ref().map(|config| config.provider));
    let base_url = params.base_url.or_else(|| {
        default_config
            .as_ref()
            .map(|config| config.base_url.clone())
    });
    let api_key = params
        .api_key
        .or_else(|| default_config.as_ref().map(|config| config.api_key.clone()));
    let model = params
        .model
        .or_else(|| default_config.as_ref().map(|config| config.model.clone()));
    let temperature = params.temperature.or_else(|| {
        default_config
            .as_ref()
            .and_then(|config| config.temperature)
    });
    let max_tokens = params
        .max_tokens
        .or_else(|| default_config.as_ref().and_then(|config| config.max_tokens));

    let mut missing = Vec::new();
    if provider.is_none() {
        missing.push("provider");
    }
    if base_url.is_none() {
        missing.push("base_url");
    }
    if api_key.is_none() {
        missing.push("api_key");
    }
    if model.is_none() {
        missing.push("model");
    }

    if !missing.is_empty() {
        return Err(HandlerError::IncompleteLlmApiCreate(missing.join(", ")));
    }

    Ok(LlmApiRecord {
        api_id,
        provider: provider.expect("checked above"),
        base_url: base_url.expect("checked above"),
        api_key: api_key.expect("checked above"),
        model: model.expect("checked above"),
        temperature,
        max_tokens,
    })
}

impl Handler {
    async fn resolve_effective_default_llm_config(
        &self,
    ) -> Result<Option<DefaultLlmConfigRecord>, HandlerError> {
        Ok(self
            .effective_default_llm_config
            .clone()
            .or(self.store.get_default_llm_config().await?))
    }
}
