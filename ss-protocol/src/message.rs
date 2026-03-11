use serde::{Deserialize, Serialize};

use crate::error::ErrorPayload;
use crate::request::RequestBody;
use crate::response::ResponseBody;
use crate::stream_event::StreamEventBody;

pub type RequestId = String;
pub type SessionId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMessage {
    pub request_id: RequestId,
    pub session_id: Option<SessionId>,
    pub body: RequestBody,
}

impl RequestMessage {
    pub fn new(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        body: RequestBody,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            body,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMessage {
    pub request_id: RequestId,
    pub session_id: Option<SessionId>,
    pub outcome: ResponseOutcome,
}

impl ResponseMessage {
    pub fn ok(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        body: ResponseBody,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            outcome: ResponseOutcome::Ok {
                body: Box::new(body),
            },
        }
    }

    pub fn err(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        error: ErrorPayload,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            session_id: session_id.map(Into::into),
            outcome: ResponseOutcome::Err { error },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ResponseOutcome {
    Ok { body: Box<ResponseBody> },
    Err { error: ErrorPayload },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResponseMessage {
    pub request_id: RequestId,
    pub session_id: Option<SessionId>,
    pub sequence: u64,
    pub frame: StreamFrame,
}

impl StreamResponseMessage {
    pub fn started(
        request_id: impl Into<RequestId>,
        session_id: Option<impl Into<SessionId>>,
        sequence: u64,
    ) -> Self {
        Self {
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
        response: ResponseBody,
    ) -> Self {
        Self {
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
    Completed { response: Box<ResponseBody> },
    Failed { error: ErrorPayload },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type", rename_all = "snake_case")]
pub enum ServerMessage {
    Response { message: ResponseMessage },
    Stream { message: StreamResponseMessage },
}
