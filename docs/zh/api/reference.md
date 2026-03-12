# API 参考

本文档列出当前 `ss-protocol` 中已经存在的请求、响应和流事件。字段名与方法名均以当前代码为准。

## 1. 请求方法总览

| 方法 | `session_id` | 成功结果 | 流式 |
| --- | --- | --- | --- |
| `upload.init` | 否 | `upload_initialized` | 否 |
| `upload.chunk` | 否 | `upload_chunk_accepted` | 否 |
| `upload.complete` | 否 | `character_card_uploaded` | 否 |
| `character.get` | 否 | `character` | 否 |
| `character.list` | 否 | `characters_listed` | 否 |
| `character.delete` | 否 | `character_deleted` | 否 |
| `story_resources.create` | 否 | `story_resources_created` | 否 |
| `story_resources.get` | 否 | `story_resources` | 否 |
| `story_resources.list` | 否 | `story_resources_listed` | 否 |
| `story_resources.update` | 否 | `story_resources_updated` | 否 |
| `story_resources.delete` | 否 | `story_resources_deleted` | 否 |
| `story.generate_plan` | 否 | `story_planned` | 否 |
| `story.generate` | 否 | `story_generated` | 否 |
| `story.get` | 否 | `story` | 否 |
| `story.list` | 否 | `stories_listed` | 否 |
| `story.delete` | 否 | `story_deleted` | 否 |
| `story.start_session` | 否 | `session_started` | 否 |
| `session.get` | 是 | `session` | 否 |
| `session.list` | 否 | `sessions_listed` | 否 |
| `session.delete` | 是 | `session_deleted` | 否 |
| `session.run_turn` | 是 | `turn_stream_accepted` | 是 |
| `session.update_player_description` | 是 | `player_description_updated` | 否 |
| `session.get_runtime_snapshot` | 是 | `runtime_snapshot` | 否 |
| `config.get_global` | 否 | `global_config` | 否 |
| `config.update_global` | 否 | `global_config` | 否 |
| `session.get_config` | 是 | `session_config` | 否 |
| `session.update_config` | 是 | `session_config` | 否 |

## 2. Upload API

### 2.1 `upload.init`

参数：

```json
{
  "target_kind": "character_card",
  "file_name": "merchant.chr",
  "content_type": "application/octet-stream",
  "total_size": 123456,
  "sha256": "..."
}
```

结果：

```json
{
  "type": "upload_initialized",
  "upload_id": "upload-0",
  "chunk_size_hint": 65536
}
```

### 2.2 `upload.chunk`

参数：

```json
{
  "upload_id": "upload-0",
  "chunk_index": 0,
  "offset": 0,
  "payload_base64": "...",
  "is_last": false
}
```

结果：

```json
{
  "type": "upload_chunk_accepted",
  "upload_id": "upload-0",
  "received_chunk_index": 0,
  "received_bytes": 65536
}
```

### 2.3 `upload.complete`

参数：

```json
{
  "upload_id": "upload-0"
}
```

结果：

```json
{
  "type": "character_card_uploaded",
  "character_id": "merchant",
  "character_summary": {
    "character_id": "merchant",
    "name": "Old Merchant",
    "personality": "greedy but friendly trader",
    "style": "talkative, casual, slightly cunning",
    "tendencies": [],
    "cover_file_name": "cover.png",
    "cover_mime_type": "image/png"
  }
}
```

## 3. Character API

### 3.1 `character.get`

参数：

```json
{
  "character_id": "merchant"
}
```

结果：`character`

- `character_id`
- `content`
- `cover_file_name`
- `cover_mime_type`

`content` 对应 `.chr` 内的 `content.json`。

### 3.2 `character.list`

参数：`{}`

结果：`characters_listed`

- `characters: CharacterCardSummaryPayload[]`

### 3.3 `character.delete`

参数：

```json
{
  "character_id": "merchant"
}
```

结果：`character_deleted`

## 4. Story Resources API

### 4.1 `story_resources.create`

参数：

```json
{
  "story_concept": "A tense negotiation in a flooded city.",
  "character_ids": ["merchant", "guard"],
  "player_state_schema_seed": {},
  "world_state_schema_seed": null,
  "planned_story": null
}
```

结果：`story_resources_created`

字段：

- `resource_id`
- `story_concept`
- `character_ids`
- `player_state_schema_seed`
- `world_state_schema_seed`
- `planned_story`

### 4.2 `story_resources.get`

参数：`{ "resource_id": "resource-0" }`

结果：`story_resources`

### 4.3 `story_resources.list`

参数：`{}`

结果：`story_resources_listed`

- `resources: StoryResourcesPayload[]`

### 4.4 `story_resources.update`

参数中所有更新字段都是可选：

- `story_concept`
- `character_ids`
- `player_state_schema_seed`
- `world_state_schema_seed`
- `planned_story`

结果：`story_resources_updated`

### 4.5 `story_resources.delete`

