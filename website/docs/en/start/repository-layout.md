# Repository Layout

SillyStage is a Rust multi-crate monorepo. Crate boundaries are intentional and should stay explicit.

## Crate Responsibilities

- `ss-llm-api`: provider-neutral LLM client abstraction and provider implementations
- `ss-agents`: planner, architect, director, actor, narrator, keeper
- `ss-engine`: runtime state, orchestration, manager, and LLM registry
- `ss-store`: persistent storage for characters, resources, stories, sessions, and config
- `ss-protocol`: transport-neutral request, response, and event payloads
- `ss-handler`: application operations and protocol dispatch
- `ss-server`: transport adapters such as HTTP and SSE
- `ss-app`: application startup, config loading, assembly, and server boot
- `ss-state` / `ss-story`: shared domain models

## Layering Rules

- `ss-protocol` defines wire shapes
- `ss-handler` owns application operations
- `ss-server` stays transport-only
- `ss-app` wires the system together
- `ss-store` persists long-lived objects only

## Frontend and Docs

- `webapp/`: product application frontend
- `website/`: docs and blog website
- `website/docs/en/api/` and `website/docs/zh/api/`: canonical protocol source docs

If an API or backend behavior changes, update the website docs directly.
