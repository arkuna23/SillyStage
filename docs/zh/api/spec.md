# API 协议结构

本文档描述当前 `ss-protocol` 的结构约定，重点说明消息封装和资源模型。具体方法清单见 [reference.md](./reference.md)。

## 1. 传输模型

后端同时使用 JSON-RPC 2.0 作为协议请求/响应封装、独立的服务端事件消息作为流式输出，
以及专门的二进制 HTTP 路由来传输文件。

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

除了 `session.run_turn` 之外，session 绑定接口也可以返回普通 unary JSON-RPC。当前一个典型例子是：

- `session.suggest_replies`
  - 需要顶层 `session_id`
  - `params`:
    - `limit?: number`
  - 返回：
    - `type = "suggested_replies"`
    - `replies: [{ reply_id, text }]`
  - 说明：
    - 只生成建议，不写入 session transcript
    - 默认返回 3 条，允许 `2..=5`

### 1.4 二进制文件传输

`/upload/{resource_id}/{file_id}` 与 `/download/{resource_id}/{file_id}` 下的路由不使用
JSON-RPC envelope。

- 上传请求体直接发送原始字节。
- `resource_id + file_id` 是协议层的文件标识。
- `x-file-name` 是上传路由的可选请求头。
- 下载响应直接返回原始字节，并通过 HTTP `Content-Type` 标明类型。
- 二进制路由出错时会返回普通 `ErrorPayload` JSON，并使用对应的 HTTP 状态码。
- 当前内置资源文件：
  - `character:{character_id}/cover`
  - `character:{character_id}/archive`
  - `package_import:{import_id}/archive`
  - `package_export:{export_id}/archive`

## 2. 资源模型

### 2.1 `api`

`api` 是可持久化的可复用连接定义。

字段：

- `api_id`
- `display_name`
- `provider`
- `base_url`
- `api_key`
- `model`

读取接口不会返回明文 `api_key`，而是返回：

- `has_api_key`
- `api_key_masked`

辅助方法：

- `api.list_models` 接收 `provider`、`base_url`、`api_key`
- 返回 `provider`、规范化后的 `base_url` 和 `models: string[]`
- 不会创建或更新已保存的 `api`

### 2.2 `api_group`

`api_group` 是可持久化的每-agent `api_id` 绑定集合。

字段：

- `api_group_id`
- `display_name`
- `bindings`

`bindings` 为每个 agent 保存一个 `api_id`：

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

### 2.3 `preset`

`preset` 是可持久化的每-agent 生成参数集合，同时也保存 agent 级提示词条目。

字段：

- `preset_id`
- `display_name`
- `agents`

每个 agent 当前支持：

- `temperature`
- `max_tokens`
- 可选 `director_shared_history_limit`
- 可选 `actor_shared_history_limit`
- 可选 `actor_private_memory_limit`
- 可选 `narrator_shared_history_limit`
- 可选 `replyer_session_history_limit`
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

`preset.create` / `preset.get` / `preset.update` 使用完整模块结构。允许只提交需要覆盖的模块或条目，后端会按 agent 默认模板补齐并规范化。

`preset.list` 返回摘要形态：每个 agent 会额外提供 `module_count`、`entry_count`，以及不带 `text/context_key` 的模块摘要。

新增单条目接口：

- `preset_entry.create`
- `preset_entry.update`
- `preset_entry.delete`

新增提示词预览接口：

- `preset_preview.template`
- `preset_preview.runtime`

说明：

- `preset_entry.create` 仅创建 `custom_text`
- `preset_entry.delete` 仅删除 `custom_text`
- `preset_entry.update` 对 built-in 条目仅允许修改 `enabled` 和 `order`
- `preset_entry.*` 只能作用于已有模块，不会隐式创建模块
- 模块按 `order`、`module_id` 排序；条目按 `order`、`entry_id` 排序
- 启用条目会被编译成一条 system message 和一条 user message
- `message_role` 用于决定模块进入 system 还是 user message
- 最终 prompt 会保留模块标题，但不会输出 entry id 和 entry 显示名
- 历史条数字段是可选的，只对会读取消息历史的 agent 生效；省略时使用后端默认值
- `replyer_session_history_limit` 作用于最近 session 消息窗口；在该窗口内，
  narration 会被压缩为只保留最新一个带 narration 的 turn，其他消息类型保持不变
