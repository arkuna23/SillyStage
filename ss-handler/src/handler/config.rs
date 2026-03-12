use engine::{AgentApiIds, LlmApiRegistry, SessionConfigMode, SessionEngineConfig};
use protocol::{
    ConfigUpdateGlobalParams, JsonRpcResponseMessage, ResponseResult, SessionConfigPayload,
    SessionUpdateConfigParams,
};

use crate::error::HandlerError;

use super::{Handler, require_session_id};

impl<'a> Handler<'a> {
    pub(crate) async fn handle_config_get_global(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_ids = self.load_global_config().await?;
        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::GlobalConfig(protocol::GlobalConfigPayload { api_ids }),
        ))
    }

    pub(crate) async fn handle_config_update_global(
        &self,
        request_id: &str,
        params: ConfigUpdateGlobalParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let current = self.load_global_config().await?;
        let updated = current.apply_overrides(&params.api_overrides);
        validate_api_ids(&self.registry, &updated)?;
        self.store.set_global_config(updated.clone()).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::GlobalConfig(protocol::GlobalConfigPayload { api_ids: updated }),
        ))
    }

    pub(crate) async fn handle_session_get_config(
        &self,
        request_id: &str,
        session_id: Option<String>,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let global = self.load_global_config().await?;
        let payload = build_session_config_payload(&session.config, &global);

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionConfig(payload),
        ))
    }

    pub(crate) async fn handle_session_update_config(
        &self,
        request_id: &str,
        session_id: Option<String>,
        params: SessionUpdateConfigParams,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let session_id = require_session_id(session_id)?;
        let mut session = self
            .store
            .get_session(&session_id)
            .await?
            .ok_or_else(|| HandlerError::MissingSession(session_id.clone()))?;
        let global = self.load_global_config().await?;
        let new_config = match params.mode {
            SessionConfigMode::UseGlobal => SessionEngineConfig::use_global(),
            SessionConfigMode::UseSession => {
                let base_api_ids = params.session_api_ids.unwrap_or_else(|| {
                    session
                        .config
                        .session_api_ids
                        .clone()
                        .unwrap_or_else(|| effective_session_api_ids(&session.config, &global))
                });
                let merged = params.api_overrides.unwrap_or_default();
                SessionEngineConfig::use_session(base_api_ids.apply_overrides(&merged))
            }
        };
        let effective = effective_session_api_ids(&new_config, &global);
        validate_api_ids(&self.registry, &effective)?;

        session.config = new_config.clone();
        self.store.save_session(session).await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionConfig(build_session_config_payload(&new_config, &global)),
        ))
    }

    pub(crate) async fn load_global_config(&self) -> Result<AgentApiIds, HandlerError> {
        self.store
            .get_global_config()
            .await?
            .ok_or(HandlerError::MissingGlobalConfig)
    }
}

pub(crate) fn validate_api_ids(
    registry: &LlmApiRegistry<'_>,
    api_ids: &AgentApiIds,
) -> Result<(), HandlerError> {
    registry.build_story_generation_configs(api_ids)?;
    registry.build_runtime_configs(api_ids)?;
    Ok(())
}

pub(crate) fn effective_session_api_ids(
    config: &SessionEngineConfig,
    global: &AgentApiIds,
) -> AgentApiIds {
    match config.mode {
        SessionConfigMode::UseGlobal => global.clone(),
        SessionConfigMode::UseSession => config
            .session_api_ids
            .clone()
            .unwrap_or_else(|| global.clone()),
    }
}

pub(crate) fn build_session_config_payload(
    config: &SessionEngineConfig,
    global: &AgentApiIds,
) -> SessionConfigPayload {
    SessionConfigPayload {
        mode: config.mode,
        session_api_ids: config.session_api_ids.clone(),
        effective_api_ids: effective_session_api_ids(config, global),
    }
}
