# HTTP Transport Notes

This document describes the current HTTP transport exposed by `ss-server`. The protocol objects themselves still come from `ss-protocol`.

## Routes

- `POST /rpc`
  - The single transport entrypoint for all protocol requests.
- `GET /healthz`
  - Returns plain-text `ok`.
  - This is an operational helper endpoint, not a `ss-protocol` method.

## Request Body

`POST /rpc` expects a full `JsonRpcRequestMessage` JSON body.

Example:

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "method": "story.generate",
  "params": {
    "resource_id": "resource-0"
  }
}
```

## Unary Responses

Regular methods return:

- HTTP status: `200 OK`
- `Content-Type: application/json`
- body: a full `JsonRpcResponseMessage`

## Streaming Responses

The current streaming method is `session.run_turn`. It still uses `POST /rpc`, but the HTTP response becomes:

- HTTP status: `200 OK`
- `Content-Type: text/event-stream`

SSE event rules:

1. The first event name is always `ack`
   - `data` is a full `JsonRpcResponseMessage`
   - its current `result.type` is `turn_stream_accepted`
2. All following event names are `message`
   - `data` is a full `ServerEventMessage`
   - the inner frame then distinguishes `started / event / completed / failed`

## Errors

- Requests that successfully reach the handler still use standard `JsonRpcResponseMessage.error`.
- Requests that fail at the HTTP parsing layer return:
  - `400 Bad Request`
  - a plain `ErrorPayload` JSON body instead of a JSON-RPC envelope

## Client Handling Guidance

- Treat unary requests as normal JSON responses.
- Use `fetch` or another client capable of reading `text/event-stream` for `session.run_turn`.
- Consume the initial `ack` event first, then the sequence of `message` events.
