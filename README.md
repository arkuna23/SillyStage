# SillyStage

[中文](README.zh.md)

SillyStage is an AI-powered interactive storytelling engine built in Rust. It orchestrates multiple specialized AI agents to collaboratively generate, direct, and execute dynamic narrative experiences.

## Features

- **Multi-agent orchestration** – Six agents (Planner, Architect, Director, Actor, Narrator, Keeper) collaborate on every interactive turn.
- **Provider-neutral LLM client** – Per-agent API configuration supports any OpenAI-compatible backend.
- **Streaming responses** – Turn execution streams events to the client via Server-Sent Events.
- **Persistent storage** – Characters, stories, sessions, schemas, presets, and player profiles are stored on disk or in memory.
- **Character archives** – Import and export characters as `.chr` archive files.
- **Data packages** – Export and import story data as ZIP archives.
- **Bilingual** – API documentation and the built-in web UI support English and Simplified Chinese.
- **Cross-platform** – Self-contained binaries for Linux and Windows.

## Architecture

The project is a Cargo workspace with ten crates, organized in strict layers:

| Crate | Responsibility |
|---|---|
| `ss-state` | Shared domain models for runtime state |
| `ss-story` | Story graph and runtime graph representation |
| `ss-llm-api` | Provider-neutral LLM client and provider implementations |
| `ss-agents` | Planner, Architect, Director, Actor, Narrator, Keeper agents |
| `ss-store` | Persistent object storage (filesystem and in-memory backends) |
| `ss-protocol` | JSON-RPC request/response/event wire shapes |
| `ss-engine` | Runtime state machine and multi-agent orchestration |
| `ss-handler` | Business logic and protocol dispatch |
| `ss-server` | HTTP/SSE transport adapter (Axum) |
| `ss-app` | Application bootstrap, config loading, and server startup |

## Prerequisites

- Rust toolchain (stable, 2024 edition) with Cargo
- [`just`](https://just.systems/) task runner (optional but recommended)
- [pnpm](https://pnpm.io/) – only required for building the web frontend

## Getting Started

### Backend only

```bash
just backend
# or without just:
SS_APP_DEV_MODE=1 cargo run -p ss-app
```

The server listens on `127.0.0.1:8080` by default.

### Full development mode (backend + frontend hot-reload)

```bash
just dev
```

### Production build and packaging

```bash
just package-linux    # Linux x86_64
just package-windows  # Windows x86_64
just package-all      # Both targets
```

## Configuration

The application looks for `ss-app.toml` in the working directory. Settings can also be overridden with environment variables or CLI flags.

```toml
[server]
listen = "127.0.0.1:8080"
open_browser = true

[store]
backend = "fs"      # "fs" or "memory"
root = "./data"

[frontend]
enabled = true
mount_path = "/"
static_dir = "webapp/dist"
```

**Override precedence:** CLI flags > environment variables > config file > built-in defaults.

Common environment variables: `SS_APP_LISTEN`, `SS_APP_STORE_BACKEND`, `SS_APP_STORE_ROOT`.

## HTTP API

All business logic is exposed over a JSON-RPC 2.0 endpoint:

| Route | Purpose |
|---|---|
| `POST /rpc` | JSON-RPC 2.0 method dispatch |
| `GET /healthz` | Health check – returns `ok` |
| `POST /upload/{resource_id}/{file_id}` | Binary file upload |
| `GET /download/{resource_id}/{file_id}` | Binary file download |

Streaming methods (e.g. `session.run_turn`) respond with Server-Sent Events:
`ack` → `started` → `event`… → `completed` / `failed`.

### Quick example

```bash
# Create a character
curl -X POST http://127.0.0.1:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "character.create",
    "params": {"name": "Alice", "personality": "Cheerful merchant"}
  }'
```

## Workflow Overview

1. Configure LLM connections (`api.create`), agent groups (`api_group.create`), and generation presets (`preset.create`).
2. Create state schemas (`schema.create`) and player profiles (`player_profile.create`).
3. Import or create character cards.
4. Create story resources (`story_resources.create`) and generate a story (`story.generate` or the draft flow).
5. Start a session (`story.start_session`) and run interactive turns (`session.run_turn`).

See [`docs/en/process.md`](docs/en/process.md) for the full end-to-end workflow.

## Documentation

| Document | Description |
|---|---|
| [`docs/en/api/spec.md`](docs/en/api/spec.md) | Wire protocol specification |
| [`docs/en/api/reference.md`](docs/en/api/reference.md) | RPC method reference |
| [`docs/en/api/http.md`](docs/en/api/http.md) | HTTP transport details |
| [`docs/en/character.md`](docs/en/character.md) | Character card format |
| [`docs/en/process.md`](docs/en/process.md) | End-to-end workflow |

Chinese translations are available under [`docs/zh/`](docs/zh/).

## License

GNU General Public License v3.0 – see [LICENSE](LICENSE).
