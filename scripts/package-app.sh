#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dist_root="$repo_root/dist"
targets=()

usage() {
  cat <<'EOF'
Usage: scripts/package-app.sh [--target <triple>]...

Builds the web frontend and the ss-app backend, assembles a self-contained
release directory, and emits a platform archive under dist/.

Examples:
  scripts/package-app.sh --target x86_64-unknown-linux-gnu
  scripts/package-app.sh --target x86_64-unknown-linux-gnu --target x86_64-pc-windows-gnu
EOF
}

while (($# > 0)); do
  case "$1" in
    --target)
      targets+=("$2")
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ((${#targets[@]} == 0)); then
  targets=("x86_64-unknown-linux-gnu")
fi

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

ensure_rust_target() {
  local target="$1"
  if ! rustup target list --installed | grep -Fxq "$target"; then
    echo "missing Rust target: $target" >&2
    echo "install it first with: rustup target add $target" >&2
    exit 1
  fi
}

archive_name_for() {
  local package_name="$1"
  local target="$2"
  if [[ "$target" == *windows* ]]; then
    printf '%s.zip' "$package_name"
  else
    printf '%s.tar.gz' "$package_name"
  fi
}

binary_name_for() {
  local target="$1"
  if [[ "$target" == *windows* ]]; then
    printf 'ss-app.exe'
  else
    printf 'ss-app'
  fi
}

write_packaged_config() {
  local package_dir="$1"
  cat >"$package_dir/ss-app.toml" <<'EOF'
[server]
listen = "127.0.0.1:8080"
open_browser = true

[store]
backend = "fs"
root = "data"

[frontend]
enabled = true
mount_path = "/"
static_dir = "webapp"
EOF
}

write_release_notes() {
  local package_dir="$1"
  local binary_name="$2"
  cat >"$package_dir/README.txt" <<EOF
SillyStage packaged release

1. Keep the directory structure intact.
2. Run ./$binary_name
3. The app will serve the bundled web frontend and store local data in ./data

Config:
- ss-app.toml is auto-discovered next to the executable.
- webapp/ contains the prebuilt frontend assets.
- data/ is created on first run if needed.
EOF
}

package_target() {
  local target="$1"
  local binary_name
  local package_name
  local package_dir
  local archive_name

  ensure_rust_target "$target"

  binary_name="$(binary_name_for "$target")"
  package_name="sillystage-${target}"
  package_dir="$dist_root/$package_name"
  archive_name="$(archive_name_for "$package_name" "$target")"

  rm -rf "$package_dir"
  mkdir -p "$package_dir/webapp" "$package_dir/data"

  cargo build --release -p ss-app --target "$target"

  cp "$repo_root/target/$target/release/$binary_name" "$package_dir/$binary_name"
  cp -R "$repo_root/webapp/dist/." "$package_dir/webapp/"
  write_packaged_config "$package_dir"
  write_release_notes "$package_dir" "$binary_name"

  rm -f "$dist_root/$archive_name"
  if [[ "$target" == *windows* ]]; then
    (
      cd "$dist_root"
      zip -rq "$archive_name" "$package_name"
    )
  else
    (
      cd "$dist_root"
      tar -czf "$archive_name" "$package_name"
    )
  fi

  echo "packaged $target -> $dist_root/$archive_name"
}

require_command cargo
require_command rustup
require_command pnpm
require_command tar
require_command zip

mkdir -p "$dist_root"

(
  cd "$repo_root/webapp"
  pnpm build
)

for target in "${targets[@]}"; do
  package_target "$target"
done
