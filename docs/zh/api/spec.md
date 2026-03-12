# API 协议规范

本文档描述 `ss-protocol` 当前使用的请求、响应和流式事件封装结构。它关注协议形状，不展开后端实现细节。

## 1. 总览

- 请求使用 JSON-RPC 2.0 风格消息。
- 普通响应使用单次 JSON-RPC response。
- 流式响应使用独立的服务端事件消息，不复用 JSON-RPC `result` 增量推送。
- 协议是传输无关的，HTTP、WebSocket 或其他通道都可以承载这些消息。

## 2. Request 结构

请求消息结构：

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": "session-1",
  "method": "story.generate",
  "params": {}
}
```

字段说明：

- `jsonrpc`: 固定为 `"2.0"`。
- `id`: 请求标识，由客户端生成，服务端原样回传。
- `session_id`: 可选。仅 session 绑定方法必须提供。
- `method`: 请求方法名。
- `params`: 与 `method` 对应的参数对象。未提供时默认空对象。

通用约定：

- 请求参数对象采用闭合结构；未声明字段应视为无效参数。
- 角色卡、resources、story、session 分别通过各自的 id 引用。
- `session_id` 放在消息顶层，不放在各个 session 方法的 `params` 里。

## 3. Response 结构

普通响应结构：

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": "session-1",
  "result": {
    "type": "story_generated"
  }
}
```

