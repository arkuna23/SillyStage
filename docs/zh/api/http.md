# HTTP 传输说明

本文描述 `ss-server` 当前暴露的 HTTP 传输层。后端现在有两种 HTTP 形态：

- `POST /rpc` 上的 JSON-RPC
- 纯传输用途的二进制路由：
  - `POST /upload/{resource_id}/{file_id}`
  - `GET /download/{resource_id}/{file_id}`

协议对象本身仍然来自 `ss-protocol`。

## 路由

- `POST /rpc`
  - JSON-RPC 请求的统一入口。
- `GET /healthz`
  - 返回纯文本 `ok`。
  - 这是运维辅助端点，不属于 `ss-protocol` 方法。
- `POST /upload/{resource_id}/{file_id}`
  - 向某个逻辑资源文件槽位上传原始字节。
- `GET /download/{resource_id}/{file_id}`
  - 从某个逻辑资源文件槽位下载原始字节。

当前内置支持的资源文件：

- `character:{character_id}/cover`
- `character:{character_id}/archive`
- `package_import:{import_id}/archive`
- `package_export:{export_id}/archive`

## JSON-RPC 请求体

`POST /rpc` 需要完整的 `JsonRpcRequestMessage` JSON。

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

## JSON-RPC 响应

普通方法返回：

- HTTP 状态：`200 OK`
- `Content-Type: application/json`
- body：完整 `JsonRpcResponseMessage`

## 流式响应

当前流式方法是 `session.run_turn`。它仍然走 `POST /rpc`，但 HTTP 响应会变成：

- HTTP 状态：`200 OK`
- `Content-Type: text/event-stream`

SSE 事件规则：

1. 第一个事件名始终是 `ack`
   - `data` 是完整 `JsonRpcResponseMessage`
   - 当前 `result.type` 为 `turn_stream_accepted`
2. 后续事件名全部是 `message`
   - `data` 是完整 `ServerEventMessage`
   - 内层 frame 再区分 `started / event / completed / failed`

## 二进制请求与响应规则

- 二进制路由不使用 JSON-RPC envelope。
- 上传 body 是原始字节，不是 JSON 里的 base64。
- `resource_id + file_id` 是协议层的文件标识，不暴露内部 blob id。
- `x-file-name` 是 `POST /upload/{resource_id}/{file_id}` 的可选请求头。
- 当 `x-file-name` 缺失或为空时：
  - `character:{character_id}/cover` 会根据 `Content-Type` 回退为 `cover.<ext>`
  - `character:{character_id}/archive` 会回报 `{character_id}.chr`
  - `package_import:{import_id}/archive` 会回报 `sillystage-data-package-{import_id}.zip`
- 缺失或空白的 `Content-Type` 会先被归一化为 `application/octet-stream`，再做校验。
- `character:{character_id}/cover` 要求使用支持的图片 `Content-Type`：
  - `image/png`
  - `image/jpeg`
  - `image/webp`
- `character:{character_id}/archive` 接收原始 `.chr` 字节。客户端通常应发送
  `Content-Type: application/x-sillystage-character-card`。
- `package_import:{import_id}/archive` 接收原始 ZIP 字节。客户端通常应发送
  `Content-Type: application/x-sillystage-data-package+zip`。

## 二进制路由语义

### `POST /upload/{resource_id}/{file_id}`

- `200 OK`
- `Content-Type: application/json`
- body：`ResourceFilePayload`
- 字段：
  - `resource_id`
  - `file_id`
  - `file_name`
  - `content_type`
  - `size_bytes`

当前内置上传语义：

- `POST /upload/character:{character_id}/cover`
  - 保存或替换角色封面
- `POST /upload/character:{character_id}/archive`
  - 把一个 `.chr` 导入到固定角色槽位
  - 压缩包里的 `content.id` 必须与 `{character_id}` 一致
- `POST /upload/package_import:{import_id}/archive`
  - 把一个数据包 ZIP 上传到临时导入槽位
  - 单独上传不会修改持久化资源；真正应用由 `data_package.import_commit` 完成

### `GET /download/{resource_id}/{file_id}`

- `200 OK`
- body：原始字节
- `Content-Type`：该逻辑文件的内容类型
- `Content-Disposition`：存在文件名时使用附件下载

当前内置下载语义：

- `GET /download/character:{character_id}/cover`
  - 返回当前封面图片字节
- `GET /download/character:{character_id}/archive`
  - 导出当前角色为 `.chr`
- `GET /download/package_export:{export_id}/archive`
  - 返回由 `data_package.export_prepare` 准备好的数据包 ZIP

## 错误

- 成功进入 `/rpc` 的请求仍然使用标准 `JsonRpcResponseMessage.error`。
- 在 `/rpc` 的 HTTP 解析阶段就失败的请求会返回：
  - `400 Bad Request`
  - 纯 `ErrorPayload` JSON，而不是 JSON-RPC envelope
- 二进制路由返回纯 `ErrorPayload` JSON，并映射为 HTTP 状态码：
  - `400 Bad Request`：二进制输入不合法，或 `resource_id/file_id` 非法
  - `404 Not Found`：资源不存在
  - `409 Conflict`：状态冲突，例如导入了已存在的角色
  - `500 Internal Server Error`：未知失败

## 客户端处理建议

- 把普通 `/rpc` 请求当作正常 JSON 处理。
- `session.run_turn` 需要使用能读取 `text/event-stream` 的客户端。
- 先消费 `ack` 事件，再消费后续 `message` 事件。
- 把 `/upload/...` 当作原始字节上传，把 `/download/...` 当作原始字节下载。
- 角色 JSON payload 只暴露封面元数据：
  - `cover_file_name`
  - `cover_mime_type`
- 实际封面字节通过 `GET /download/character:{character_id}/cover` 获取。
