# SillyStage 端到端流程

本文描述一个 story 从零开始被导入、生成、启动，再到玩家参与对话和状态保存、切换的完整过程。这里不展开每个请求的字段细节，字段结构以 `docs/zh/api/spec.md` 和 `docs/zh/api/reference.md` 为准。

## 1. 总览

系统当前分成几层：

- `ss-protocol`：定义请求、响应、流式事件的结构。
- `ss-handler`：接收协议请求，执行业务操作。
- `ss-engine`：负责 story 生成、session 运行和 agent 编排。
- `ss-store`：负责持久化角色卡、资源、story、session 和配置。
- `ss-server`：提供 HTTP/SSE 传输。
- `ss-app`：启动整个后端应用。

从外部看，当前主要入口是：

- `POST /rpc`
- `GET /healthz`

其中普通请求返回 JSON-RPC 响应，`session.run_turn` 则返回 SSE 流。

## 2. 从资源到 story 的主流程

### 2.1 导入角色卡

角色卡文件后缀为 `.chr`，本质上是一个打包文件，内部包含：

- `manifest.json`
- `content.json`
- `cover.<ext>`

上传采用分段上传流程：

1. 调用 `upload.init`
2. 多次调用 `upload.chunk`
3. 调用 `upload.complete`

上传完成后，后端会解析 `.chr`，把角色卡内容和封面写入 `ss-store`，并返回一个 `character_id`。后续所有资源和 story 都只引用这个 `character_id`，不再重复上传完整角色卡内容。

如果需要查看或管理角色卡，可继续使用：

- `character.get`
- `character.list`
- `character.delete`

### 2.2 创建 story resources

拿到角色卡 ID 之后，客户端调用 `story_resources.create` 创建一份 story 生成资源。这个资源通常包含：

- `story_concept`
- `character_ids`
- `world_state_schema_seed`
- `player_state_schema_seed`
- 可选 `planned_story`

这里的 `story resources` 可以理解为“生成 story 之前的原材料”。它还不是一个可运行的 story，只是后续交给 `Planner` 和 `Architect` 的输入集合。

创建之后，可以通过这些请求继续管理：

- `story_resources.get`
- `story_resources.list`
- `story_resources.update`
- `story_resources.delete`

### 2.3 可选：先生成可编辑剧本

如果希望先得到一份更容易人工修改、也更适合 `Architect` 读取的剧本文本，可以调用：

- `story.generate_plan`

这个请求会触发 `Planner`。它读取 `story resources`，输出一份纯文本的 `planned_story`。

典型用法是：

1. 先调用 `story.generate_plan`
2. 取回 `story_script`
3. 客户端人工编辑
4. 用 `story_resources.update` 把编辑后的 `planned_story` 回写到同一个 `resource_id`

这一步是可选的。如果不需要中间剧本，可以直接进入下一步。

### 2.4 生成 story

调用：

- `story.generate`

这个请求会读取指定的 `resource_id`，并交给 `Architect`。`Architect` 当前会生成：

- `graph`
- `world_state_schema`
- `player_state_schema`
- `introduction`

生成成功后，后端会把结果作为一个新的 `story` 对象保存到 `ss-store`，并返回：

- `story_id`
- 生成结果预览

从这一步开始，story 已经是一个可以被拿来启动 session 的对象了。

如果需要查看或管理 story，可使用：

- `story.get`
- `story.list`
- `story.delete`

注意：

- 删除 resources 前，如果它已经被用于生成 story，应返回冲突。
- 删除 story 前，如果还有 session 依赖它，也应返回冲突。

## 3. 从 story 到 session

### 3.1 启动一个新 session

调用：

- `story.start_session`

输入至少包括：

- `story_id`
- `player_description`

这里的 `player_description` 是运行时传入的玩家描述，不属于 `story resources`。它描述的是这次具体游玩中的玩家形象、背景或行为风格。

启动 session 时，后端会：

1. 从 `ss-store` 读取 `story`
2. 读取 story 对应的角色卡和 schema
3. 构造初始 `RuntimeState`
4. 创建 `session`
5. 把初始快照写入 `ss-store`

最终返回：

- `session_id`
- `snapshot`
- 当前 story 的角色摘要

同一个 story 可以创建多个 session。每个 session 都是独立存档，互不影响。

