use protocol::{
    ApiCreateParams, ApiDeleteParams, ApiDeletedPayload, ApiGetParams, ApiPayload, ApiUpdateParams,
    ApisListedPayload, JsonRpcResponseMessage, ResponseResult,
};
use store::{ApiRecord, Store};

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_api_create(
        &self,
        request_id: &str,
        params: ApiCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        if self.store.get_api(&api_id).await?.is_some() {
            return Err(HandlerError::DuplicateApi(api_id));
        }

        let record = ApiRecord {
            api_id: api_id.clone(),
            display_name: params.display_name,
            provider: params.provider,
            base_url: params.base_url,
            api_key: params.api_key,
            model: params.model,
        };
        self.store.save_api(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Api(Box::new(api_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_api_get(
        &self,
        request_id: &str,
        params: ApiGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        let record = self
            .store
            .get_api(&api_id)
            .await?
            .ok_or_else(|| HandlerError::MissingApi(api_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Api(Box::new(api_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_api_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut apis = self
            .store
            .list_apis()
            .await?
            .into_iter()
            .map(|record| api_payload_from_record(&record))
            .collect::<Vec<_>>();
        apis.sort_by(|left, right| left.api_id.cmp(&right.api_id));

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApisListed(ApisListedPayload { apis }),
        ))
    }

    pub(crate) async fn handle_api_update(
        &self,
        request_id: &str,
        params: ApiUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        let mut record = self
            .store
            .get_api(&api_id)
            .await?
            .ok_or_else(|| HandlerError::MissingApi(api_id.clone()))?;

        if let Some(display_name) = params.display_name {
            record.display_name = display_name;
        }
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
        self.store.save_api(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::Api(Box::new(api_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_api_delete(
        &self,
        request_id: &str,
        params: ApiDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_id = normalize_api_id(&params.api_id)?;
        ensure_api_not_in_use(self.store.as_ref(), &api_id).await?;
        self.store
            .delete_api(&api_id)
            .await?
            .ok_or_else(|| HandlerError::MissingApi(api_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApiDeleted(ApiDeletedPayload { api_id }),
        ))
    }
}

async fn ensure_api_not_in_use(store: &dyn Store, api_id: &str) -> Result<(), HandlerError> {
    if store.list_api_groups().await?.into_iter().any(|group| {
        let bindings = group.agents;
        [
            bindings.planner_api_id,
            bindings.architect_api_id,
            bindings.director_api_id,
            bindings.actor_api_id,
            bindings.narrator_api_id,
            bindings.keeper_api_id,
            bindings.replyer_api_id,
        ]
        .into_iter()
        .any(|bound_api_id| bound_api_id == api_id)
    }) {
        return Err(HandlerError::ApiInUse(api_id.to_owned()));
    }

    Ok(())
}

fn normalize_api_id(api_id: &str) -> Result<String, HandlerError> {
    let trimmed = api_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyApiId);
    }
    Ok(trimmed.to_owned())
}

pub(crate) fn api_payload_from_record(record: &ApiRecord) -> ApiPayload {
    ApiPayload {
        api_id: record.api_id.clone(),
        display_name: record.display_name.clone(),
        provider: record.provider,
        base_url: record.base_url.clone(),
        model: record.model.clone(),
        has_api_key: !record.api_key.trim().is_empty(),
        api_key_masked: mask_api_key(&record.api_key),
    }
}

fn mask_api_key(api_key: &str) -> Option<String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return None;
    }

    let chars: Vec<char> = trimmed.chars().collect();
    let len = chars.len();
    if len <= 8 {
        return Some("*".repeat(len));
    }

    let prefix: String = chars.iter().take(4).collect();
    let suffix: String = chars.iter().skip(len - 4).collect();
    Some(format!("{prefix}...{suffix}"))
}
