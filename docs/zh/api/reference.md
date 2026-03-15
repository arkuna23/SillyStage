# API 参考

本文档列出当前已经实现的 JSON-RPC 方法。

## 1. upload

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `upload.init` | 否 | `upload_initialized` | 初始化分段上传 |
| `upload.chunk` | 否 | `upload_chunk_accepted` | 上传分片 |
| `upload.complete` | 否 | `character_card_uploaded` | 完成 `.chr` 上传并落为角色卡 |

## 2. api

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `api.create` | 否 | `api` | 创建一个可复用 API 定义 |
| `api.get` | 否 | `api` | 获取单个 API 定义 |
| `api.list` | 否 | `apis_listed` | 列出 API 定义 |
| `api.update` | 否 | `api` | 更新 API 定义 |
| `api.delete` | 否 | `api_deleted` | 删除 API 定义 |

`api` 保存一份可复用的连接配置：

- `provider`
- `base_url`
- `api_key`
- `model`

说明：

- 读取接口不返回明文 `api_key`
- 如果 API 仍被 `api_group` 引用，删除返回 `conflict`

## 3. api_group

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `api_group.create` | 否 | `api_group` | 创建 API 组 |
| `api_group.get` | 否 | `api_group` | 获取单个 API 组 |
| `api_group.list` | 否 | `api_groups_listed` | 列出 API 组 |
| `api_group.update` | 否 | `api_group` | 更新 API 组 |
| `api_group.delete` | 否 | `api_group_deleted` | 删除 API 组 |

`api_group` 为每个 agent 保存 `api_id` 绑定：

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

说明：

- 如果 API 组仍被 story draft 或 session 引用，删除返回 `conflict`

## 4. preset

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `preset.create` | 否 | `preset` | 创建预设 |
| `preset.get` | 否 | `preset` | 获取单个预设 |
| `preset.list` | 否 | `presets_listed` | 列出预设 |
| `preset.update` | 否 | `preset` | 更新预设 |
| `preset.delete` | 否 | `preset_deleted` | 删除预设 |

`preset` 当前支持的每 agent 常用生成参数：

- `temperature`
- `max_tokens`

说明：

- 如果 preset 仍被 story draft 或 session 引用，删除返回 `conflict`
- 未来还可以继续往 preset 里扩展别的 agent 参数

## 5. schema

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

## 6. player_profile

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

## 7. character

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

## 7. story_resources

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

## 8. story

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `story.generate_plan` | 否 | `story_planned` | 调用 Planner 生成可编辑剧本 |
| `story.generate` | 否 | `story_generated` | 兼容封装：内部执行 `story_draft.start -> continue* -> finalize` |
| `story.get` | 否 | `story` | 获取 story 详情 |
| `story.update` | 否 | `story` | 更新 story 元数据 |
| `story.update_graph` | 否 | `story` | 替换整个 story graph，包括节点的 `on_enter_updates` |
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
- 可选 `api_group_id`
- 可选 `preset_id`

如果未显式传入，而后端中存在可用资源，则按 id 排序后自动选择第一个 `api_group`
和第一个 `preset`。

`story.update` 输入：

- `story_id`
- `display_name`

`story.update_graph` 输入：

- `story_id`
- `graph`

## 9. story_draft

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `story_draft.start` | 否 | `story_draft` | 启动分段 Architect 生成，并创建第一段 partial graph |
| `story_draft.get` | 否 | `story_draft` | 获取 draft 详情，包括当前 partial graph |
| `story_draft.list` | 否 | `story_drafts_listed` | 列出 draft 摘要 |
| `story_draft.update_graph` | 否 | `story_draft` | 替换当前 partial graph，包括节点的 `on_enter_updates` |
| `story_draft.continue` | 否 | `story_draft` | 继续生成下一个 outline section，并合并进 draft |
| `story_draft.finalize` | 否 | `story_generated` | 校验完成后的 draft，并创建最终 story |
| `story_draft.delete` | 否 | `story_draft_deleted` | 删除 draft |

说明：

- draft 生成按大纲 section 推进，而不是固定节点数切片。
- partial graph 始终保存在服务端 `story_draft` 对象中，客户端不需要回传已生成节点。
- `story_draft.update_graph` 会替换未 finalized draft 的 `partial_graph`，其中也包含节点的 `on_enter_updates`。
- `story.generate` 仍保留给旧调用方；新的调用方应优先使用 `story_draft.*`。
- `story.generate_plan`、`story.generate`、`story_draft.start` 都支持可选
  `api_group_id` / `preset_id`；省略时会自动选择首个可用资源。

