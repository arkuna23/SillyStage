use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::error::ErrorPayload;
use crate::request::{RequestMethod, RequestParams};
use crate::response::ResponseResult;
use crate::stream_event::StreamEventBody;

pub type RequestId = String;
pub type SessionId = String;

const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug, Clone)]
pub struct JsonRpcRequestMessage {
    pub id: RequestId,
    pub session_id: Option<SessionId>,
    pub params: RequestParams,
}

impl JsonRpcRequestMessage {
    pub fn new(
        id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        params: RequestParams,
    ) -> Self {
        Self {
            id: id.into(),
            session_id: session_id.map(Into::into),
            params,
        }
    }

    pub const fn method(&self) -> RequestMethod {
        self.params.method()
    }
}

impl Serialize for JsonRpcRequestMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("JsonRpcRequestMessage", 5)?;
        state.serialize_field("jsonrpc", JSONRPC_VERSION)?;
        state.serialize_field("id", &self.id)?;
        if let Some(session_id) = &self.session_id {
            state.serialize_field("session_id", session_id)?;
        }
        state.serialize_field("method", &self.method())?;
        state.serialize_field(
            "params",
            &self.params.to_value().map_err(serde::ser::Error::custom)?,
        )?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for JsonRpcRequestMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawJsonRpcRequestMessage {
            jsonrpc: String,
            id: RequestId,
            #[serde(default)]
            session_id: Option<SessionId>,
            method: RequestMethod,
            #[serde(default = "default_params_value")]
            params: Value,
        }

        let raw = RawJsonRpcRequestMessage::deserialize(deserializer)?;
        if raw.jsonrpc != JSONRPC_VERSION {
            return Err(de::Error::custom("jsonrpc must be \"2.0\""));
        }

        let params = RequestParams::from_method_and_value(raw.method, raw.params)
            .map_err(de::Error::custom)?;

        Ok(Self {
            id: raw.id,
            session_id: raw.session_id,
            params,
        })
    }
}

#[derive(Debug, Clone)]
pub struct JsonRpcResponseMessage {
    pub id: RequestId,
    pub session_id: Option<SessionId>,
    pub outcome: JsonRpcOutcome,
}

impl JsonRpcResponseMessage {
    pub fn ok(
        id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        result: ResponseResult,
    ) -> Self {
        Self {
            id: id.into(),
            session_id: session_id.map(Into::into),
            outcome: JsonRpcOutcome::Ok(Box::new(result)),
        }
    }

    pub fn err(
        id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        error: ErrorPayload,
    ) -> Self {
        Self {
            id: id.into(),
            session_id: session_id.map(Into::into),
            outcome: JsonRpcOutcome::Err(error),
        }
    }
}

#[derive(Debug, Clone)]
pub enum JsonRpcOutcome {
    Ok(Box<ResponseResult>),
    Err(ErrorPayload),
}

impl Serialize for JsonRpcResponseMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = if self.session_id.is_some() { 4 } else { 3 };
        let mut state = serializer.serialize_struct("JsonRpcResponseMessage", field_count)?;
        state.serialize_field("jsonrpc", JSONRPC_VERSION)?;
        state.serialize_field("id", &self.id)?;
        if let Some(session_id) = &self.session_id {
            state.serialize_field("session_id", session_id)?;
        }
        match &self.outcome {
            JsonRpcOutcome::Ok(result) => state.serialize_field("result", &**result)?,
            JsonRpcOutcome::Err(error) => state.serialize_field("error", error)?,
        }
        state.end()
    }
}

impl<'de> Deserialize<'de> for JsonRpcResponseMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Jsonrpc,
            Id,
            SessionId,
            Result,
            Error,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(
                        &self,
                        formatter: &mut std::fmt::Formatter<'_>,
                    ) -> std::fmt::Result {
                        formatter.write_str("a valid json-rpc response field")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "jsonrpc" => Ok(Field::Jsonrpc),
                            "id" => Ok(Field::Id),
                            "session_id" => Ok(Field::SessionId),
                            "result" => Ok(Field::Result),
                            "error" => Ok(Field::Error),
                            _ => Err(de::Error::unknown_field(
                                value,
                                &["jsonrpc", "id", "session_id", "result", "error"],
                            )),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ResponseVisitor;

        impl<'de> Visitor<'de> for ResponseVisitor {
            type Value = JsonRpcResponseMessage;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a json-rpc response message")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut jsonrpc: Option<String> = None;
                let mut id: Option<RequestId> = None;
                let mut session_id: Option<SessionId> = None;
                let mut result: Option<ResponseResult> = None;
                let mut error: Option<ErrorPayload> = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Jsonrpc => jsonrpc = Some(map.next_value()?),
                        Field::Id => id = Some(map.next_value()?),
                        Field::SessionId => session_id = Some(map.next_value()?),
                        Field::Result => result = Some(map.next_value()?),
                        Field::Error => error = Some(map.next_value()?),
                    }
                }

                if jsonrpc.as_deref() != Some(JSONRPC_VERSION) {
                    return Err(de::Error::custom("jsonrpc must be \"2.0\""));
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let outcome = match (result, error) {
                    (Some(result), None) => JsonRpcOutcome::Ok(Box::new(result)),
                    (None, Some(error)) => JsonRpcOutcome::Err(error),
                    (Some(_), Some(_)) => {
                        return Err(de::Error::custom(
                            "json-rpc response cannot contain both result and error",
                        ));
                    }
                    (None, None) => {
                        return Err(de::Error::custom(
                            "json-rpc response must contain either result or error",
                        ));
                    }
                };

                Ok(JsonRpcResponseMessage {
                    id,
                    session_id,
                    outcome,
                })
            }
        }

        deserializer.deserialize_struct(
            "JsonRpcResponseMessage",
            &["jsonrpc", "id", "session_id", "result", "error"],
            ResponseVisitor,
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerMessageType {
    Stream,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEventMessage {
    pub message_type: ServerMessageType,
    pub request_id: RequestId,
    pub session_id: Option<SessionId>,
    pub sequence: u64,
    pub frame: StreamFrame,
}

impl ServerEventMessage {
    pub fn started(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        sequence: u64,
    ) -> Self {
        Self {
            message_type: ServerMessageType::Stream,
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            sequence,
            frame: StreamFrame::Started,
        }
    }

    pub fn event(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        sequence: u64,
        body: StreamEventBody,
    ) -> Self {
        Self {
            message_type: ServerMessageType::Stream,
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            sequence,
            frame: StreamFrame::Event { body },
        }
    }

    pub fn completed(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        sequence: u64,
        response: ResponseResult,
    ) -> Self {
        Self {
            message_type: ServerMessageType::Stream,
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            sequence,
            frame: StreamFrame::Completed {
                response: Box::new(response),
            },
        }
    }

    pub fn failed(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        sequence: u64,
        error: ErrorPayload,
    ) -> Self {
        Self {
            message_type: ServerMessageType::Stream,
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            sequence,
            frame: StreamFrame::Failed { error },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamFrame {
    Started,
    Event { body: StreamEventBody },
    Completed { response: Box<ResponseResult> },
    Failed { error: ErrorPayload },
}

fn default_params_value() -> Value {
    Value::Object(serde_json::Map::new())
}