- `preset_preview.template` 会渲染编译后的提示词，并把未解析的 `context_ref` 保留成
  `<context:story_concept>` 这种占位符
- `preset_preview.runtime` 会渲染结构化的真实 entry 文本；传 `module_id` 时只返回单个模块的编译结果
- architect 预览是按模式区分的，必须指定
  `architect_mode = graph | draft_init | draft_continue`
- 运行期预览的上下文来源规则：
  - planner 和 architect 的 `graph` 模式使用 `resource_id`
  - architect 的 `draft_init` 和 `draft_continue` 使用 `draft_id`
  - director / actor / narrator / keeper / replyer 使用顶层 `session_id`
  - actor 的运行期预览还要求传 `character_id`
- 预览响应返回：
  - `preview_kind = template | runtime`
  - `message_role = system | user | full`
  - `messages[]`
  - 每个 `message` 包含 `role` 和有序的 `modules[]`
  - 每个 `module` 包含 `module_id`、`display_name`、`order` 和有序的 `entries[]`
  - 每个 `entry` 包含 `entry_id`、`display_name`、`kind`、`order`、`source` 和 `compiled_text`
  - `unresolved_context_keys: string[]`
- `compiled_text` 只存在于 entry 级别；模块标题和完整 prompt 由客户端自行拼接
- `source = preset | synthetic`，其中 `synthetic` 表示后端注入的附加条目，例如 architect 模式专用 contract

运行时绑定模型现在是 `api_group_id + preset_id`。

如果请求省略其中某个 id，而后端存在可用资源，则按 id 排序后自动选择第一个。

### 2.4 `schema`

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
- `StateFieldSchema` 支持 `value_type`、可选 `default`、可选 `description`，以及可选 `enum_values`。
- `enum_values` 当前只支持标量类型：`bool`、`int`、`float`、`string`。
- 如果配置了 `enum_values`，那么 `default` 也必须类型匹配，并且必须包含在允许值里。

### 2.5 `lorebook`

`lorebook` 是可持久化的可复用世界设定集合。

字段：

- `lorebook_id`
- `display_name`
- `entries`

每个 `entry` 包含：

- `entry_id`
- `title`
- `content`
- `keywords: string[]`
- `enabled`
- `always_include`

说明：

- `lorebook.update` 用于更新基础元数据，例如 `display_name`
- `lorebook_entry.*` 作用于单个 `lorebook` 内的条目
- `keywords` 默认是 `[]`
- `enabled` 默认是 `true`
- `always_include` 默认是 `false`

### 2.6 `player_profile`

`player_profile` 表示“玩家设定”，是独立的可切换资源。

字段：

- `player_profile_id`
- `display_name`
- `description`

说明：

- 一个 story 可以对应多个 player profile。
- 一个 session 同时只激活一个 `player_profile_id`。
- 切换 player profile 不会切换 `player_state`。

### 2.7 `resource_file`

`resource_file` 是公开的、与传输方式无关的二进制文件标识。

字段：

- `resource_id`
- `file_id`
- `file_name`
- `content_type`
- `size_bytes`

说明：

- 对外接口通过 `resource_id + file_id` 标识文件，不暴露内部 blob id。
- `POST /upload/{resource_id}/{file_id}` 返回 `ResourceFilePayload`。
- `GET /download/{resource_id}/{file_id}` 返回该逻辑文件的原始字节。
- 当前内置资源文件：
  - `character:{character_id}/cover`
  - `character:{character_id}/archive`
  - `package_import:{import_id}/archive`
  - `package_export:{export_id}/archive`

#### data package 传输

`data_package.*` 通过临时 `resource_file` 标识完成 ZIP 传输。

- `data_package.export_prepare`
  - 选择一个或多个持久化资源
  - 可选自动带上依赖
  - 返回 `export_id`、`archive`、`contents`
  - ZIP 字节通过 `GET /download/package_export:{export_id}/archive` 下载
- `data_package.import_prepare`
  - 分配一个临时上传槽位
  - 返回 `import_id` 和 `archive`
  - ZIP 字节通过 `POST /upload/package_import:{import_id}/archive` 上传
