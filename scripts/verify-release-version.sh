#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
expected_tag="${1:-}"
expected_version=""
if [[ -n "$expected_tag" ]]; then
  expected_version="${expected_tag#v}"
fi

read_json_version() {
  local file="$1"
  sed -n 's/^[[:space:]]*"version":[[:space:]]*"\([^"]*\)".*/\1/p' "$file" | head -n 1
}

read_toml_workspace_version() {
  local file="$1"
  sed -n '/^\[workspace\.package\]/,/^\[/{s/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p}' "$file" | head -n 1
}

cargo_version="$(read_toml_workspace_version "$repo_root/Cargo.toml")"
package_version="$(read_json_version "$repo_root/apps/vibe-app/package.json")"
package_lock_version="$(read_json_version "$repo_root/apps/vibe-app/package-lock.json")"
tauri_version="$(read_json_version "$repo_root/apps/vibe-app/src-tauri/tauri.conf.json")"

if [[ -z "$cargo_version" || -z "$package_version" || -z "$package_lock_version" || -z "$tauri_version" ]]; then
  echo "Failed to resolve one or more version sources." >&2
  exit 1
fi

declare -A versions=(
  ["Cargo.toml workspace.package.version"]="$cargo_version"
  ["apps/vibe-app/package.json version"]="$package_version"
  ["apps/vibe-app/package-lock.json version"]="$package_lock_version"
  ["apps/vibe-app/src-tauri/tauri.conf.json version"]="$tauri_version"
)

reference_version="$cargo_version"
for source in "${!versions[@]}"; do
  if [[ "${versions[$source]}" != "$reference_version" ]]; then
    echo "Release version mismatch detected." >&2
    printf '  %s = %s\n' "Cargo.toml workspace.package.version" "$cargo_version" >&2
    printf '  %s = %s\n' "apps/vibe-app/package.json version" "$package_version" >&2
    printf '  %s = %s\n' "apps/vibe-app/package-lock.json version" "$package_lock_version" >&2
    printf '  %s = %s\n' "apps/vibe-app/src-tauri/tauri.conf.json version" "$tauri_version" >&2
    exit 1
  fi
done

if [[ -n "$expected_version" && "$reference_version" != "$expected_version" ]]; then
  echo "Release tag/version mismatch detected." >&2
  printf '  tag version = %s\n' "$expected_version" >&2
  printf '  source version = %s\n' "$reference_version" >&2
  exit 1
fi

printf 'Verified release version sources: %s\n' "$reference_version"
if [[ -n "$expected_tag" ]]; then
  printf 'Verified release tag alignment: %s\n' "$expected_tag"
fi
