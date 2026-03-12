set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
    @just --list

help:
    @just --list

backend:
    cargo run -p ss-app

backend-config config:
    cargo run -p ss-app -- --config "{{config}}"

frontend:
    cd webapp && pnpm dev

frontend-build:
    cd webapp && pnpm build

frontend-lint:
    cd webapp && pnpm lint

frontend-check:
    cd webapp && pnpm check

dev:
    bash -lc 'set -euo pipefail; cargo run -p ss-app & backend_pid=$!; cleanup() { kill "$backend_pid" 2>/dev/null || true; }; trap cleanup EXIT INT TERM; cd webapp; pnpm dev'
