use engine::ResolvedSessionConfig;
use protocol::{
    ConfigUpdateGlobalParams, JsonRpcResponseMessage, ResponseResult, SessionConfigPayload,
    SessionUpdateConfigParams,
};

use crate::error::HandlerError;

use super::{Handler, require_session_id};

impl Handler {
    pub(crate) async fn handle_config_get_global(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let api_ids = self.manager.get_global_config().await?;
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
        let updated = self
            .manager
            .update_global_config(params.api_overrides)
            .await?;

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
        let payload = build_session_config_payload(
            self.manager
                .get_resolved_session_config(&session_id)
                .await?,
        );

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
        let resolved = self
            .manager
            .update_session_config(
                &session_id,
                params.mode,
                params.session_api_ids,
                params.api_overrides,
            )
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionConfig(build_session_config_payload(resolved)),
        ))
    }
}

pub(crate) fn build_session_config_payload(
    resolved: ResolvedSessionConfig,
) -> SessionConfigPayload {
    SessionConfigPayload {
        mode: resolved.config.mode,
        session_api_ids: resolved.config.session_api_ids,
        effective_api_ids: resolved.effective_api_ids,
    }
}
