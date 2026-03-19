# SillyStage

[English](README.md)

SillyStage 是一个基于 Rust 构建的 AI 交互叙事引擎。它协调多个专门化 AI 智能体，协作生成、导演并执行动态叙事体验。

## 功能特性

- **多智能体协作** – Planner、Architect、Director、Actor、Narrator、Keeper 六个智能体共同参与每一个交互回合。
- **中立的 LLM 客户端** – 每个智能体可单独配置 API，支持任何兼容 OpenAI 接口的后端。
- **流式响应** – 回合执行通过 Server-Sent Events 将事件实时推送给客户端。
- **持久化存储** – 角色、故事、会话、schema、预设和玩家设定均可持久化到磁盘或内存。
- **角色归档** – 支持以 `.chr` 归档文件格式导入和导出角色卡。
- **数据包** – 支持以 ZIP 归档格式导出和导入故事数据。
- **双语支持** – API 文档及内置 Web UI 同时支持英语和简体中文。
- **跨平台** – 提供 Linux 和 Windows 自包含二进制包。

## 架构

本项目是一个包含十个 crate 的 Cargo workspace，按严格的分层组织：

| Crate | 职责 |
|---|---|
| `ss-state` | 运行时状态的共享领域模型 |
| `ss-story` | 故事图和运行时图的表示 |
| `ss-llm-api` | 中立的 LLM 客户端及各提供商实现 |
| `ss-agents` | Planner、Architect、Director、Actor、Narrator、Keeper 智能体 |
| `ss-store` | 持久化对象存储（文件系统和内存两种后端） |
| `ss-protocol` | JSON-RPC 请求/响应/事件的消息格式定义 |
| `ss-engine` | 运行时状态机和多智能体编排 |
| `ss-handler` | 业务逻辑和协议分发 |
| `ss-server` | HTTP/SSE 传输适配层（Axum） |
| `ss-app` | 应用启动、配置加载和服务器初始化 |

## 前置要求

- Rust 工具链（stable，2024 edition）及 Cargo
- [`just`](https://just.systems/) 任务运行器（可选，但推荐）
- [pnpm](https://pnpm.io/) – 仅在构建 Web 前端时需要

## 快速上手

### 仅启动后端

```bash
just backend
# 或不使用 just：
SS_APP_DEV_MODE=1 cargo run -p ss-app
```

服务器默认监听 `127.0.0.1:8080`。

### 完整开发模式（后端 + 前端热更新）

```bash
just dev
```

### 生产构建与打包

```bash
just package-linux    # Linux x86_64
just package-windows  # Windows x86_64
just package-all      # 两个目标平台
```

## 配置

应用会在工作目录中查找 `ss-app.toml`。配置项也可以通过环境变量或命令行参数覆盖。

```toml
[server]
listen = "127.0.0.1:8080"
open_browser = true

[store]
backend = "fs"      # "fs" 或 "memory"
root = "./data"

[frontend]
enabled = true
mount_path = "/"
static_dir = "webapp/dist"
```

**优先级顺序：** 命令行参数 > 环境变量 > 配置文件 > 内置默认值。

常用环境变量：`SS_APP_LISTEN`、`SS_APP_STORE_BACKEND`、`SS_APP_STORE_ROOT`。

## HTTP API

所有业务逻辑通过 JSON-RPC 2.0 接口暴露：

| 路由 | 用途 |
|---|---|
| `POST /rpc` | JSON-RPC 2.0 方法分发 |
| `GET /healthz` | 健康检查，返回 `ok` |
| `POST /upload/{resource_id}/{file_id}` | 二进制文件上传 |
| `GET /download/{resource_id}/{file_id}` | 二进制文件下载 |

流式方法（如 `session.run_turn`）通过 Server-Sent Events 响应：
`ack` → `started` → `event`… → `completed` / `failed`。

### 快速示例

```bash
# 创建角色
curl -X POST http://127.0.0.1:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "character.create",
    "params": {"name": "Alice", "personality": "热情的商人"}
  }'
```

## 流程概览

1. 配置 LLM 连接（`api.create`）、智能体组（`api_group.create`）和生成预设（`preset.create`）。
2. 创建状态 schema（`schema.create`）和玩家设定（`player_profile.create`）。
3. 导入或手动创建角色卡。
4. 创建故事资源（`story_resources.create`）并生成故事（`story.generate` 或 draft 流程）。
5. 启动会话（`story.start_session`）并进行交互回合（`session.run_turn`）。

完整的端到端流程请参阅 [`docs/zh/process.md`](docs/zh/process.md)。

## 文档

| 文档 | 说明 |
|---|---|
| [`docs/zh/api/spec.md`](docs/zh/api/spec.md) | 协议规范 |
| [`docs/zh/api/reference.md`](docs/zh/api/reference.md) | RPC 方法参考 |
| [`docs/zh/api/http.md`](docs/zh/api/http.md) | HTTP 传输细节 |
| [`docs/zh/character.md`](docs/zh/character.md) | 角色卡格式说明 |
| [`docs/zh/process.md`](docs/zh/process.md) | 端到端流程 |

英文文档请参阅 [`docs/en/`](docs/en/)。

## 许可证

GNU 通用公共许可证 v3.0 – 详见 [LICENSE](LICENSE)。
