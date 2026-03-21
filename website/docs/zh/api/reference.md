# API 参考

本文汇总当前已经实现的 JSON-RPC 方法和传输层 HTTP 端点。

## 1. binary HTTP

| 路由 | 请求体 | 成功响应 | 说明 |
| --- | --- | --- | --- |
| `POST /upload/{resource_id}/{file_id}` | 原始字节 | `ResourceFilePayload` | 逻辑资源文件上传 |
| `GET /download/{resource_id}/{file_id}` | 无 | 原始字节 | 逻辑资源文件下载 |

## 2. 配置与资源方法族

| 方法族 | 主要方法 | 说明 |
| --- | --- | --- |
| `api.*` | `create` `get` `list` `list_models` `update` `delete` | 管理可复用 API 连接 |
| `api_group.*` | `create` `get` `list` `update` `delete` | 管理每-agent API 绑定 |
| `preset.*` | `create` `get` `list` `update` `delete` | 管理生成参数与模块化提示词 |
| `preset_entry.*` | `create` `update` `delete` | 管理单条自定义提示词条目 |
| `preset_preview.*` | `template` `runtime` | 预览编译后的提示词 |
| `schema.*` | `create` `get` `list` `update` `delete` | 管理状态 schema |
| `lorebook.*` | `create` `get` `list` `update` `delete` | 管理 lorebook |
| `lorebook_entry.*` | `create` `get` `list` `update` `delete` | 管理 lorebook 条目 |
| `player_profile.*` | `create` `get` `list` `update` `delete` | 管理玩家设定 |
| `character.*` | `create` `get` `list` `update` `delete` | 管理角色卡 |

说明：

- `api.delete` 若仍被 `api_group` 引用，返回 `conflict`
- `preset.delete` 若仍被 story draft 或 session 引用，返回 `conflict`
- `schema.delete` 若仍被角色、resources、story 或 session 引用，返回 `conflict`
- `character` 的封面与 `.chr` 文件通过二进制路由传输

## 3. Story 相关方法族

| 方法族 | 主要方法 | 说明 |
| --- | --- | --- |
| `story_resources.*` | `create` `get` `list` `update` `delete` | 生成前输入资源 |
| `story.*` | `generate_plan` `create` `generate` `get` `update` `update_graph` `list` `delete` `start_session` | 管理 story 与启动 session |
| `story_draft.*` | `start` `get` `list` `update_graph` `continue` `finalize` `delete` | 分段生成大型 story |

重点：

- `story.generate` 是兼容封装，内部仍走完整 draft 流程
- `story_draft.start`、`story.generate`、`story.generate_plan` 都支持可选 `api_group_id` / `preset_id`
- `story.update_graph` 与 `story_draft.update_graph` 都会校验 graph

## 4. Session 相关方法族

| 方法族 | 主要方法 | 说明 |
| --- | --- | --- |
| `session.*` | `get` `update` `list` `delete` `run_turn` `suggest_replies` `set_player_profile` `update_player_description` `get_runtime_snapshot` `get_variables` `update_variables` `get_config` `update_config` | 核心运行时接口 |
| `session_character.*` | `get` `list` `update` `delete` `enter_scene` `leave_scene` | 管理 session 级临时角色 |
| `session_message.*` | `create` `get` `list` `update` `delete` | 管理 transcript 消息记录 |

说明：

- `session.run_turn` 是当前主要流式方法
- `session.suggest_replies` 默认返回 3 条建议，允许请求 `2..=5`
- `session.update_variables` 只接受变量类更新
- session 级临时角色不会修改底层 story graph

## 5. 全局与聚合接口

| 方法 | 返回 | 说明 |
| --- | --- | --- |
| `config.get_global` | `global_config` | 获取兜底 `api_group_id` / `preset_id` |
| `dashboard.get` | `dashboard` | 获取 dashboard 聚合摘要 |

`dashboard` 包含：

- `health`
- `counts`
- `global_config`
- `recent_stories`
- `recent_sessions`

## 6. 数据包接口

| 方法 | 返回 | 说明 |
| --- | --- | --- |
| `data_package.export_prepare` | `data_package_export_prepared` | 构建临时 ZIP 导出槽位 |
| `data_package.import_prepare` | `data_package_import_prepared` | 分配临时 ZIP 导入槽位 |
| `data_package.import_commit` | `data_package_import_committed` | 校验并原子导入 ZIP |

支持打包的资源类型：

- `preset`
- `schema`
- `lorebook`
- `player_profile`
- `character`
- `story_resources`
- `story`

导入导出流程：

1. `data_package.export_prepare` 后通过 `GET /download/package_export:{export_id}/archive` 下载
2. `data_package.import_prepare` 后通过 `POST /upload/package_import:{import_id}/archive` 上传
3. `data_package.import_commit` 执行校验和原子导入
