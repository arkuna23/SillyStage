set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command"]

backend_cmd := if os_family() == "windows" {
    "$env:SS_APP_DEV_MODE='1'; cargo run -p ss-app"
} else {
    "SS_APP_DEV_MODE=1 cargo run -p ss-app"
}

backend_config_cmd := if os_family() == "windows" {
    "$env:SS_APP_DEV_MODE='1'; cargo run -p ss-app -- --config"
} else {
    "SS_APP_DEV_MODE=1 cargo run -p ss-app -- --config"
}

dev_cmd := if os_family() == "windows" {
    "$env:SS_APP_DEV_MODE='1'; $backend = Start-Process cargo -ArgumentList 'run','-p','ss-app' -WorkingDirectory $PWD -NoNewWindow -PassThru; try { Set-Location webapp; pnpm dev } finally { if ($backend -and -not $backend.HasExited) { Stop-Process -Id $backend.Id } }"
} else {
    "bash -lc 'set -euo pipefail; SS_APP_DEV_MODE=1 cargo run -p ss-app & backend_pid=$!; cleanup() { kill \"$backend_pid\" 2>/dev/null || true; }; trap cleanup EXIT INT TERM; cd webapp; pnpm dev'"
}

package_linux_cmd := if os_family() == "windows" {
    "& ./scripts/package-app.ps1 -Target x86_64-unknown-linux-gnu"
} else {
    "bash scripts/package-app.sh --target x86_64-unknown-linux-gnu"
}

package_windows_cmd := if os_family() == "windows" {
    "& ./scripts/package-app.ps1 -Target x86_64-pc-windows-msvc"
} else {
    "bash scripts/package-app.sh --target x86_64-pc-windows-gnu"
}

package_all_cmd := if os_family() == "windows" {
    "& ./scripts/package-app.ps1 -Target x86_64-unknown-linux-gnu -Target x86_64-pc-windows-msvc"
} else {
    "bash scripts/package-app.sh --target x86_64-unknown-linux-gnu --target x86_64-pc-windows-gnu"
}

version_cmd := "bash scripts/bump-version.sh"

default:
    @just --list

help:
    @just --list

backend:
    {{backend_cmd}}

backend-config config:
    {{backend_config_cmd}} "{{config}}"

frontend:
    cd webapp; pnpm dev

frontend-build:
    cd webapp; pnpm build

frontend-lint:
    cd webapp; pnpm lint

frontend-check:
    cd webapp; pnpm check

docs:
    cd website; pnpm dev

docs-host:
    cd website; pnpm dev:host

docs-build:
    cd website; pnpm build

docs-lint:
    cd website; pnpm lint

docs-check:
    cd website; pnpm check

docs-preview:
    cd website; pnpm preview

docs-preview-host:
    cd website; pnpm preview:host

dev:
    {{dev_cmd}}

package-linux:
    {{package_linux_cmd}}

package-windows:
    {{package_windows_cmd}}

package-all:
    {{package_all_cmd}}

version arg:
    {{version_cmd}} "{{arg}}"
