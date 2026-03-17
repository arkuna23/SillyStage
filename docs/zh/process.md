# 运行流程

本文档描述当前系统从资源导入到玩家参与对话的完整流程。

## 1. 准备基础资源

### 1.1 配置 API、API 组和预设

先创建一个或多个可复用的 `api` 资源：

- `api.create`
- `api.list`
- `api.update`
- `api.delete`

`api` 保存一份连接定义：

- `provider`
- `base_url`
- `api_key`
- `model`

先创建一个或多个 `api_group` 资源：

- `api_group.create`
- `api_group.list`
- `api_group.update`
- `api_group.delete`

`api_group` 为每个 agent 保存 `api_id` 绑定：

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

再创建一个或多个 `preset` 资源：

- `preset.create`
- `preset.list`
- `preset.update`
- `preset.delete`

`preset` 为每个 agent 保存生成参数。当前字段包括：

- `temperature`
- `max_tokens`

系统允许在没有任何 `api_group` 和 `preset` 的情况下启动。此时浏览和配置接口可用，
但真正需要调用 agent 的接口会返回“LLM 配置尚未初始化”错误。

如果请求没有显式传 `api_group_id` 或 `preset_id`，而后端中存在可用资源，那么会按 id
排序后自动选择第一个 `api_group` 和第一个 `preset`。

### 1.2 创建 schema 资源

再创建通用 `schema` 资源：

- 角色私有状态 schema
- player state schema
- world state schema seed

通过：

- `schema.create`
- `schema.list`
- `schema.get`

当前 schema 本体包含：

- `schema_id`
- `display_name`
- `tags`
- `fields`

`fields` 现在还可以通过 `enum_values` 约束标量字段的允许值，这样既方便后端校验，也能让
LLM 更新变量时更稳定地落在预期取值集合内。

## 2. 准备玩家设定

玩家设定现在是独立资源 `player_profile`，可以有多个。

字段：

- `player_profile_id`
- `display_name`
- `description`

接口：

- `player_profile.create`
- `player_profile.list`
- `player_profile.get`
- `player_profile.update`
- `player_profile.delete`

一个 session 同时只激活一个玩家设定，但系统中可以保存很多个 profile 供切换。

## 3. 导入角色卡

角色卡有两种创建路径。

### 3.1 导入 `.chr`

流程：

1. `POST /upload/character:{character_id}/archive`

请求体是原始 `.chr` 字节，不使用 JSON-RPC，也不使用 base64。导入完成后，服务端会
解析归档、在内部保存封面，并生成一个 `character` 资源。

### 3.2 直接创建角色卡

流程：

1. `character.create`
2. 可选 `POST /upload/character:{character_id}/cover`

角色内容现在只保存：

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`

也就是说，角色卡通过 `schema_id` 引用自己的私有状态 schema，而不是内联 schema 内容。
角色 payload 现在会暴露 `cover_file_name` 和 `cover_mime_type`；实际封面字节通过
`GET /download/character:{character_id}/cover` 获取。

## 4. 创建 story resources

有了角色卡和 schema 之后，创建 `story_resources`：

- `story_resources.create`

主要字段：

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `planned_story`

这里引用的都是资源 id，而不是内联对象。

## 5. 可选：先做 Planner

如果需要先生成一份更容易修改的剧本草案：

- `story.generate_plan`

这个步骤会从 `story_resources` 读取故事概念和角色信息，生成可编辑的文本稿。  
如果用户修改后想继续生成，可以再用：

- `story_resources.update`

把 `planned_story` 写回。

## 6. 生成 story

推荐流程：

- `story_draft.start`
- `story_draft.continue`
- `story_draft.finalize`

兼容流程：

- `story.generate`

推荐的 draft 流程如下：

1. `story_draft.start` 先读取 `story_resources`
2. 如果缺少 `planned_story`，服务端会先内部执行一次 `story.generate_plan`
3. `Architect` 只生成第一段大纲对应的节点，以及初始 schema 和 introduction
4. 服务端把 partial graph、进度，以及调用方传入的 `common_variables` 一起保存到 `story_draft`
5. 每次 `story_draft.continue` 再生成下一段 section，并合并进同一个 draft
6. `story_draft.finalize` 对合并后的图做校验，再创建最终 `story`，并继承 draft 中的 `common_variables`

`story.generate` 仍然保留，作为一次性封装，内部就是跑完整个 draft 流程。它同样支持可选的 `common_variables` 输入，并会写入最终 story。

这一阶段会：

1. 读取 `story_resources`
2. 把已生成节点保存在服务端 `story_draft`
3. 后续继续生成时，只把图摘要和当前 section 传给 Architect，而不是把完整旧图反复回放
4. 生成：
   - `graph`
   - world schema 内容
   - player schema 内容
   - `introduction`
5. 把生成出的 schema 内容先落库成独立 `schema`
6. 创建最终 `story`

最终 `story` 保存：

- `story_id`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`
- `common_variables`