- `data_package.import_commit`
  - 校验已上传的 ZIP
  - 原子应用包内全部资源
  - 返回导入后的 `contents`

当前冲突策略：

- 包内 id 不能与现有资源重复
- 导入不会覆盖已有资源，也不会重映射 id
- 导入是全有或全无

### 2.8 `character`

角色卡内容通过 `CharacterCardContent` 表示：

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`

角色读取 payload 会在内容之外附带可选封面元数据：

- `cover_file_name`
- `cover_mime_type`

说明：

- 角色卡不再内联 `state_schema`。
- 角色私有 schema 通过 `schema_id` 引用独立 `schema` 资源。
- JSON-RPC payload 中不再内联封面字节。
- 封面字节通过 `GET /download/character:{character_id}/cover` 获取。
- `.chr` 的导入与导出通过以下路由完成：
  - `POST /upload/character:{character_id}/archive`
  - `GET /download/character:{character_id}/archive`

### 2.9 `story_resources`

`story_resources` 是生成 story 前的输入资源集合。

字段：

- `resource_id`
- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `lorebook_ids`
- `planned_story`

说明：

- `character_ids` 只引用已上传或已创建的角色卡。
- 两个 schema seed 都是可选 id。
- `lorebook_ids` 只引用已存在的 lorebook，也可以为空。
- `planned_story` 是可选的 planner 文本。

### 2.10 `story`

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

### 2.11 `session`

session 绑定一个 story 和一份运行时快照。

字段：

- `session_id`
- `story_id`
- `display_name`
- `player_profile_id`
- `player_schema_id`
- `snapshot`
- `history`
- `created_at_ms`
- `updated_at_ms`
- `config`

说明：

- `player_profile_id` 可为空；为空表示当前 session 使用手动覆盖后的 `player_description`。
- `player_schema_id` 固定引用该 session 使用的玩家状态 schema。
- `snapshot` 保存当前动态运行状态，包括 `world_state`、`turn_index`，以及当前生效的 `player_description` 文本。
- `history` 保存可见 transcript，按时间顺序记录：
  - 玩家输入
  - 旁白
  - 角色可见动作
  - 角色台词
- `history` 由独立的 `session_message` 记录聚合生成
- `created_at_ms` / `updated_at_ms` 使用 Unix 毫秒时间戳。
- session 级临时角色会独立于 story graph 存储，并且只在当前 session 运行时内存在。

## 3. 角色卡文件 `.chr`

`.chr` 是一个 ZIP 容器，固定包含：

- `manifest.json`
- `content.json`
- `cover.<ext>`

`content.json` 使用 `CharacterCardContent`。

导入与导出通过以下路由完成：

- `POST /upload/character:{character_id}/archive`
- `GET /download/character:{character_id}/archive`

更多细节见：

- [../character.md](../character.md)

## 4. 方法族

当前协议按资源分为以下方法族：

- `/upload/{resource_id}/{file_id}` 与 `/download/{resource_id}/{file_id}` 下的二进制文件路由
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

## 5. Session 相关语义

### 5.1 启动 session

`story.start_session` 输入：

- `story_id`
- 可选 `display_name`
- 可选 `player_profile_id`
- 可选 `api_group_id`
- 可选 `preset_id`

如果省略绑定 id，而后端中至少有一个 `api_group` 和一个 `preset`，就按 id 排序后
自动选择各自的第一个。

`story.create` 输入：

- `resource_id`
- 可选 `display_name`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`
- 可选 `common_variables`

`story.generate` 与 draft start 一样，支持以下创建输入：

- `resource_id`
- 可选 `display_name`
- 可选 `api_group_id`
- 可选 `preset_id`
- 可选 `common_variables`

`story.update` 只更新 story 的元数据，当前仅支持：

- `story_id`
- 可选 `display_name`
- 可选 `common_variables`

每个 `common_variables` 条目包含：

- `scope`: `world | player | character`
- `key`
- `display_name`
- 可选 `character_id`
- 可选 `pinned`，默认是 `true`

校验规则：