参数：`{ "resource_id": "resource-0" }`

结果：`story_resources_deleted`

## 5. Story API

### 5.1 `story.generate_plan`

参数：

```json
{
  "resource_id": "resource-0",
  "planner_api_id": "planner-fast"
}
```

`planner_api_id` 可选。

结果：`story_planned`

- `resource_id`
- `story_script`

### 5.2 `story.generate`

参数：

```json
{
  "resource_id": "resource-0",
  "display_name": "Flood Market",
  "architect_api_id": "architect-main"
}
```

结果：`story_generated`

- `resource_id`
- `story_id`
- `display_name`
- `graph`
- `world_state_schema`
- `player_state_schema`
- `introduction`

### 5.3 `story.get`

参数：`{ "story_id": "story-0" }`

结果：`story`

- `story_id`
- `display_name`
- `resource_id`
- `graph`
- `world_state_schema`
- `player_state_schema`
- `introduction`

### 5.4 `story.list`

参数：`{}`

结果：`stories_listed`

每项 `StorySummaryPayload` 包含：

- `story_id`
- `display_name`
- `resource_id`
- `introduction`

### 5.5 `story.delete`

参数：`{ "story_id": "story-0" }`

结果：`story_deleted`

### 5.6 `story.start_session`

参数：

```json
{
  "story_id": "story-0",
  "display_name": "Negotiation Run",
  "player_description": "A careful trader with little money.",
  "config_mode": "use_global",
  "session_api_ids": null
}
```

字段说明：

- `display_name`: 可选；session 显示名。
- `player_description`: 必填。
- `config_mode`: `use_global` 或 `use_session`，默认 `use_global`。
- `session_api_ids`: 仅在 `use_session` 场景下有意义。

结果：`session_started`

- `story_id`
- `display_name`
- `snapshot`
- `character_summaries`
- `config`

## 6. Session API

### 6.1 `session.get`

顶层必须带 `session_id`，参数固定为空对象。

结果：`session`

- `session_id`
- `story_id`
- `display_name`
- `snapshot`
- `config`

### 6.2 `session.list`

参数：`{}`

结果：`sessions_listed`

每项包含：

- `session_id`
- `story_id`
- `display_name`
- `turn_index`

### 6.3 `session.delete`

顶层必须带 `session_id`，参数固定为空对象。

结果：`session_deleted`

### 6.4 `session.run_turn`

顶层必须带 `session_id`。

请求参数：

```json
{
  "player_input": "I ask the merchant about the flood gate.",
  "api_overrides": {
    "director_api_id": "director-main"
  }
}
```

普通 ack 结果：

```json
{
  "type": "turn_stream_accepted"
}
```

随后服务端开始发送 stream 事件。完成帧中的最终结果是：

```json
{
  "type": "turn_completed",
  "result": {}
}
```

### 6.5 `session.update_player_description`

顶层必须带 `session_id`。

参数：

```json
{
  "player_description": "Now the player is suspicious and impatient."
}
```

结果：`player_description_updated`

- `snapshot`

### 6.6 `session.get_runtime_snapshot`

顶层必须带 `session_id`，参数固定为空对象。

结果：`runtime_snapshot`

- `snapshot`

### 6.7 `session.get_config`

顶层必须带 `session_id`，参数固定为空对象。

结果：`session_config`

- `mode`
- `session_api_ids`
- `effective_api_ids`

### 6.8 `session.update_config`

顶层必须带 `session_id`。

参数：

```json
{
  "mode": "use_session",
  "session_api_ids": {
    "planner_api_id": "default",
    "architect_api_id": "default",
    "director_api_id": "default",
    "actor_api_id": "default",
    "narrator_api_id": "default",
    "keeper_api_id": "default"
  },
  "api_overrides": {
    "actor_api_id": "actor-large"
  }
}
```

结果：`session_config`

## 7. Config API

### 7.1 `config.get_global`

参数：`{}`

结果：`global_config`

- `api_ids`

### 7.2 `config.update_global`

参数：

```json
{
  "api_overrides": {
    "architect_api_id": "architect-large"
  }
}
```

结果：`global_config`

## 8. Stream 事件

`session.run_turn` 当前会发出以下 `event.body.type`：

- `turn_started`
- `player_input_recorded`
- `keeper_applied`
- `director_completed`
- `narrator_started`
- `narrator_text_delta`
- `narrator_completed`
- `actor_started`
- `actor_thought_delta`
- `actor_action_complete`
- `actor_dialogue_delta`
- `actor_completed`

流结束方式：

- `completed`，其中 `response.type = "turn_completed"`
- `failed`，其中 `error` 为标准错误对象

## 9. 角色卡归档参考

上传到 `upload.*` 的角色卡文件必须是 `.chr` ZIP 归档，内部包含：

- `manifest.json`
- `content.json`
- `cover.<ext>`

`content.json` 当前字段：

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `state_schema`
- `system_prompt`
