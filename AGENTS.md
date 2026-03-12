# SillyStage Repo Guide

This repository is a Rust multi-crate monorepo. Keep responsibilities separated by crate and avoid pushing logic across layers just because it is convenient.

## Crate Boundaries

- `ss-llm-api`: provider-neutral LLM client abstraction and provider implementations.
- `ss-agents`: planner, architect, director, actor, narrator, keeper.
- `ss-engine`: runtime state, orchestration, manager, and LLM registry.
- `ss-store`: persistent object storage for characters, resources, stories, sessions, and config.
- `ss-protocol`: request/response/event payloads and transport-neutral protocol objects.
- `ss-handler`: protocol dispatch and business operations over store and engine.
- `ss-server`: transport adapters such as HTTP/SSE.
- `ss-app`: application startup, config loading, store/registry assembly, and server boot.
- `ss-state` / `ss-story`: shared domain models for state and story graph/runtime graph.

## Layering Rules

- `ss-protocol` defines wire shapes. Do not invent ad hoc request/response JSON outside it.
- `ss-handler` owns application operations. Do not move HTTP-specific behavior into it.
- `ss-server` is transport-only. Keep protocol mapping there, not domain logic.
- `ss-app` is the composition layer. It should wire config, store, registry, handler, and server together.
- `ss-store` persists long-lived objects. Temporary upload state should stay out of it unless there is a clear persistence need.

## Working Rules

- Prefer adding tests under `tests/` instead of inline source tests unless there is a strong reason not to.
- When protocol payloads change, update the docs under `docs/zh/api/` and `docs/en/api/`.
- Keep frontend concerns out of the Rust crates. The future frontend lives under `webapp/`.
- If a task touches `webapp/`, follow `webapp/AGENTS.md` in addition to this file.
- If a task touches the frontend, Node tooling, or `webapp/`, use `pnpm` as the package manager. Do not introduce `npm`/`yarn` workflows or their lockfiles.

## Frontend Status

- `webapp/` is currently a reserved folder for the future web frontend.
- Do not initialize a frontend toolchain unless the task explicitly asks for it.
- The current HTTP entrypoints are `POST /rpc` and `GET /healthz`; the placeholder frontend route is mounted by `ss-app`.
