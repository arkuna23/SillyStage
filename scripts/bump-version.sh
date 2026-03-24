#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'EOF'
Usage:
  bash scripts/bump-version.sh <version>
  bash scripts/bump-version.sh <major|minor|patch>

Examples:
  bash scripts/bump-version.sh 0.1.1
  bash scripts/bump-version.sh patch
EOF
}

if [[ $# -ne 1 ]]; then
    usage >&2
    exit 1
fi

input="$1"
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
webapp_package_json="${repo_root}/webapp/package.json"

mapfile -t crate_files < <(find "${repo_root}" -maxdepth 2 -path "${repo_root}/ss-*/Cargo.toml" | sort)

if [[ ${#crate_files[@]} -eq 0 ]]; then
    echo "error: no crate Cargo.toml files found under ${repo_root}" >&2
    exit 1
fi

extract_version() {
    local file="$1"
    sed -nE 's/^version = "([0-9]+\.[0-9]+\.[0-9]+)"$/\1/p' "${file}" | head -n 1
}

extract_package_version() {
    local file="$1"
    sed -nE 's/^[[:space:]]*"version":[[:space:]]*"([0-9]+\.[0-9]+\.[0-9]+)",?[[:space:]]*$/\1/p' "${file}" | head -n 1
}

replace_cargo_version() {
    local file="$1"
    local old_version="$2"
    local new_version="$3"
    local temp_file
    temp_file="$(mktemp)"

    if ! awk -v old="${old_version}" -v new="${new_version}" '
        $0 == "version = \"" old "\"" {
            print "version = \"" new "\""
            replaced = 1
            next
        }
        { print }
        END {
            if (!replaced) {
                exit 1
            }
        }
    ' "${file}" > "${temp_file}"; then
        rm -f "${temp_file}"
        return 1
    fi

    mv "${temp_file}" "${file}"
}

replace_package_version() {
    local file="$1"
    local old_version="$2"
    local new_version="$3"
    local temp_file
    temp_file="$(mktemp)"

    if ! awk -v old="${old_version}" -v new="${new_version}" '
        {
            original = $0
            marker = "\"version\": \"" old "\""
            if (index(original, marker) > 0) {
                sub(marker, "\"version\": \"" new "\"", original)
                replaced = 1
            }
            print original
        }
        END {
            if (!replaced) {
                exit 1
            }
        }
    ' "${file}" > "${temp_file}"; then
        rm -f "${temp_file}"
        return 1
    fi

    mv "${temp_file}" "${file}"
}

current_version="$(extract_version "${crate_files[0]}")"
if [[ -z "${current_version}" ]]; then
    echo "error: failed to detect version from ${crate_files[0]}" >&2
    exit 1
fi

for file in "${crate_files[@]:1}"; do
    file_version="$(extract_version "${file}")"
    if [[ -z "${file_version}" ]]; then
        echo "error: failed to detect version from ${file}" >&2
        exit 1
    fi
    if [[ "${file_version}" != "${current_version}" ]]; then
        echo "error: crate versions are not aligned (${file} uses ${file_version}, expected ${current_version})" >&2
        exit 1
    fi
done

webapp_current_version=""
if [[ -f "${webapp_package_json}" ]]; then
    webapp_current_version="$(extract_package_version "${webapp_package_json}")"
    if [[ -z "${webapp_current_version}" ]]; then
        echo "error: failed to detect version from ${webapp_package_json}" >&2
        exit 1
    fi
fi

if [[ "${input}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    target_version="${input}"
else
    IFS='.' read -r current_major current_minor current_patch <<<"${current_version}"
    case "${input}" in
        major)
            target_version="$((current_major + 1)).0.0"
            ;;
        minor)
            target_version="${current_major}.$((current_minor + 1)).0"
            ;;
        patch)
            target_version="${current_major}.${current_minor}.$((current_patch + 1))"
            ;;
        *)
            echo "error: invalid version argument '${input}'" >&2
            usage >&2
            exit 1
            ;;
    esac
fi

if [[ "${target_version}" == "${current_version}" ]] \
    && [[ -z "${webapp_current_version}" || "${webapp_current_version}" == "${target_version}" ]]; then
    echo "error: target version matches current version (${current_version})" >&2
    exit 1
fi

echo "Current version: ${current_version}"
if [[ -n "${webapp_current_version}" ]]; then
    echo "Webapp version:  ${webapp_current_version}"
fi
echo "Target version:  ${target_version}"
if [[ "${target_version}" != "${current_version}" ]]; then
    echo "Updating crate manifests:"

    for file in "${crate_files[@]}"; do
        relative_path="${file#${repo_root}/}"
        echo "  - ${relative_path}"
        if ! replace_cargo_version "${file}" "${current_version}" "${target_version}"; then
            echo "error: failed to update ${relative_path}" >&2
            exit 1
        fi

        updated_version="$(extract_version "${file}")"
        if [[ "${updated_version}" != "${target_version}" ]]; then
            echo "error: failed to update ${relative_path}" >&2
            exit 1
        fi
    done
else
    echo "Crate manifests already match target version."
fi

if [[ -n "${webapp_current_version}" ]]; then
    if [[ "${webapp_current_version}" != "${current_version}" ]]; then
        echo "Webapp version differs from backend and will be synchronized."
    fi
    if [[ "${webapp_current_version}" != "${target_version}" ]]; then
        echo "Updating frontend package:"
        echo "  - webapp/package.json"
        if ! replace_package_version "${webapp_package_json}" "${webapp_current_version}" "${target_version}"; then
            echo "error: failed to update webapp/package.json" >&2
            exit 1
        fi

        updated_webapp_version="$(extract_package_version "${webapp_package_json}")"
        if [[ "${updated_webapp_version}" != "${target_version}" ]]; then
            echo "error: failed to update webapp/package.json" >&2
            exit 1
        fi
    else
        echo "Frontend package already matches target version."
    fi
fi

if [[ -f "${repo_root}/Cargo.lock" ]]; then
    echo "Note: Cargo.lock may need refresh via cargo check if workspace package versions changed."
fi

echo "Version bump complete."