## 10. session

| 方法 | session_id | 返回 | 流式 |
| --- | --- | --- | --- |
| `session.get` | 是 | `session` | 否 |
| `session.update` | 是 | `session` | 否 |
| `session.list` | 否 | `sessions_listed` | 否 |
| `session.delete` | 是 | `session_deleted` | 否 |
| `session.run_turn` | 是 | `turn_stream_accepted` / `turn_completed` | 是 |
| `session.suggest_replies` | 是 | `suggested_replies` | 否 |
| `session.set_player_profile` | 是 | `session` | 否 |
| `session.update_player_description` | 是 | `player_description_updated` | 否 |
| `session.get_runtime_snapshot` | 是 | `runtime_snapshot` | 否 |
| `session.get_variables` | 是 | `session_variables` | 否 |
| `session.update_variables` | 是 | `session_variables` | 否 |
| `session.get_config` | 是 | `session_config` | 否 |
| `session.update_config` | 是 | `session_config` | 否 |

说明：

- `session.update` 只更新 session 的 `display_name`
- `session.suggest_replies` 按需生成玩家候选回复，不写入 `history`
- `session.suggest_replies` 默认返回 3 条，可通过 `limit` 请求 `2..=5` 条
- `session.start_session` 和 `session.get` 返回的 `session` 详情现在会带：
  - `created_at_ms`
  - `updated_at_ms`
  - `history`
- `session.list` 返回的摘要会带：
  - `created_at_ms`
  - `updated_at_ms`
- `history` 按时间顺序保存会话 transcript，当前包含：
  - `player_input`
  - `narration`
  - `dialogue`
  - `action`
- `history` 现在由独立的 `session_message` 记录聚合生成，`session.get` 返回的是排好序的消息列表
- `session.set_player_profile` 只切换当前玩家设定，不切换 `player_state`
- `session.update_player_description` 会清空 `player_profile_id`，改成直接使用手动描述
- `session.get_variables` 返回当前可变的对话变量：
  - `custom`
  - `player_state`
  - `character_state`
- `session.update_variables` 用一个 `StateUpdate` 修改这三类变量
- `session.update_variables` 会拒绝非变量类 op，例如：
  - `SetCurrentNode`
  - `SetActiveCharacters`
  - `AddActiveCharacter`
  - `RemoveActiveCharacter`
- `session.get_config` 返回当前 session 绑定的：
  - `api_group_id`
  - `preset_id`
- `session.update_config` 用于更新这两个绑定；未提供的字段保持原值

## 11. session_message

| 方法 | session_id | 返回 | 流式 |
| --- | --- | --- | --- |
| `session_message.create` | 是 | `session_message` | 否 |
| `session_message.get` | 是 | `session_message` | 否 |
| `session_message.list` | 是 | `session_messages_listed` | 否 |
| `session_message.update` | 是 | `session_message` | 否 |
| `session_message.delete` | 是 | `session_message_deleted` | 否 |

说明：

- message CRUD 只修改 transcript 数据
- 编辑或删除消息不会重放历史，也不会修改 session snapshot
- 手动 `create` 会把消息追加到当前 transcript 末尾
- `session.get.history` 和 `session_message.list` 使用同一套有序消息结构

## 12. config

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `config.get_global` | 否 | `global_config` | 获取当前兜底使用的 `api_group_id` / `preset_id` |

说明：

- `config.get_global` 在尚未初始化时仍会成功返回
- 此时 `api_group_id` 和 `preset_id` 都为 `null`
- 若存在资源，则返回按 id 排序后的第一个 `api_group` 和第一个 `preset`

## 13. dashboard

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `dashboard.get` | 否 | `dashboard` | 获取 dashboard 聚合摘要 |

`dashboard` 包含：

- `health`
- `counts`
- `global_config`
- `recent_stories`
- `recent_sessions`

说明：

- `dashboard.global_config.api_group_id` 和 `dashboard.global_config.preset_id` 在未初始化时同样可能为 `null`
- 在这种状态下，浏览类接口仍可用，但需要 agent 的接口会返回“LLM 配置尚未初始化”错误