## 7. 启动 session

调用：

- `story.start_session`

输入里可以带：

- `story_id`
- 可选 `display_name`
- 可选 `player_profile_id`
- 可选 `api_group_id`
- 可选 `preset_id`

启动后得到一个新的 `session`。

返回的 `session` 详情里仍然有 `history`，但这份 transcript 现在由独立的 `session_message` 记录支撑。

如果前端需要为玩家展示几个可点击的下一句建议，可以调用：

- `session.suggest_replies`

这些建议不会写入 transcript；只有真正提交到 `session.run_turn` 的输入才会进入历史。

如果前端想单独读取或 patch 可变的对话变量，而不是整份 snapshot，也可以调用：

- `session.get_variables`
- `session.update_variables`

如果前端还想固定展示一部分常用变量，应额外读取 `story.common_variables`，再把这些定义
映射到 `session.get_variables` 或 runtime snapshot 返回的实时值上。

## 8. session 内部状态

session 现在同时保存：

- `player_profile_id`
- `player_schema_id`
- `api_group_id`
- `preset_id`
- `snapshot`

其中：

- `player_profile_id` 决定当前生效的玩家设定
- `player_schema_id` 决定玩家状态使用哪一套 schema
- `api_group_id` 决定当前使用哪组 agent 到 `api_id` 的绑定
- `preset_id` 决定当前使用哪组生成参数
- `snapshot` 保存动态状态，包括：
  - `world_state`
  - `turn_index`
  - 当前生效的 `player_description`

注意：

- session 里只有一份 `player_state`
- 切换 `player_profile_id` 不会切换 `player_state`

## 9. 玩家进行对话

玩家每一轮输入通过：

- `session.run_turn`
- `session.suggest_replies`

执行顺序固定为：

1. 用户输入
2. `Keeper`（after player input）
3. `Director`
4. 应用 `Director` 的 `role_actions`
5. `Narrator` / `Actor`
6. `Keeper`（after turn outputs）

返回方式是流式：

- unary `ack`
- `started`
- 多个 `event`
- `completed` 或 `failed`

如果后续要编辑 transcript，则使用：

- `session_message.create`
- `session_message.get`
- `session_message.list`
- `session_message.update`
- `session_message.delete`

这些操作只改 transcript 数据，不会重放历史，也不会修改 session snapshot。

`session.suggest_replies` 读取的历史窗口是最近 8 条 transcript 消息。

`Director` 也可以在第 4 步创建 session 级临时角色。它们会在 beat 执行前被加入当前
active cast，因此可以在同一回合立刻参与表演。这些角色只存在于当前 session 的运行时，
不会修改 story graph。

## 10. 管理 session 临时角色

可以通过下面的接口查看和管理 session 级临时角色：

- `session_character.get`
- `session_character.list`
- `session_character.update`
- `session_character.delete`
- `session_character.enter_scene`
- `session_character.leave_scene`

默认创建入口仍然是 `session.run_turn` 中 `Director` 的 `role_actions.create_and_enter`。

## 11. 切换玩家设定

如果当前 session 想切换到另一个玩家设定：

- `session.set_player_profile`

这个操作会：

- 更新当前 session 的 `player_profile_id`
- 更新当前生效的描述文本
- 保留已有 `player_state`

## 12. 手动覆盖玩家描述

如果不想使用某个现成的玩家设定，而是临时写一段描述：

- `session.update_player_description`

这个操作会：

- 直接覆盖当前 session 的描述文本
- 把 `player_profile_id` 置空

## 13. 查看和修改对话变量

session 变量接口只暴露可变的 `world_state` 变量区，不暴露场景控制字段：

- `custom`
- `player_state`
- `character_state`

`session.update_variables` 只接受变量类 `StateUpdate` op，不允许：

- `SetCurrentNode`
- `SetActiveCharacters`
- `AddActiveCharacter`
- `RemoveActiveCharacter`

## 14. 保存、恢复与切换

系统会把这些对象持久化到 store：

- `api_group`
- `preset`
- `schema`
- `player_profile`
- `character`
- `story_resources`
- `story_draft`
- `story`
- `session`
- `session_character`
- `session_message`

因此可以：

- 用 `story.list` 浏览多个 story
- 用 `session.list` 浏览多个对话
- 后续请求直接带另一个 `session_id`，实现“切换到另一段对话”
