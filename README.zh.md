# SillyStage

[English](README.md)

SillyStage 是一个基于 Rust 构建的 AI 交互叙事引擎。它协调多个专门化 AI 智能体，协作生成、导演并执行动态叙事体验。

## 前置要求

- Rust 工具链（stable，2024 edition）及 Cargo
- [`just`](https://just.systems/) 任务运行器（可选，但推荐）
- [pnpm](https://pnpm.io/) – 仅在构建 Web 前端时需要

## 快速上手

```bash
just backend
# 或
SS_APP_DEV_MODE=1 cargo run -p ss-app
```

```bash
just dev
```

```bash
just package-linux    # Linux x86_64
just package-windows  # Windows x86_64
just package-all      # 两个目标平台
```

服务器默认监听 `127.0.0.1:8080`。

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

优先级顺序：命令行参数 > 环境变量 > 配置文件 > 内置默认值。

常用环境变量：`SS_APP_LISTEN`、`SS_APP_STORE_BACKEND`、`SS_APP_STORE_ROOT`。

## 许可证

GNU 通用公共许可证 v3.0 – 详见 [LICENSE](LICENSE)。
