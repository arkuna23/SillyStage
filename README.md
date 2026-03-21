# SillyStage

[中文](README.zh.md)

SillyStage is an AI-powered interactive storytelling engine built in Rust. It orchestrates multiple specialized AI agents to collaboratively generate, direct, and execute dynamic narrative experiences.

## Prerequisite

- Rust toolchain (stable, 2024 edition) with Cargo
- [`just`](https://just.systems/) task runner (optional but recommended)
- [pnpm](https://pnpm.io/) – only required for building the web frontend

## Getting Started

```bash
just backend
# or
SS_APP_DEV_MODE=1 cargo run -p ss-app
```

```bash
just dev
```

```bash
just package-linux    # Linux x86_64
just package-windows  # Windows x86_64
just package-all      # Both targets
```

The server listens on `127.0.0.1:8080` by default.

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
static_dir = "webapp"
```

Override precedence: CLI flags > environment variables > config file > built-in defaults.

Common environment variables: `SS_APP_LISTEN`, `SS_APP_STORE_BACKEND`, `SS_APP_STORE_ROOT`.

## License

GNU General Public License v3.0 – see [LICENSE](LICENSE).
