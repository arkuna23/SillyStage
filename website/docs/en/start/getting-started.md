# Getting Started

SillyStage currently ships with a Rust backend, an application frontend workspace in `webapp/`, and this documentation site in `website/`.

## Prerequisites

- Rust toolchain, stable, 2024 edition
- Cargo
- `just`, optional but recommended
- `pnpm` for `webapp/` and `website/`

## Start the Backend

```bash
just backend
# or
SS_APP_DEV_MODE=1 cargo run -p ss-app
```

The server listens on `127.0.0.1:8080` by default.

## Run Full Development Mode

```bash
just dev
```

## Develop This Docs Site

From `website/`:

```bash
pnpm dev
pnpm lint
pnpm format
pnpm build
```

## Default HTTP Entrypoints

- `POST /rpc`
- `GET /healthz`

## Configuration Loading

The app looks for `ss-app.toml` in the working directory.

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

Override order:

- CLI flags
- environment variables
- config file
- built-in defaults

Common environment variables:

- `SS_APP_LISTEN`
- `SS_APP_STORE_BACKEND`
- `SS_APP_STORE_ROOT`
