# API 参考

本文档列出当前已经实现的 JSON-RPC 方法和传输层二进制 HTTP 端点。

## 1. binary HTTP

| 路由 | 请求体 | 成功响应 | 说明 |
| --- | --- | --- | --- |
| `POST /upload/{resource_id}/{file_id}` | 原始字节 | `ResourceFilePayload` JSON（`200 OK`） | 可选 `x-file-name`；作为逻辑资源文件的传输适配层 |
| `GET /download/{resource_id}/{file_id}` | 无 | 原始字节（`200 OK`） | 下载一个逻辑资源文件；若存在文件名则使用附件下载 disposition |

当前内置资源文件：

- `character:{character_id}/cover`
- `character:{character_id}/archive`
- `package_import:{import_id}/archive`
- `package_export:{export_id}/archive`

## 2. api

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `api.create` | 否 | `api` | 创建一个可复用 API 定义 |
| `api.get` | 否 | `api` | 获取单个 API 定义 |
| `api.list` | 否 | `apis_listed` | 列出 API 定义 |
| `api.list_models` | 否 | `api_models_listed` | 用一组连接参数探测可用模型 |
| `api.update` | 否 | `api` | 更新 API 定义 |
| `api.delete` | 否 | `api_deleted` | 删除 API 定义 |

`api` 保存一份可复用的连接配置：

- `provider`
- `base_url`
- `api_key`
- `model`

说明：

- 读取接口不返回明文 `api_key`
- `api.list_models` 直接接收 `provider`、`base_url`、`api_key`，不会持久化 `api`
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
| `preset_entry.create` | 否 | `preset_entry` | 给某个 agent 模块新增自定义提示词条目 |
| `preset_entry.update` | 否 | `preset_entry` | 更新某个模块中的单条提示词条目 |
| `preset_entry.delete` | 否 | `preset_entry_deleted` | 删除某个模块中的单条自定义提示词条目 |
| `preset_preview.template` | 否 | `preset_prompt_preview` | 预览编译后的提示词模板，并保留上下文占位符 |
| `preset_preview.runtime` | 视情况而定 | `preset_prompt_preview` | 基于真实资源、draft 或 session 上下文预览编译后的提示词 |

`preset` 当前支持每个 agent 的常用生成参数，以及模块化提示词设定：

- `temperature`
- `max_tokens`
- 可选 `extra`
- `modules`

每个 `module` 包含：

- `module_id`
- `display_name`
- `message_role`
- `order`
- `entries`

内置 `module_id` 当前为：

- `role`
- `task`
- `static_context`
- `dynamic_context`
- `output`

也允许自定义模块 id，按普通字符串存储。

每个 `entries` 条目包含：

- `entry_id`
- `display_name`
- `kind`
- `enabled`
- `order`
- `required`
- 可选 `text`
- 可选 `context_key`

`kind` 当前支持：

- `built_in_text`
- `built_in_context_ref`
- `custom_text`

行为说明：

- `preset.create`、`preset.get`、`preset.update` 使用完整模块结构
- `preset.list` 返回摘要；每个 agent 会给出 `module_count`、`entry_count`，以及不带 `text/context_key` 的模块摘要
- 允许只提交需要覆盖的模块或条目；后端会按 agent 内置模板做规范化并补齐缺失的 built-in 项
- 模块按 `order`、`module_id` 排序；条目按 `order`、`entry_id` 排序
- `built_in_text` 和 `built_in_context_ref` 来自后端默认模板，不能通过 `preset_entry.create` 新增，也不能通过 `preset_entry.delete` 删除
- `preset_entry.create` 仅创建 `custom_text` 条目，并落在已有的 `agent + module_id` 下
- `preset_entry.update` 对 `custom_text` 可修改 `display_name`、`text`、`enabled`、`order`；对 built-in 条目仅允许改 `enabled`、`order`
- 启用条目最终会被编译成一条 system message 和一条 user message
- `message_role` 用于决定模块进入 system 还是 user message
- 最终 prompt 会保留模块标题，但不会输出 entry id 和 entry 显示名
- `context_key` 只用于 `built_in_context_ref`；`custom_text` 使用 `text`
- `preset_preview.template` 会把未解析的 `context_ref` 渲染成 `<context:story_concept>` 这种占位符
- `preset_preview.runtime` 会返回真实编译后的 entry 文本；如果传了 `module_id`，则只预览该模块，不返回完整 system/user 组合
- `preset_preview.runtime` 的上下文来源规则：
  - planner 和 architect 的 `graph` 模式要求传 `resource_id`
  - architect 的 `draft_init` / `draft_continue` 要求传 `draft_id`
  - director / actor / narrator / keeper / replyer 要求顶层 `session_id`
  - actor 的运行期预览还要求传 `character_id`
