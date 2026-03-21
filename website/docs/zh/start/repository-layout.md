# 仓库结构

SillyStage 是一个 Rust 多 crate monorepo。每个 crate 都有清晰边界，避免把逻辑为了方便而跨层挪动。

## Crate 职责

- `ss-llm-api`：provider-neutral 的 LLM client 抽象与 provider 实现
- `ss-agents`：planner、architect、director、actor、narrator、keeper
- `ss-engine`：运行时状态、编排、manager 和 LLM registry
- `ss-store`：角色、资源、story、session、config 的持久化
- `ss-protocol`：传输无关的请求、响应、事件 payload
- `ss-handler`：业务操作与协议分发
- `ss-server`：HTTP / SSE 等传输适配层
- `ss-app`：应用启动、配置加载、store/registry 组装和 server boot
- `ss-state` / `ss-story`：共享领域模型

## 分层规则

- `ss-protocol` 定义 wire shape，不要在别处发明临时 JSON 结构
- `ss-handler` 负责应用操作，不要把 HTTP 细节搬进去
- `ss-server` 只做传输映射，不写领域逻辑
- `ss-app` 是组合层，负责把 config、store、engine、handler、server 接起来
- `ss-store` 只持久化长生命周期对象，临时上传态不要默认落库

## 文档与前端

- `webapp/`：应用前端工作区
- `website/`：独立的文档与博客站点
- `website/docs/en/api/` 与 `website/docs/zh/api/`：协议和接口的规范化文档源

如果后端 API 或行为发生变化，直接更新站点下的文档源即可。
