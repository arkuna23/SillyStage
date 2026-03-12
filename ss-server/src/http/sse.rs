use std::convert::Infallible;

use async_stream::stream;
use axum::response::Sse;
use axum::response::sse::Event;
use futures_core::Stream;
use futures_util::StreamExt;
use handler::HandlerEventStream;
use protocol::JsonRpcResponseMessage;
use serde::Serialize;

pub fn stream_response(
    ack: JsonRpcResponseMessage,
    events: HandlerEventStream,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        yield Ok(sse_event("ack", &ack));

        let mut events = events;
        while let Some(message) = events.next().await {
            yield Ok(sse_event("message", &message));
        }
    };

    Sse::new(stream)
}

fn sse_event<T: Serialize>(event: &str, payload: &T) -> Event {
    let json = serde_json::to_string(payload).expect("sse payload should serialize");
    Event::default().event(event).data(json)
}
