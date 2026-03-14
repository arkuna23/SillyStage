# API 协议结构

本文档描述当前 `ss-protocol` 的结构约定，重点说明消息封装和资源模型。具体方法清单见 [reference.md](./reference.md)。

## 1. 传输模型

后端协议使用 JSON-RPC 2.0 作为请求/响应封装，流式结果通过独立的服务端事件消息发送。

### 1.1 请求

所有请求都是一个 JSON-RPC 请求对象：

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": "session-1",
  "method": "story.generate",
  "params": {}
}
```

- `id`: 请求 id，由客户端生成。
- `session_id`: 只有 session 绑定请求才需要；非 session 请求可省略或为 `null`。
- `method`: 方法名。
- `params`: 该方法对应的参数对象。

### 1.2 单次响应

单次响应也是标准 JSON-RPC 结构：

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": null,
  "result": {
    "type": "story_generated",
    "...": "..."
  }
}
```

错误响应：

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": null,
  "error": {
    "code": "conflict",
    "message": "schema 'schema-player-default' is still referenced",
    "details": null,
    "retryable": false
  }
}
```

### 1.3 流式响应

`session.run_turn` 这类请求会先返回一个 unary `ack`，随后发送服务端事件：

```json
{
  "message_type": "stream",
  "request_id": "req-turn",
  "session_id": "session-1",
  "sequence": 3,
  "frame": {
    "type": "event",
    "event": {
      "type": "actor_dialogue_delta",
      "...": "..."
    }
  }
}
```

流式帧类型：

- `started`
- `event`
- `completed`
- `failed`

`completed` 会携带最终聚合结果，前端不需要自己拼完整 turn 结果。

## 2. 资源模型

### 2.1 `llm_api`

`llm_api` 是可持久化的大模型 API 定义对象。每条记录包含：

- `api_id`
- `provider`
- `base_url`
- `api_key`
- `model`

读取接口不会返回明文 `api_key`，而是返回：

- `has_api_key`
- `api_key_masked`

global config 和 session config 只引用 `api_id`，不内联 provider 配置。

### 2.2 `schema`

`schema` 是独立资源，不再内联到角色卡、resources 或 story 中。

字段：

- `schema_id`
- `display_name`
- `tags: string[]`
- `fields`

说明：

- `schema` 没有固定 `kind`。
- `tags` 用于用户或前端标注该 schema 的用途，例如 `player`、`world`、`character`。
- `fields` 的结构与 `StateFieldSchema` 一致。

### 2.3 `player_profile`

`player_profile` 表示“玩家设定”，是独立的可切换资源。

字段：

- `player_profile_id`
- `display_name`
- `description`

说明：

- 一个 story 可以对应多个 player profile。
- 一个 session 同时只激活一个 `player_profile_id`。
- 切换 player profile 不会切换 `player_state`。

### 2.4 `character`

角色卡内容通过 `CharacterCardContent` 表示：

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `schema_id`
- `system_prompt`

说明：

- 角色卡不再内联 `state_schema`。
- 角色私有 schema 通过 `schema_id` 引用独立 `schema` 资源。
- 封面与 `.chr` 导出是独立接口。

### 2.5 `story_resources`

`story_resources` 是生成 story 前的输入资源集合。

字段：

- `resource_id`
- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `planned_story`

说明：

- `character_ids` 只引用已上传或已创建的角色卡。
- 两个 schema seed 都是可选 id。
- `planned_story` 是可选的 planner 文本。

### 2.6 `story`

生成完成后的 `story` 记录包含：

- `story_id`
- `display_name`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`

说明：

- `Architect` 生成 world/player schema 本体后，engine manager 会先落成 `schema` 资源，再把 id 写入 story。
- story 本身只保存 schema id。

### 2.7 `session`

session 绑定一个 story 和一份运行时快照。

字段：

- `session_id`
- `story_id`
- `display_name`
- `player_profile_id`
- `player_schema_id`
- `snapshot`
- `config`

说明：

- `player_profile_id` 可为空；为空表示当前 session 使用手动覆盖后的 `player_description`。
- `player_schema_id` 固定引用该 session 使用的玩家状态 schema。
- `snapshot` 保存当前动态运行状态，包括 `world_state`、`turn_index`，以及当前生效的 `player_description` 文本。

## 3. 角色卡文件 `.chr`

`.chr` 是一个 ZIP 容器，固定包含：

- `manifest.json`
- `content.json`
- `cover.<ext>`

`content.json` 使用 `CharacterCardContent`。

更多细节见：

- [../character.md](../character.md)

## 4. 方法族

当前协议按资源分为以下方法族：

- `upload.*`
- `llm_api.*`
- `schema.*`
- `player_profile.*`
- `character.*`
- `story_resources.*`
- `story.*`
- `session.*`
- `config.*`
- `dashboard.get`

## 5. Session 相关语义

### 5.1 启动 session

`story.start_session` 输入：

- `story_id`
- 可选 `display_name`
- 可选 `player_profile_id`
- `config_mode`
- 可选 `session_api_ids`

### 5.2 切换玩家设定

`session.set_player_profile` 只切换当前 session 的 `player_profile_id` 和生效描述，不切换 `player_state`。

### 5.3 手动覆盖玩家描述

`session.update_player_description` 会直接覆盖当前 session 的描述文本，并把 `player_profile_id` 置空。

## 6. 删除约束

- `schema.delete`: 若 schema 仍被角色卡、resources、story 或 session 引用，则返回 `conflict`
- `player_profile.delete`: 若 profile 仍被 session 引用，则返回 `conflict`
- `character.delete`: 若角色卡仍被 `story_resources` 引用，则返回 `conflict`
- `story.delete`: 若 story 仍有关联 session，则返回 `conflict`