### 3.2 列出、读取、删除 session

session 创建之后，可以通过以下请求操作：

- `session.get`
- `session.list`
- `session.delete`

这里没有单独的“切换 session”接口。切换的含义就是：后续请求改为携带另一个 `session_id`。

## 4. 玩家参与对话的运行流程

### 4.1 发起一轮对话

玩家每输入一句话，客户端调用：

- `session.run_turn`

这个请求是流式请求。HTTP 层会返回 `text/event-stream`。第一帧是 `ack`，后续是 `message` 事件，每一帧都承载协议 JSON。

### 4.2 引擎内部如何处理这一轮

当前一轮 turn 的顺序固定为：

1. 玩家输入写入 shared memory
2. `Keeper` 处理 `AfterPlayerInput`
3. `Director` 规划这一轮的 beats
4. 按顺序执行 `Narrator` 和 `Actor`
5. `Keeper` 再次处理 `AfterTurnOutputs`
6. 把新的 `RuntimeSnapshot` 回写到 store

也就是：

`用户输入 -> Keeper -> Director -> Actors/Narrator -> Keeper`

这里几个关键点是：

- `Actor` 负责角色表现
- `Narrator` 负责环境和结果描述
- `Keeper` 负责把已经发生的事实整理成状态更新
- `Director` 负责安排这一轮要发生什么

### 4.3 客户端会收到什么流式事件

`session.run_turn` 当前会逐步发出事件，例如：

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

最后会以两种方式之一结束：

- `completed`
- `failed`

其中 `completed` 帧会带最终聚合结果，前端不需要自己把所有增量事件重新拼装成完整 turn 结果。

## 5. 运行时状态如何保存

### 5.1 保存哪些对象

当前系统会把这些对象保存到 `ss-store`：

- 角色卡
- story resources
- generated stories
- sessions
- global config

一个 session 的核心持久化内容是：

- `session_id`
- `story_id`
- `RuntimeSnapshot`
- session 配置模式和 agent API 配置

### 5.2 如何恢复和继续对话

当新的 `session.run_turn` 请求到来时，后端不会长期持有一个常驻 `Engine` 实例，而是会：

1. 通过 `session_id` 从 `ss-store` 读取 session record
2. 取出 `RuntimeSnapshot`
3. 再结合对应的 story、角色卡和 schema
4. 重建 `RuntimeState`
5. 创建临时 `Engine`
6. 执行这一轮
7. 将新的 snapshot 写回 store

这意味着：

- 服务重启后，session 依然可以恢复
- 多个 story、多个 session 可以并存
- 切换对话本质上就是读取另一个 `session_id`

## 6. 配置如何影响 story 生成和运行

当前每个 agent 都可以绑定自己的 LLM API 配置，例如：

- planner
- architect
- director
- actor
- narrator
- keeper

配置分成三层：

- global config
- session config
- request override

优先级从高到低为：

`request override > session config / global config`

session 还可以配置为两种模式：

- `UseGlobal`
- `UseSession`

含义是：

- `UseGlobal`：这个 session 每次都读取当前全局配置
- `UseSession`：这个 session 固定使用自己的独立配置

相关请求包括：

- `config.get_global`
- `config.update_global`
- `session.get_config`
- `session.update_config`

## 7. 一条完整用户旅程

把上面的步骤串起来，一次完整流程通常是：

1. 上传多个 `.chr` 角色卡
2. 拿到多个 `character_id`
3. 创建 `story resources`
4. 可选生成 `planned_story`
5. 可选修改 `story resources`
6. 调用 `story.generate`
7. 拿到 `story_id`
8. 调用 `story.start_session`
9. 拿到 `session_id`
10. 玩家反复调用 `session.run_turn`
11. 如有需要，调用 `session.update_player_description`
12. 如有需要，调用 `session.get_runtime_snapshot`
13. 通过 `session.list` 找回历史对话并继续

## 8. 前端接入时最重要的三个点

- 普通请求统一走 `POST /rpc`
- `session.run_turn` 是流式请求，HTTP 返回 SSE
- 切换 story 或切换存档，本质上是切换 `story_id` 或 `session_id`，不是切换一套新的后端实例

如果要看每个请求和响应的精确字段，请继续阅读：

- `docs/zh/api/spec.md`
- `docs/zh/api/reference.md`
