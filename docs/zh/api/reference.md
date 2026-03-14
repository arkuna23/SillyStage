# API 参考

本文档列出当前已经实现的 JSON-RPC 方法。

## 1. upload

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `upload.init` | 否 | `upload_initialized` | 初始化分段上传 |
| `upload.chunk` | 否 | `upload_chunk_accepted` | 上传分片 |
| `upload.complete` | 否 | `character_card_uploaded` | 完成 `.chr` 上传并落为角色卡 |

## 2. llm_api

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `llm_api.create` | 否 | `llm_api` | 创建 LLM API 定义 |
| `llm_api.get` | 否 | `llm_api` | 获取单个 LLM API 定义 |
| `llm_api.list` | 否 | `llm_apis_listed` | 列出 LLM API 定义 |
| `llm_api.update` | 否 | `llm_api` | 更新 LLM API 定义 |
| `llm_api.delete` | 否 | `llm_api_deleted` | 删除 LLM API 定义 |

说明：

- 读取接口不返回明文 `api_key`
- 删除时若仍被 global config 或 session config 引用，返回 `conflict`

## 3. schema

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `schema.create` | 否 | `schema` | 创建 schema 资源 |
| `schema.get` | 否 | `schema` | 获取单个 schema |
| `schema.list` | 否 | `schemas_listed` | 列出 schema |
| `schema.update` | 否 | `schema` | 更新 schema |
| `schema.delete` | 否 | `schema_deleted` | 删除 schema |

说明：

- `schema` 没有固定 kind，分类由 `tags` 负责
- 若 schema 仍被角色卡、resources、story 或 session 引用，删除返回 `conflict`

## 4. player_profile

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `player_profile.create` | 否 | `player_profile` | 创建玩家设定 |
| `player_profile.get` | 否 | `player_profile` | 获取单个玩家设定 |
| `player_profile.list` | 否 | `player_profiles_listed` | 列出玩家设定 |
| `player_profile.update` | 否 | `player_profile` | 更新玩家设定 |
| `player_profile.delete` | 否 | `player_profile_deleted` | 删除玩家设定 |

说明：

- 玩家设定字段为 `player_profile_id`、`display_name`、`description`
- 删除时若仍被 session 引用，返回 `conflict`

## 5. character

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `character.create` | 否 | `character_created` | 直接从请求数据创建角色卡 |
| `character.get` | 否 | `character` | 获取完整角色内容 |
| `character.update` | 否 | `character` | 更新角色内容 |
| `character.list` | 否 | `characters_listed` | 获取角色摘要列表 |
| `character.delete` | 否 | `character_deleted` | 删除角色卡 |
| `character.set_cover` | 否 | `character_cover_updated` | 设置或替换封面 |
| `character.get_cover` | 否 | `character_cover` | 获取封面 base64 |
| `character.export_chr` | 否 | `character_chr_export` | 导出 `.chr` |

角色内容字段：

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `schema_id`
- `system_prompt`

说明：

- `schema_id` 引用角色私有状态 schema
- 封面是独立更新接口
- `.chr` 导出要求角色卡已具备封面

## 6. story_resources

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `story_resources.create` | 否 | `story_resources` | 创建 story resources |
| `story_resources.get` | 否 | `story_resources` | 获取单个 resources |
| `story_resources.list` | 否 | `story_resources_listed` | 列出 resources |
| `story_resources.update` | 否 | `story_resources` | 更新 resources |
| `story_resources.delete` | 否 | `story_resources_deleted` | 删除 resources |

字段：

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `planned_story`

## 7. story

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `story.generate_plan` | 否 | `story_planned` | 调用 Planner 生成可编辑剧本 |
| `story.generate` | 否 | `story_generated` | 生成剧情图和 schema 引用 |
| `story.get` | 否 | `story` | 获取 story 详情 |
| `story.list` | 否 | `stories_listed` | 列出 story |
| `story.delete` | 否 | `story_deleted` | 删除 story |
| `story.start_session` | 否 | `session_started` | 从 story 创建新 session |

`story_generated` 关键字段：

- `story_id`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`

`story.start_session` 输入：

- `story_id`
- 可选 `display_name`
- 可选 `player_profile_id`
- `config_mode`
- 可选 `session_api_ids`

## 8. session

| 方法 | session_id | 返回 | 流式 |
| --- | --- | --- | --- |
| `session.get` | 是 | `session` | 否 |
| `session.list` | 否 | `sessions_listed` | 否 |
| `session.delete` | 是 | `session_deleted` | 否 |
| `session.run_turn` | 是 | `turn_stream_accepted` / `turn_completed` | 是 |
| `session.set_player_profile` | 是 | `session` | 否 |
| `session.update_player_description` | 是 | `player_description_updated` | 否 |
| `session.get_runtime_snapshot` | 是 | `runtime_snapshot` | 否 |
| `session.get_config` | 是 | `session_config` | 否 |
| `session.update_config` | 是 | `session_config` | 否 |

说明：

- `session.set_player_profile` 只切换当前玩家设定，不切换 `player_state`
- `session.update_player_description` 会清空 `player_profile_id`，改成直接使用手动描述

## 9. config

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `config.get_global` | 否 | `global_config` | 获取全局 agent API 选择 |
| `config.update_global` | 否 | `global_config` | 更新全局 agent API 选择 |

## 10. dashboard

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `dashboard.get` | 否 | `dashboard` | 获取 dashboard 聚合摘要 |

`dashboard` 包含：

- `health`
- `counts`
- `global_config`
- `recent_stories`
- `recent_sessions`

