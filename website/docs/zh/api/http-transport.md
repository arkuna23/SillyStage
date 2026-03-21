# HTTP 传输说明

本文描述 `ss-server` 当前暴露的 HTTP 传输层。

## 路由

- `POST /rpc`
- `GET /healthz`
- `POST /upload/{resource_id}/{file_id}`
- `GET /download/{resource_id}/{file_id}`

## JSON-RPC

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

普通方法返回：

- `200 OK`
- `Content-Type: application/json`
- body 为完整 `JsonRpcResponseMessage`

## 流式响应

`session.run_turn` 仍然走 `POST /rpc`，但响应类型变为：

- `200 OK`
- `Content-Type: text/event-stream`

事件规则：

1. 第一个事件名固定为 `ack`
2. 后续事件名固定为 `message`

## 二进制上传与下载

二进制路由不使用 JSON-RPC。

- 上传 body 是原始字节
- `x-file-name` 是可选请求头
- 缺失 `Content-Type` 时先归一化为 `application/octet-stream`

当前内置资源文件：

- `character:{character_id}/cover`
- `character:{character_id}/archive`
- `package_import:{import_id}/archive`
- `package_export:{export_id}/archive`

### `POST /upload/{resource_id}/{file_id}`

成功返回：

- `200 OK`
- `Content-Type: application/json`
- body 为 `ResourceFilePayload`

当前内置语义：

- 角色封面上传
- `.chr` 角色卡导入
- 数据包 ZIP 上传到临时导入槽位

### `GET /download/{resource_id}/{file_id}`

成功返回：

- `200 OK`
- body 为原始字节
- `Content-Type` 为逻辑文件类型

当前内置语义：

- 下载角色封面
- 导出 `.chr`
- 下载准备好的数据包 ZIP

## 错误处理

- `/rpc` 解析失败：`400 Bad Request` + `ErrorPayload`
- 二进制路由：
  - `400 Bad Request`
  - `404 Not Found`
  - `409 Conflict`
  - `500 Internal Server Error`

## 客户端建议

- 普通 `/rpc` 请求按 JSON 处理
- `session.run_turn` 使用支持 `text/event-stream` 的客户端
- `/upload/...` 作为原始字节上传
- `/download/...` 作为原始字节下载
