# API 协议结构

本文描述当前 `ss-protocol` 的结构约定，重点说明消息封装、资源模型与 session 相关语义。

## 1. 传输模型

后端同时使用三种传输形态：

- `POST /rpc` 上的 JSON-RPC 2.0
- 基于 SSE 的流式响应
- `/upload/{resource_id}/{file_id}` 与 `/download/{resource_id}/{file_id}` 的二进制文件传输

### 1.1 请求

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": "session-1",
  "method": "story.generate",
  "params": {}
}
```

### 1.2 单次响应

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": null,
  "result": {
    "type": "story_generated"
  }
}
```

### 1.3 流式响应

`session.run_turn` 会先返回一个 `ack`，随后连续推送 `message` 事件。

流式帧类型：

- `started`
- `event`
- `completed`
- `failed`

### 1.4 二进制文件传输

- 上传和下载不使用 JSON-RPC envelope
- `resource_id + file_id` 是公开文件标识
- 当前内置资源文件包括：
  - `character:{character_id}/cover`
  - `character:{character_id}/archive`
  - `package_import:{import_id}/archive`
  - `package_export:{export_id}/archive`

## 2. 核心资源模型

### 2.1 连接与提示词配置

- `api`：单个连接定义
- `api_group`：每个 agent 的 `api_id` 绑定
- `preset`：每个 agent 的生成参数与模块化提示词配置

运行时绑定模型当前使用 `api_group_id + preset_id`。

### 2.2 状态与资料资源

- `schema`：独立 schema 资源
- `lorebook`：可复用世界设定集合
- `player_profile`：独立玩家设定
- `resource_file`：二进制资源文件的传输无关标识

### 2.3 创作资源

- `character`：角色卡内容与封面元数据
- `story_resources`：生成 story 前的输入资源集合
- `story`：最终故事图与 schema 绑定
- `session`：story 运行时快照与 transcript

## 3. 方法族

当前协议按资源分组为：

- 二进制文件路由
- `api.*`
- `api_group.*`
- `preset.*`
- `schema.*`
- `lorebook.*`
- `lorebook_entry.*`
- `player_profile.*`
- `character.*`
- `story_resources.*`
- `story.*`
- `story_draft.*`
- `session.*`
- `session_character.*`
- `session_message.*`
- `config.*`
- `dashboard.get`
- `data_package.*`

## 4. Session 相关语义

### 4.1 启动 session

`story.start_session` 接收：

- `story_id`
- 可选 `display_name`
- 可选 `player_profile_id`
- 可选 `api_group_id`
- 可选 `preset_id`

若省略绑定 id，而系统里存在可用资源，则自动选择排序后的第一个 `api_group` 和 `preset`。

### 4.2 draft story 生成

`story_draft.*` 是大体量 story 的推荐生成路径：

- `story_draft.start`
- `story_draft.update_graph`
- `story_draft.continue`
- `story_draft.finalize`

它把 partial graph 存在服务端，不要求客户端反复回传已生成节点。

### 4.3 变量面板

`story.common_variables` 可以与 `session.get_variables` 或 runtime snapshot 组合使用，用于渲染固定展示的变量面板。

### 4.4 session 变量更新

`session.update_variables` 只允许变量类 op，例如：

- `SetState`
- `RemoveState`
- `SetPlayerState`
- `RemovePlayerState`
- `SetCharacterState`
- `RemoveCharacterState`

像 `SetCurrentNode` 这类场景控制 op 会被拒绝。