- architect 预览必须指定 `architect_mode = graph | draft_init | draft_continue`
- 预览响应会返回 `preview_kind`、`message_role`、`messages` 和 `unresolved_context_keys`
- `messages` 按顺序组织为 `message -> module -> entry`
- `module` 返回 `module_id`、`display_name`、`order` 和有序 `entries`
- `entry` 返回 `entry_id`、`display_name`、`kind`、`order`、`source = preset | synthetic` 和 `compiled_text`
- 预览正文只放在 entry 级别；模块标题和完整 prompt 由前端自行拼接

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
- schema 字段遵循 `StateFieldSchema`：
  - `value_type`
  - 可选 `default`
  - 可选 `description`
  - 标量字段可选 `enum_values`，支持 `bool`、`int`、`float`、`string`
- 若 schema 仍被角色卡、resources、story 或 session 引用，删除返回 `conflict`

## 6. lorebook

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `lorebook.create` | 否 | `lorebook` | 创建 lorebook |
| `lorebook.get` | 否 | `lorebook` | 获取单个 lorebook |
| `lorebook.list` | 否 | `lorebooks_listed` | 列出 lorebook |
| `lorebook.update` | 否 | `lorebook` | 更新 lorebook 基础元数据 |
| `lorebook.delete` | 否 | `lorebook_deleted` | 删除 lorebook |

`lorebook` 保存：

- `lorebook_id`
- `display_name`
- `entries`

说明：

- `lorebook.create` 可直接带初始 `entries`
- `lorebook.update` 当前只更新基础元数据，例如 `display_name`
- 若 lorebook 仍被 `story_resources` 引用，`lorebook.delete` 返回 `conflict`

## 7. lorebook_entry

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `lorebook_entry.create` | 否 | `lorebook_entry` | 创建 lorebook 条目 |
| `lorebook_entry.get` | 否 | `lorebook_entry` | 获取单个 lorebook 条目 |
| `lorebook_entry.list` | 否 | `lorebook_entries_listed` | 列出某个 lorebook 的条目 |
| `lorebook_entry.update` | 否 | `lorebook_entry` | 更新 lorebook 条目 |
| `lorebook_entry.delete` | 否 | `lorebook_entry_deleted` | 删除 lorebook 条目 |

条目字段：

- `entry_id`
- `title`
- `content`
- `keywords`
- `enabled`
- `always_include`

说明：

- `lorebook_entry.*` 都通过 `lorebook_id` 作用于某个 lorebook
- 创建时 `enabled` 默认为 `true`

## 8. player_profile

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

## 9. character

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `character.create` | 否 | `character_created` | 直接从请求数据创建角色卡 |
| `character.get` | 否 | `character` | 获取完整角色内容 |
| `character.update` | 否 | `character` | 更新角色内容 |
| `character.list` | 否 | `characters_listed` | 获取角色摘要列表 |
| `character.delete` | 否 | `character_deleted` | 删除角色卡 |

角色内容字段：

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`
- `tags`
- `folder`

说明：

- `schema_id` 引用角色私有状态 schema
- `tags` 是角色卡的用户标签列表
- `folder` 是角色卡的文件夹分组；空字符串表示未分组
- 角色摘要和详情 payload 还会包含：
  - `tags`
  - `folder`
  - `cover_file_name`
  - `cover_mime_type`
- 封面字节通过 `GET /download/character:{character_id}/cover` 获取
- `.chr` 导入与导出使用：
  - `POST /upload/character:{character_id}/archive`
  - `GET /download/character:{character_id}/archive`
- 封面上传使用 `POST /upload/character:{character_id}/cover`

## 10. story_resources

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `story_resources.create` | 否 | `story_resources_created` | 创建 story resources |
| `story_resources.get` | 否 | `story_resources` | 获取单个 resources |
| `story_resources.list` | 否 | `story_resources_listed` | 列出 resources |
| `story_resources.update` | 否 | `story_resources_updated` | 更新 resources |
| `story_resources.delete` | 否 | `story_resources_deleted` | 删除 resources |

字段：

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `lorebook_ids`
- `planned_story`

说明：

- `lorebook_ids` 引用生成时使用的 lorebook，可以为空

## 11. story

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `story.generate_plan` | 否 | `story_planned` | 调用 Planner 生成可编辑剧本 |
| `story.create` | 否 | `story_generated` | 直接用调用方传入的 graph 创建 story |
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
- `common_variables`

`story.create` 输入：

- `resource_id`
- 可选 `display_name`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`
- 可选 `common_variables`

`story.generate` 输入：

