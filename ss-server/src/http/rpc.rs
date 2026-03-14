use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use handler::HandlerReply;
use protocol::{ErrorCode, ErrorPayload, JsonRpcRequestMessage, JsonRpcResponseMessage};
use serde_json::Value;
use tracing::warn;

use crate::ServerState;

use super::logging::{json_for_log, log_rpc_request, log_rpc_response};
use super::sse::stream_response;

pub async fn handle_rpc(State(state): State<ServerState>, body: Bytes) -> Response {
    let request = match parse_request(&body) {
        Ok(request) => request,
        Err(error) => return error.into_response(),
    };

    let method = request.method();
    log_rpc_request(&request);

    match state.handler().handle(request).await {
        HandlerReply::Unary(response) => {
            log_rpc_response(method, &response);
            (StatusCode::OK, Json(response)).into_response()
        }
        HandlerReply::Stream { ack, events } => {
            log_rpc_response(method, &ack);
            stream_response(ack, events).into_response()
        }
    }
}

enum RequestParseError {
    Plain(ErrorPayload),
    JsonRpc(JsonRpcResponseMessage),
}

impl IntoResponse for RequestParseError {
    fn into_response(self) -> Response {
        match self {
            Self::Plain(error) => (StatusCode::BAD_REQUEST, Json(error)).into_response(),
            Self::JsonRpc(response) => (StatusCode::BAD_REQUEST, Json(response)).into_response(),
        }
    }
}

fn parse_request(body: &[u8]) -> Result<JsonRpcRequestMessage, RequestParseError> {
    let value: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(error) => {
            warn!(
                body_len = body.len(),
                error = %error,
                "failed to parse rpc request body as json"
            );
            return Err(RequestParseError::Plain(ErrorPayload::new(
                ErrorCode::ParseError,
                error.to_string(),
            )));
        }
    };

    match serde_json::from_value::<JsonRpcRequestMessage>(value.clone()) {
        Ok(request) => Ok(request),
        Err(error) => {
            warn!(
                body_len = body.len(),
                error = %error,
                payload = %json_for_log(&value),
                "failed to decode rpc request payload"
            );
            let request_id = value
                .get("id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let session_id = value
                .get("session_id")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned);
            let error = ErrorPayload::new(ErrorCode::InvalidRequest, error.to_string());

            match request_id {
                Some(request_id) => Err(RequestParseError::JsonRpc(JsonRpcResponseMessage::err(
                    request_id, session_id, error,
                ))),
                None => Err(RequestParseError::Plain(error)),
            }
        }
    }
}
