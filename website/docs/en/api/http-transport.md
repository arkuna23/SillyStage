# HTTP Transport

This page describes the current HTTP transport exposed by `ss-server`.

## Routes

- `POST /rpc`
- `GET /healthz`
- `POST /upload/{resource_id}/{file_id}`
- `GET /download/{resource_id}/{file_id}`

## JSON-RPC

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

Regular methods return:

- `200 OK`
- `Content-Type: application/json`
- a full `JsonRpcResponseMessage`

## Streaming Responses

`session.run_turn` still uses `POST /rpc`, but the response becomes:

- `200 OK`
- `Content-Type: text/event-stream`

Rules:

1. the first event name is always `ack`
2. all following event names are `message`

## Binary Upload and Download

Binary routes do not use JSON-RPC envelopes.

- upload bodies are raw bytes
- `x-file-name` is optional
- missing `Content-Type` is normalized to `application/octet-stream`

Current built-in resource files:

- `character:{character_id}/cover`
- `character:{character_id}/archive`
- `package_import:{import_id}/archive`
- `package_export:{export_id}/archive`

### `POST /upload/{resource_id}/{file_id}`

Success response:

- `200 OK`
- `Content-Type: application/json`
- body: `ResourceFilePayload`

Current built-in behavior:

- character cover upload
- `.chr` character import
- ZIP upload into a temporary data-package import slot

### `GET /download/{resource_id}/{file_id}`

Success response:

- `200 OK`
- raw bytes
- `Content-Type` set to the logical file type

Current built-in behavior:

- download cover bytes
- export `.chr`
- download a prepared data-package ZIP

## Errors

- `/rpc` parse failures: `400 Bad Request` with `ErrorPayload`
- binary routes may return:
  - `400 Bad Request`
  - `404 Not Found`
  - `409 Conflict`
  - `500 Internal Server Error`
