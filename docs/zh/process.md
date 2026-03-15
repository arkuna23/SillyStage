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

### 3.1 上传 `.chr`

流程：

1. `upload.init`
2. `upload.chunk`
3. `upload.complete`

上传完成后，服务端会解析 `.chr` 并生成一个 `character` 资源。

### 3.2 直接创建角色卡

流程：

1. `character.create`
2. 可选 `character.set_cover`

角色内容现在只保存：

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `schema_id`
- `system_prompt`

也就是说，角色卡通过 `schema_id` 引用自己的私有状态 schema，而不是内联 schema 内容。

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
4. 服务端把 partial graph 和进度保存到 `story_draft`
5. 每次 `story_draft.continue` 再生成下一段 section，并合并进同一个 draft
6. `story_draft.finalize` 对合并后的图做校验，再创建最终 `story`

`story.generate` 仍然保留，作为一次性封装，内部就是跑完整个 draft 流程。

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

执行顺序固定为：

1. 用户输入
2. `Keeper`（after player input）
3. `Director`
4. `Narrator` / `Actor`
5. `Keeper`（after turn outputs）

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

## 10. 切换玩家设定

如果当前 session 想切换到另一个玩家设定：

- `session.set_player_profile`

这个操作会：

- 更新当前 session 的 `player_profile_id`
- 更新当前生效的描述文本
- 保留已有 `player_state`

## 11. 手动覆盖玩家描述

如果不想使用某个现成的玩家设定，而是临时写一段描述：

- `session.update_player_description`

这个操作会：

- 直接覆盖当前 session 的描述文本
- 把 `player_profile_id` 置空

## 12. 查看和修改对话变量

session 变量接口只暴露可变的 `world_state` 变量区，不暴露场景控制字段：

- `custom`
- `player_state`
- `character_state`

`session.update_variables` 只接受变量类 `StateUpdate` op，不允许：

- `SetCurrentNode`
- `SetActiveCharacters`
- `AddActiveCharacter`
- `RemoveActiveCharacter`

## 13. 保存、恢复与切换

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
- `session_message`

因此可以：

- 用 `story.list` 浏览多个 story
- 用 `session.list` 浏览多个对话
- 后续请求直接带另一个 `session_id`，实现“切换到另一段对话”
