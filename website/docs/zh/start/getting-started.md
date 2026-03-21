# 快速开始

SillyStage 默认提供一个 Rust 后端、一个应用前端工作区 `webapp/`，以及当前文档站 `website/`。

## 前置要求

- Rust 工具链（stable，2024 edition）
- Cargo
- `just` 任务运行器，可选但推荐
- `pnpm`，用于 `webapp/` 和 `website/` 的开发与构建

## 启动后端

```bash
just backend
# 或
SS_APP_DEV_MODE=1 cargo run -p ss-app
```

默认监听地址为 `127.0.0.1:8080`。

## 联调开发

```bash
just dev
```

## 当前文档站开发

在 `website/` 目录中运行：

```bash
pnpm dev
pnpm lint
pnpm format
pnpm build
```

## 默认 HTTP 入口

- `POST /rpc`
- `GET /healthz`

## 配置加载

应用会在工作目录查找 `ss-app.toml`。CLI 参数、环境变量和配置文件会共同参与覆盖。

```toml
[server]
listen = "127.0.0.1:8080"
open_browser = true

[store]
backend = "fs"
root = "./data"

[frontend]
enabled = true
mount_path = "/"
static_dir = "webapp"
```

优先级顺序：

- CLI 参数
- 环境变量
- 配置文件
- 内置默认值

常见环境变量：

- `SS_APP_LISTEN`
- `SS_APP_STORE_BACKEND`
- `SS_APP_STORE_ROOT`

## 前端目录说明

- `webapp/`：产品应用前端
- `website/`：文档与博客站点

两者职责独立，不要把产品界面逻辑放进 `website/`。
