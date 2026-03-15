use protocol::{
    ApiGroupBindingsInput, ApiGroupBindingsPayload, ApiGroupCreateParams, ApiGroupDeleteParams,
    ApiGroupDeletedPayload, ApiGroupGetParams, ApiGroupPayload, ApiGroupUpdateParams,
    ApiGroupsListedPayload, JsonRpcResponseMessage, ResponseResult,
};
use store::{ApiGroupAgentBindings, ApiGroupRecord};

use crate::error::HandlerError;

use super::Handler;

impl Handler {
    pub(crate) async fn handle_api_group_create(
        &self,
        request_id: &str,
        params: ApiGroupCreateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_group_id = normalize_api_group_id(&params.api_group_id)?;
        if self.store.get_api_group(&api_group_id).await?.is_some() {
            return Err(HandlerError::DuplicateApiGroup(api_group_id));
        }

        let bindings = api_group_bindings_from_input(params.bindings)?;
        ensure_bound_apis_exist(self, &bindings).await?;

        let record = ApiGroupRecord {
            api_group_id: api_group_id.clone(),
            display_name: params.display_name,
            agents: bindings,
        };
        self.store.save_api_group(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApiGroup(Box::new(api_group_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_api_group_get(
        &self,
        request_id: &str,
        params: ApiGroupGetParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_group_id = normalize_api_group_id(&params.api_group_id)?;
        let record = self
            .store
            .get_api_group(&api_group_id)
            .await?
            .ok_or_else(|| HandlerError::MissingApiGroup(api_group_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApiGroup(Box::new(api_group_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_api_group_list(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let mut api_groups = self
            .store
            .list_api_groups()
            .await?
            .into_iter()
            .map(|record| api_group_payload_from_record(&record))
            .collect::<Vec<_>>();
        api_groups.sort_by(|left, right| left.api_group_id.cmp(&right.api_group_id));

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApiGroupsListed(ApiGroupsListedPayload { api_groups }),
        ))
    }

    pub(crate) async fn handle_api_group_update(
        &self,
        request_id: &str,
        params: ApiGroupUpdateParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_group_id = normalize_api_group_id(&params.api_group_id)?;
        let mut record = self
            .store
            .get_api_group(&api_group_id)
            .await?
            .ok_or_else(|| HandlerError::MissingApiGroup(api_group_id.clone()))?;

        if let Some(display_name) = params.display_name {
            record.display_name = display_name;
        }
        if let Some(bindings) = params.bindings {
            let bindings = api_group_bindings_from_input(bindings)?;
            ensure_bound_apis_exist(self, &bindings).await?;
            record.agents = bindings;
        }
        self.store.save_api_group(record.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApiGroup(Box::new(api_group_payload_from_record(&record))),
        ))
    }

    pub(crate) async fn handle_api_group_delete(
        &self,
        request_id: &str,
        params: ApiGroupDeleteParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_group_id = normalize_api_group_id(&params.api_group_id)?;
        ensure_api_group_not_in_use(self, &api_group_id).await?;
        self.store
            .delete_api_group(&api_group_id)
            .await?
            .ok_or_else(|| HandlerError::MissingApiGroup(api_group_id.clone()))?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::ApiGroupDeleted(ApiGroupDeletedPayload { api_group_id }),
        ))
    }
}

async fn ensure_api_group_not_in_use(
    handler: &Handler,
    api_group_id: &str,
) -> Result<(), HandlerError> {
    if handler
        .store
        .list_story_drafts()
        .await?
        .into_iter()
        .any(|draft| draft.api_group_id == api_group_id)
    {
        return Err(HandlerError::ApiGroupInUse(api_group_id.to_owned()));
    }

    if handler
        .store
        .list_sessions()
        .await?
        .into_iter()
        .any(|session| session.binding.api_group_id == api_group_id)
    {
        return Err(HandlerError::ApiGroupInUse(api_group_id.to_owned()));
    }

    Ok(())
}

async fn ensure_bound_apis_exist(
    handler: &Handler,
    bindings: &ApiGroupAgentBindings,
) -> Result<(), HandlerError> {
    for api_id in [
        &bindings.planner_api_id,
        &bindings.architect_api_id,
        &bindings.director_api_id,
        &bindings.actor_api_id,
        &bindings.narrator_api_id,
        &bindings.keeper_api_id,
        &bindings.replyer_api_id,
    ] {
        if handler.store.get_api(api_id).await?.is_none() {
            return Err(HandlerError::MissingApi(api_id.clone()));
        }
    }
    Ok(())
}

fn normalize_api_group_id(api_group_id: &str) -> Result<String, HandlerError> {
    let trimmed = api_group_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyApiGroupId);
    }
    Ok(trimmed.to_owned())
}

fn api_group_bindings_from_input(
    input: ApiGroupBindingsInput,
) -> Result<ApiGroupAgentBindings, HandlerError> {
    Ok(ApiGroupAgentBindings {
        planner_api_id: normalize_api_id(&input.planner_api_id)?,
        architect_api_id: normalize_api_id(&input.architect_api_id)?,
        director_api_id: normalize_api_id(&input.director_api_id)?,
        actor_api_id: normalize_api_id(&input.actor_api_id)?,
        narrator_api_id: normalize_api_id(&input.narrator_api_id)?,
        keeper_api_id: normalize_api_id(&input.keeper_api_id)?,
        replyer_api_id: normalize_api_id(&input.replyer_api_id)?,
    })
}

fn api_group_payload_from_record(record: &ApiGroupRecord) -> ApiGroupPayload {
    ApiGroupPayload {
        api_group_id: record.api_group_id.clone(),
        display_name: record.display_name.clone(),
        bindings: ApiGroupBindingsPayload {
            planner_api_id: record.agents.planner_api_id.clone(),
            architect_api_id: record.agents.architect_api_id.clone(),
            director_api_id: record.agents.director_api_id.clone(),
            actor_api_id: record.agents.actor_api_id.clone(),
            narrator_api_id: record.agents.narrator_api_id.clone(),
            keeper_api_id: record.agents.keeper_api_id.clone(),
            replyer_api_id: record.agents.replyer_api_id.clone(),
        },
    }
}

fn normalize_api_id(api_id: &str) -> Result<String, HandlerError> {
    let trimmed = api_id.trim();
    if trimmed.is_empty() {
        return Err(HandlerError::EmptyApiId);
    }
    Ok(trimmed.to_owned())
}
