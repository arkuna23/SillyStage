use protocol::{
    JsonRpcRequestMessage, JsonRpcResponseMessage, RequestMethod, ServerEventMessage, StreamFrame,
};
use serde::Serialize;
use serde_json::{Value, json};
use tracing::{debug, info};

const REDACTED_STRING_KEYS: &[&str] = &["api_key"];
const OMITTED_BASE64_KEYS: &[&str] = &["payload_base64", "cover_base64", "chr_base64"];

pub(crate) fn json_for_log<T: Serialize>(payload: &T) -> String {
    let mut value = match serde_json::to_value(payload) {
        Ok(value) => value,
        Err(error) => return format!("{{\"serialization_error\":\"{error}\"}}"),
    };
    redact_value(&mut value, None);
    serde_json::to_string(&value)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"))
}

pub(crate) fn log_rpc_request(request: &JsonRpcRequestMessage) {
    let level = log_level_for_method(request.method());
    let payload = json_for_log(request);

    match level {
        LogLevel::Info => info!(
            request_id = %request.id,
            session_id = ?request.session_id,
            method = ?request.method(),
            payload = %payload,
            "received rpc request"
        ),
        LogLevel::Debug => debug!(
            request_id = %request.id,
            session_id = ?request.session_id,
            method = ?request.method(),
            payload = %payload,
            "received rpc request"
        ),
    }
}

pub(crate) fn log_rpc_response(method: RequestMethod, response: &JsonRpcResponseMessage) {
    let level = log_level_for_method(method);
    let payload = json_for_log(response);

    match level {
        LogLevel::Info => info!(
            request_id = %response.id,
            session_id = ?response.session_id,
            method = ?method,
            payload = %payload,
            "sending rpc response"
        ),
        LogLevel::Debug => debug!(
            request_id = %response.id,
            session_id = ?response.session_id,
            method = ?method,
            payload = %payload,
            "sending rpc response"
        ),
    }
}

pub(crate) fn log_stream_event(message: &ServerEventMessage) {
    let payload = json_for_log(message);

    match message.frame {
        StreamFrame::Started | StreamFrame::Event { .. } => debug!(
            request_id = %message.request_id,
            session_id = ?message.session_id,
            sequence = message.sequence,
            payload = %payload,
            "streaming rpc event"
        ),
        StreamFrame::Completed { .. } | StreamFrame::Failed { .. } => info!(
            request_id = %message.request_id,
            session_id = ?message.session_id,
            sequence = message.sequence,
            payload = %payload,
            "streaming rpc event"
        ),
    }
}

#[derive(Clone, Copy)]
enum LogLevel {
    Info,
    Debug,
}

fn log_level_for_method(method: RequestMethod) -> LogLevel {
    if is_frequent_method(method) {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

fn is_frequent_method(method: RequestMethod) -> bool {
    matches!(
        method,
        RequestMethod::UploadInit
            | RequestMethod::UploadChunk
            | RequestMethod::ApiGroupGet
            | RequestMethod::ApiGroupList
            | RequestMethod::PresetGet
            | RequestMethod::PresetList
            | RequestMethod::SchemaGet
            | RequestMethod::SchemaList
            | RequestMethod::PlayerProfileGet
            | RequestMethod::PlayerProfileList
            | RequestMethod::CharacterGet
            | RequestMethod::CharacterGetCover
            | RequestMethod::CharacterList
            | RequestMethod::StoryResourcesGet
            | RequestMethod::StoryResourcesList
            | RequestMethod::StoryGet
            | RequestMethod::StoryList
            | RequestMethod::SessionGet
            | RequestMethod::SessionList
            | RequestMethod::SessionSuggestReplies
            | RequestMethod::SessionGetRuntimeSnapshot
            | RequestMethod::ConfigGetGlobal
            | RequestMethod::SessionGetConfig
            | RequestMethod::DashboardGet
    )
}

fn redact_value(value: &mut Value, key: Option<&str>) {
    match value {
        Value::Object(object) => {
            for (entry_key, entry_value) in object {
                redact_value(entry_value, Some(entry_key.as_str()));
            }
        }
        Value::Array(array) => {
            for entry in array {
                redact_value(entry, key);
            }
        }
        Value::String(content) => {
            if let Some(key) = key {
                if REDACTED_STRING_KEYS.contains(&key) {
                    *value = Value::String("<redacted>".to_owned());
                    return;
                }

                if OMITTED_BASE64_KEYS.contains(&key) {
                    *value = json!({
                        "omitted": true,
                        "kind": "base64",
                        "length": content.len(),
                    });
                }
            }
        }
        _ => {}
    }
}
