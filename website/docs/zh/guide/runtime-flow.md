# 运行流程

本文描述当前系统从资源导入到玩家参与对话的完整流程。

## 1. 准备基础资源

先准备三类可复用配置：

- `api`：连接定义，包含 `provider`、`base_url`、`api_key`、`model`
- `api_group`：每个 agent 绑定的 `api_id`
- `preset`：每个 agent 的生成参数和提示词模块

常用方法：

- `api.create` / `api.list` / `api.update` / `api.delete`
- `api_group.create` / `api_group.list` / `api_group.update` / `api_group.delete`
- `preset.create` / `preset.list` / `preset.update` / `preset.delete`

如果请求没有显式传入 `api_group_id` 或 `preset_id`，而后端存在可用资源，系统会按 id 排序并自动选择第一个。

## 2. 创建 schema

再创建可复用的 `schema` 资源，用于：

- 角色私有状态
- 玩家状态
- 世界状态 seed

Schema 通过 `schema.create`、`schema.list`、`schema.get` 管理，字段包括：

- `schema_id`
- `display_name`
- `tags`
- `fields`

## 3. 准备玩家设定

玩家设定是独立资源 `player_profile`，支持多份共存。

字段：

- `player_profile_id`
- `display_name`
- `description`

一个 session 同时只激活一个 profile，但系统里可以保存多个供后续切换。

## 4. 导入或创建角色卡

角色卡有两种路径：

### 4.1 导入 `.chr`

```text
POST /upload/character:{character_id}/archive
```

- 请求体是原始 `.chr` 字节
- 不走 JSON-RPC，也不走 base64
- 服务端会解析归档、保存封面，并创建 `character`

### 4.2 直接创建对象

1. `character.create`
2. 可选 `POST /upload/character:{character_id}/cover`

角色内容当前主要字段：

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`

## 5. 创建 story resources

有了角色卡和 schema 之后，创建 `story_resources`：

- `story_resources.create`

主要字段：

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `lorebook_ids`
- `planned_story`

## 6. 可选：先做 Planner

如果需要一份可编辑的剧本草案，可以先调用：

- `story.generate_plan`

编辑完成后再通过 `story_resources.update` 写回 `planned_story`。

## 7. 生成 story

推荐流程：

- `story_draft.start`
- `story_draft.continue`
- `story_draft.finalize`

兼容的一次性封装：

- `story.generate`

推荐 draft 流程会：

1. 读取 `story_resources`
2. 必要时先内部生成 `planned_story`
3. 由 `Architect` 先生成第一段 partial graph、初始 schema 和 `introduction`
4. 把 partial graph 与 `common_variables` 保存在服务端 `story_draft`
5. 每次 `story_draft.continue` 追加一个大纲 section
6. `story_draft.finalize` 校验完整图并创建最终 `story`

最终 `story` 会保存：

- `story_id`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`
- `common_variables`

## 8. 启动 session

调用：

- `story.start_session`

输入可包含：

- `story_id`
- `display_name`
- `player_profile_id`
- `api_group_id`
- `preset_id`

## 9. 运行回合

核心接口是：

- `session.run_turn`

它通过 `POST /rpc` 发起，但响应是 `text/event-stream`。客户端先收到 `ack`，再收到连续的 `message` 事件帧。
