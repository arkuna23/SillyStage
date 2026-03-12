# HTTP 传输约定

本文档描述 `ss-server` 当前提供的 HTTP 承载方式。协议对象本身仍然来自 `ss-protocol`。

## 路由

- `POST /rpc`
  - 统一承载所有协议请求。
- `GET /healthz`
  - 返回纯文本 `ok`。
  - 这是运维辅助端点，不属于 `ss-protocol` 方法集。

## 请求体

`POST /rpc` 的请求体必须是完整的 `JsonRpcRequestMessage` JSON。

示例：

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

## Unary 响应

普通方法返回：

- HTTP 状态码：`200 OK`
- `Content-Type: application/json`
- body：完整 `JsonRpcResponseMessage`

## Stream 响应

当前流式方法是 `session.run_turn`。它仍然走 `POST /rpc`，但 HTTP 响应为：

- HTTP 状态码：`200 OK`
- `Content-Type: text/event-stream`

SSE 事件规则：

1. 第一条事件名固定为 `ack`
   - `data` 是完整 `JsonRpcResponseMessage`
   - 其 `result.type` 当前为 `turn_stream_accepted`
2. 后续事件名固定为 `message`
   - `data` 是完整 `ServerEventMessage`
   - 内部再区分 `started / event / completed / failed`

## 错误

- 对于可成功解析并进入 handler 的请求，错误仍按标准 `JsonRpcResponseMessage.error` 返回。
- 对于 HTTP 层无法解析的原始 body：
  - 返回 `400 Bad Request`
  - body 是 `ErrorPayload` JSON，而不是 JSON-RPC envelope

## 客户端处理建议

- 所有 unary 请求按普通 JSON 处理。
- `session.run_turn` 使用 `fetch` 或其他支持读取 `text/event-stream` 响应体的客户端。
- 先消费 `ack`，再消费连续的 `message` 事件。
