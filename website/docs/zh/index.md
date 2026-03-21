# 中文文档

SillyStage 是一个基于 Rust 构建的 AI 交互叙事引擎。它协调多个专门化 AI agent，协作生成、导演并执行动态叙事体验。

## 文档范围

- 快速启动、配置方式和本地开发命令
- Rust 多 crate 仓库结构与分层职责
- 从资源准备到 session 运行的端到端流程
- `POST /rpc`、SSE 流式返回、二进制上传下载等协议说明
- 当前已实现方法的参考清单

## 推荐阅读顺序

1. [快速开始](./start/getting-started)
2. [仓库结构](./start/repository-layout)
3. [运行流程](./guide/runtime-flow)
4. [API 协议结构](./api/protocol)
5. [API 参考](./api/reference)

## 相关目录

- `website/docs/zh` 和 `website/docs/en`：当前站点的文档源
- `webapp/`：应用前端工作区
- `website/`：当前文档与博客站点