- `resource_id`
- 可选 `display_name`
- 可选 `api_group_id`
- 可选 `preset_id`
- 可选 `common_variables`

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
- 可选 `display_name`
- 可选 `common_variables`

每个 `common_variables` 条目包含：

- `scope`
- `key`
- `display_name`
- 可选 `character_id`
- 可选 `pinned`，默认是 `true`

`story`、`stories_listed`、`story_generated` 返回都会带 `common_variables`。

`story.update_graph` 输入：

- `story_id`
- `graph`

`story.create` 对传入 graph 使用与 `story.update_graph` 相同的校验规则。

## 12. story_draft

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
- `story_draft.start` 支持可选 `common_variables`。
- `story_draft` 详情响应会返回 `common_variables`。
- `story_draft.finalize` 会把 draft 中的 `common_variables` 复制到最终 `story`。
- `story_draft.update_graph` 会替换未 finalized draft 的 `partial_graph`，其中也包含节点的 `on_enter_updates`。
- `story.generate` 仍保留给旧调用方；新的调用方应优先使用 `story_draft.*`。
- `story.generate_plan`、`story.generate`、`story_draft.start` 都支持可选
  `api_group_id` / `preset_id`；省略时会自动选择首个可用资源。

## 13. session

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
- `session.suggest_replies` 当前会使用最近 8 条 transcript 消息作为回复建议上下文
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
- `Director` 现在可以在 turn 规划阶段通过 `role_actions` 创建 session 级临时角色
- 这些临时角色会在 beat 执行前先被创建并加入当前场景，因此可以在同一回合立即参与表演
- session 级临时角色不会修改底层 story graph，也不会持久化到当前 session 之外

## 14. session_character

| 方法 | session_id | 返回 | 流式 |
| --- | --- | --- | --- |
| `session_character.get` | 是 | `session_character` | 否 |
| `session_character.list` | 是 | `session_characters_listed` | 否 |
| `session_character.update` | 是 | `session_character` | 否 |
| `session_character.delete` | 是 | `session_character_deleted` | 否 |
| `session_character.enter_scene` | 是 | `session_character` | 否 |
| `session_character.leave_scene` | 是 | `session_character` | 否 |

说明：

- session character 是只存在于当前 session 的临时运行时角色
- 默认创建入口是 `session.run_turn` 中 `Director` 的 `role_actions.create_and_enter`
- `session_character.enter_scene` 和 `session_character.leave_scene` 只修改该角色是否处于当前 active cast
- session character 不属于 `story`、`story_draft` 或 `character`

## 15. session_message

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

## 16. config

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `config.get_global` | 否 | `global_config` | 获取当前兜底使用的 `api_group_id` / `preset_id` |

说明：

- `config.get_global` 在尚未初始化时仍会成功返回
- 此时 `api_group_id` 和 `preset_id` 都为 `null`
- 若存在资源，则返回按 id 排序后的第一个 `api_group` 和第一个 `preset`

## 17. dashboard

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

## 18. data_package

| 方法 | session_id | 返回 | 说明 |
| --- | --- | --- | --- |
| `data_package.export_prepare` | 否 | `data_package_export_prepared` | 构建临时 ZIP 导出槽位，并返回可下载的归档引用 |
| `data_package.import_prepare` | 否 | `data_package_import_prepared` | 分配临时 ZIP 导入槽位，并返回可上传的归档引用 |
| `data_package.import_commit` | 否 | `data_package_import_committed` | 校验并原子导入已上传的 ZIP 归档 |

支持打包的资源类型：

- `preset`
- `schema`
- `lorebook`
- `player_profile`
- 带可选封面字节的 `character`
- `story_resources`
- `story`

`data_package.export_prepare` 输入：

- 可选 `preset_ids`
- 可选 `schema_ids`
- 可选 `lorebook_ids`
- 可选 `player_profile_ids`
- 可选 `character_ids`
- 可选 `story_resource_ids`
- 可选 `story_ids`
- 可选 `include_dependencies`，默认是 `true`

导出说明：

- 至少需要选择一个 id
- 当 `include_dependencies = true` 时，story 会自动带上其引用的 `story_resources`、story/player/world schema、character schema、character、lorebook
- 导出的 character 如果有封面，会一并带上封面字节
- 生成好的 ZIP 通过 `GET /download/package_export:{export_id}/archive` 下载

导入说明：

- 先调用 `data_package.import_prepare`，再通过 `POST /upload/package_import:{import_id}/archive` 上传字节，最后调用 `data_package.import_commit`
- 导入是全有或全无
- 当前冲突策略是严格失败：只要包内任一 id 已存在，就返回 `conflict`
- 导入不会覆盖已有资源，也不会重映射 id
