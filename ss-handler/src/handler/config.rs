use protocol::{
    JsonRpcResponseMessage, ResponseResult, SessionConfigPayload, SessionUpdateConfigParams,
};

use crate::error::HandlerError;

use super::{Handler, require_session_id};

impl Handler {
    pub(crate) async fn handle_config_get_global(
        &self,
        request_id: &str,
    ) -> Result<JsonRpcResponseMessage, HandlerError> {
        let binding = self.manager.get_global_config().await?;
        Ok(JsonRpcResponseMessage::ok(
            request_id,
            None::<String>,
            ResponseResult::GlobalConfig(protocol::GlobalConfigPayload {
                api_group_id: binding.as_ref().map(|binding| binding.api_group_id.clone()),
                preset_id: binding.as_ref().map(|binding| binding.preset_id.clone()),
            }),
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
            .update_session_config(&session_id, params.api_group_id, params.preset_id)
            .await?;

        Ok(JsonRpcResponseMessage::ok(
            request_id,
            Some(session_id),
            ResponseResult::SessionConfig(build_session_config_payload(resolved)),
        ))
    }
}

pub(crate) fn build_session_config_payload(
    resolved: engine::ResolvedSessionConfig,
) -> SessionConfigPayload {
    SessionConfigPayload {
        api_group_id: resolved.binding.api_group_id,
        preset_id: resolved.binding.preset_id,
    }
}