- `world` 和 `player` 条目不能设置 `character_id`
- `character` 条目必须设置 `character_id`
- `character_id` 必须属于该 story 绑定的角色之一
- `key` 必须存在于对应作用域绑定的 schema 中
- 同一个绑定变量不能重复配置

前端可以把 `story.common_variables` 与 `session.get_variables` 或 runtime snapshot 组合使用，
从而渲染固定展示的变量面板，而不需要硬编码 schema key。

`story.update_graph` 会整体替换已有 story 的 `graph` 字段，其中也包含每个节点的
`on_enter_updates`。
后端会先校验 graph；以下情况会返回 `invalid_request`：

- `start_node` 不存在
- 有 transition 指向不存在的节点
- 存在重复的节点 id

`story.create` 在创建 story 前会使用同样的 graph 校验规则。

### 5.1.1 draft story 生成

`story_draft.*` 是大体量 story 的推荐生成路径，用来避免一次 Architect 调用塞入过多节点。

- `story_draft.start` 创建服务端 draft，并生成第一段
- `story_draft.start` 也可以持久化调用方传入的 `common_variables`
- `story_draft.update_graph` 替换 draft 当前的 `partial_graph`，其中也包含节点的
  `on_enter_updates`
- `story_draft.continue` 继续把下一个 outline section 合并进 partial graph
- `story_draft.finalize` 校验合并后的图，并创建最终 `story`
- `story.generate` 仍保留，作为对整套 draft 流程的兼容封装
- `story_draft.start` 会把当前选中的 `api_group_id` 和 `preset_id` 一起保存进 draft
- draft 详情响应也会带上持久化后的 `common_variables`
- `story_draft.finalize` 会把 draft 中的 `common_variables` 复制到最终 `story`

`story_draft.update_graph` 使用与 `story.update_graph` 相同的 graph 校验规则，并且 finalized draft 不允许再编辑。

### 5.2 切换玩家设定

`session.set_player_profile` 只切换当前 session 的 `player_profile_id` 和生效描述，不切换 `player_state`。

### 5.3 手动覆盖玩家描述

`session.update_player_description` 会直接覆盖当前 session 的描述文本，并把 `player_profile_id` 置空。

### 5.4 更新 session 元数据

`session.update` 只更新 session 的元数据，当前仅支持修改 `display_name`。

### 5.5 读取和修改 session 变量

`session.get_variables` 返回当前 session snapshot 中可变的对话变量：

- `custom`
- `player_state`
- `character_state`

`session.update_variables` 用一个 `StateUpdate` 修改同一组变量。

只允许变量类 op：

- `SetState`
- `RemoveState`
- `SetPlayerState`
- `RemovePlayerState`
- `SetCharacterState`
- `RemoveCharacterState`

像 `SetCurrentNode`、`SetActiveCharacters` 这类场景控制 op 会被拒绝。

### 5.6 管理 session 临时角色

`session_character.*` 用来管理某个 session 内的临时运行时角色。

- 默认创建入口是 `session.run_turn` 中 `Director` 的 `role_actions.create_and_enter`
- 创建出来的 session 临时角色会立刻进入当前场景，并能在同一回合参与表演
- `session_character.enter_scene` 和 `session_character.leave_scene` 只修改角色是否处于当前场景
- session 临时角色不会修改 `story`、`story_draft` 或 `character`

### 5.7 编辑 session transcript 消息

`session_message.*` 用来管理某个 session 的独立 transcript 消息资源。

- `session_message.create` 会把一条可见消息追加到 transcript 末尾
- `session_message.get` 和 `session_message.list` 返回 transcript 消息资源
- `session_message.update` 原地修改 transcript 数据
- `session_message.delete` 从 transcript 中移除一条消息
- `session.suggest_replies` 读取的历史窗口是最近 8 条 transcript 消息

注意：

- transcript 编辑不会重放历史
- transcript 编辑不会修改 `snapshot` 或 `world_state`

## 6. 删除约束

- `schema.delete`: 若 schema 仍被角色卡、resources、story 或 session 引用，则返回 `conflict`
- `player_profile.delete`: 若 profile 仍被 session 引用，则返回 `conflict`
- `character.delete`: 若角色卡仍被 `story_resources` 引用，则返回 `conflict`
- `story.delete`: 若 story 仍有关联 session，则返回 `conflict`