或错误结构：

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "error": {
    "code": -32602,
    "message": "invalid params"
  }
}
```

规则：

- response 必须包含 `result` 或 `error` 二者之一。
- `result` 和 `error` 不能同时出现。
- `session_id` 在 session 相关响应中会原样回显。

### 3.1 Error 结构

```json
{
  "code": 40909,
  "message": "story has sessions",
  "data": {}
}
```

字段说明：

- `code`: 数值错误码。
- `message`: 可读错误信息。
- `data`: 可选补充信息。

当前错误码映射：

| 名称 | 数值 |
| --- | ---: |
| `parse_error` | `-32700` |
| `invalid_request` | `-32600` |
| `method_not_found` | `-32601` |
| `invalid_params` | `-32602` |
| `internal_error` | `-32603` |
| `not_found` | `40404` |
| `conflict` | `40909` |
| `backend_error` | `50001` |
| `stream_error` | `50002` |

## 4. Stream 结构

流式响应不是 JSON-RPC response，而是独立的服务端消息：

```json
{
  "message_type": "stream",
  "request_id": "req-1",
  "session_id": "session-1",
  "sequence": 0,
  "frame": {
    "type": "started"
  }
}
```

字段说明：

- `message_type`: 当前固定为 `"stream"`。
- `request_id`: 对应触发这条流的请求 id。
- `session_id`: 可选，session 绑定流通常会带上。
- `sequence`: 单条流内的顺序号，从 `0` 开始递增。
- `frame`: 流帧内容。

### 4.1 StreamFrame 结构

`frame` 目前有四种形状：

```json
{ "type": "started" }
```

```json
{
  "type": "event",
  "body": {
    "type": "narrator_text_delta"
  }
}
```

```json
{
  "type": "completed",
  "response": {
    "type": "turn_completed"
  }
}
```

```json
{
  "type": "failed",
  "error": {
    "code": 50002,
    "message": "turn failed"
  }
}
```

流式约定：

- 一条流必须先发 `started`。
- 中间可以发任意数量的 `event`。
- 最终必须以 `completed` 或 `failed` 结束。
- `completed` 内必须带最终聚合 `response`，客户端不需要自行拼装最终结果。

## 5. 方法分组

当前方法按职责分为：

- `upload.*`: 分段上传
- `character.*`: 角色卡对象读取、封面读取与删除
- `story_resources.*`: 剧情生成资源对象
- `story.*`: 规划、生成、读取、删除、启动 session
- `session.*`: 会话读取、运行、更新与删除
- `config.*`: 全局配置读取与修改

## 6. 会话绑定规则

必须使用顶层 `session_id` 的方法：

- `session.get`
- `session.delete`
- `session.run_turn`
- `session.update_player_description`
- `session.get_runtime_snapshot`
- `session.get_config`
- `session.update_config`

不需要 `session_id` 的方法：

- `upload.*`
- `character.*`
- `story_resources.*`
- `story.*`
- `session.list`
- `config.*`

## 7. 分段上传协议

大文件采用三段式流程：

1. `upload.init`
2. `upload.chunk` 多次
3. `upload.complete`

上传约定：

- 当前 `target_kind` 只支持 `character_card`。
- chunk 通过 `payload_base64` 传输。
- chunk 需要同时带 `chunk_index` 和 `offset`。
- 完成上传后，服务端会解析 `.chr` 文件并落为角色卡对象。

### 7.1 角色卡直接创建与封面设置

除 `.chr` 上传外，角色卡当前还支持两步式 JSON-RPC 创建：

1. `character.create`
2. `character.set_cover`

约定如下：

- `character.create` 只创建角色内容，不要求封面
- 新建成功后，角色摘要里的 `cover_file_name` / `cover_mime_type` 可以为 `null`
- `character.set_cover` 通过 `cover_base64` 设置封面
- 封面文件名由服务端按 mime type 固定派生，例如 `image/png -> cover.png`

### 7.2 角色卡封面读取

角色卡封面当前通过普通 JSON-RPC 方法读取：

- `character.get_cover`

这个方法：

- 不需要 `session_id`
- 返回普通 unary response
- 响应中携带：
  - `character_id`
  - `cover_file_name`
  - `cover_mime_type`
  - `cover_base64`

- 如果角色还没有封面，则返回 `conflict`

协议当前不提供单独的二进制 HTTP 图片下载端点。

### 7.3 角色卡 `.chr` 导出

角色卡完整归档当前也通过普通 JSON-RPC 方法导出：

- `character.export_chr`

这个方法：

- 不需要 `session_id`
- 返回普通 unary response
- 响应中携带：
  - `character_id`
  - `file_name`
  - `content_type`
  - `chr_base64`

`chr_base64` 是完整 `.chr` ZIP 归档的 base64 编码。协议当前不提供单独的 HTTP 二进制下载端点。
如果角色还没有封面，则返回 `conflict`。

## 8. `.chr` 角色卡文件格式

`.chr` 是 ZIP 容器，内部固定包含：

- `manifest.json`
- `content.json`
- `cover.<ext>`

### 8.1 manifest.json

```json
{
  "format": "sillystage_character_card",
  "version": 1,
  "character_id": "merchant",
  "content_path": "content.json",
  "cover_path": "cover.png",
  "cover_mime_type": "image/png"
}
```

### 8.2 content.json

```json
{
  "id": "merchant",
  "name": "Old Merchant",
  "personality": "greedy but friendly trader",
  "style": "talkative, casual, slightly cunning",
  "tendencies": ["likes profitable deals"],
  "state_schema": {},
  "system_prompt": "..."
}
```

### 8.3 cover

- 支持 `image/png`
- 支持 `image/jpeg`
- 支持 `image/webp`

## 9. 配置结构

### 9.1 AgentApiIds

```json
{
  "planner_api_id": "default",
  "architect_api_id": "default",
  "director_api_id": "default",
  "actor_api_id": "default",
  "narrator_api_id": "default",
  "keeper_api_id": "default"
}
```

### 9.2 AgentApiIdOverrides

与 `AgentApiIds` 同字段，但每个字段都是可选字符串，用于按请求覆盖。

### 9.3 SessionConfigMode

- `use_global`
- `use_session`

## 10. 运行时流式返回

`session.run_turn` 是当前唯一标准流式方法：

1. 普通 JSON-RPC ack 返回 `turn_stream_accepted`
2. 服务端随后发送 stream 事件
3. `completed.response` 返回 `turn_completed`
4. 或以 `failed.error` 结束

前端应把 ack 和后续 stream 当作同一次请求的两段结果处理。
