# 运行流程

本文档描述当前系统从资源导入到玩家参与对话的完整流程。

## 1. 准备基础资源

### 1.1 配置 LLM API

先创建一个或多个 `llm_api` 资源：

- `llm_api.create`
- `llm_api.list`
- `llm_api.update`
- `llm_api.delete`

这些对象描述可用的大模型入口，例如 OpenAI-compatible API。  
后续 global config 和 session config 只引用 `api_id`。

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

调用：

- `story.generate`

这一阶段会：

1. 读取 `story_resources`
2. 通过 `Architect` 生成：
   - `graph`
   - world schema 内容
   - player schema 内容
   - `introduction`
3. 把生成出的 schema 内容先落库成独立 `schema`
4. 创建最终 `story`

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
- `config_mode`
- 可选 `session_api_ids`

启动后得到一个新的 `session`。

## 8. session 内部状态

session 现在同时保存：

- `player_profile_id`
- `player_schema_id`
- `snapshot`

其中：

- `player_profile_id` 决定当前生效的玩家设定
- `player_schema_id` 决定玩家状态使用哪一套 schema
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

## 12. 保存、恢复与切换

系统会把这些对象持久化到 store：

- `llm_api`
- `schema`
- `player_profile`
- `character`
- `story_resources`
- `story`
- `session`

因此可以：

- 用 `story.list` 浏览多个 story
- 用 `session.list` 浏览多个对话
- 后续请求直接带另一个 `session_id`，实现“切换到另一段对话”

