#!/usr/bin/env bash
set -euo pipefail

RED_COLOR='\e[1;31m'
GREEN_COLOR='\e[1;32m'
YELLOW_COLOR='\e[1;33m'
BLUE_COLOR='\e[1;34m'
RES='\e[0m'

REPO_OWNER="${REPO_OWNER:-fage-ac-org}"
REPO_NAME="${REPO_NAME:-vibe-everywhere}"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"
VIBE_RELEASE_TAG="${VIBE_RELEASE_TAG:-}"
VIBE_CLI_ARCHIVE_URL="${VIBE_CLI_ARCHIVE_URL:-${RELAY_CLI_ARCHIVE_URL:-}}"
VIBE_CLI_ARCHIVE_PATH="${VIBE_CLI_ARCHIVE_PATH:-${RELAY_CLI_ARCHIVE_PATH:-}}"
INSTALL_COMPONENT="${INSTALL_COMPONENT:-all}"
GH_PROXY="${GH_PROXY:-https://ghfast.top/}"
NO_GH_PROXY=false

help() {
  cat <<'EOF'
Vibe Everywhere CLI Binary Installer

Usage:
  ./install-relay.sh <command> [bin-dir] [options]

Commands:
  install                 Download and install CLI binaries
  update                  Download and replace existing CLI binaries
  uninstall               Remove installed CLI binaries
  help                    Show this help message

Options:
  --bin-dir DIR           Target directory for installed binaries (default: /usr/local/bin)
  --component VALUE       Which binaries to manage: all, relay, or agent (default: all)
  --release-tag TAG       Install a specific release tag, for example v0.1.8
  --archive-url URL       Download from a custom archive URL
  --archive-path PATH     Install from a local archive path
  --gh-proxy URL          Prefix applied to GitHub release and redirect URLs
  --no-gh-proxy           Disable the GitHub proxy prefix
  --repo-owner OWNER      Override the GitHub repository owner
  --repo-name NAME        Override the GitHub repository name

Examples:
  ./install-relay.sh install
  ./install-relay.sh install --component relay
  ./install-relay.sh install --component agent
  ./install-relay.sh update --component all --release-tag v0.1.8
  ./install-relay.sh uninstall --component agent
  ./install-relay.sh install --archive-path /tmp/vibe-everywhere-cli.tar.gz

Notes:
  This script manages the published CLI binaries from the release archive.
  On Linux, the default install location is /usr/local/bin, which follows the common convention
  for administrator-installed local executables.
  It does not create services, write environment files, or start relay or agent processes.
EOF
}

info() {
  printf '%b\n' "${BLUE_COLOR}$*${RES}"
}

success() {
  printf '%b\n' "${GREEN_COLOR}$*${RES}"
}

warn() {
  printf '%b\n' "${YELLOW_COLOR}$*${RES}" >&2
}

die() {
  printf '%b\n' "${RED_COLOR}$*${RES}" >&2
  exit 1
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    die "Missing required command: $1"
  fi
}

apply_gh_proxy() {
  local url="$1"
  if $NO_GH_PROXY; then
    printf '%s' "$url"
  else
    printf '%s%s' "$GH_PROXY" "$url"
  fi
}

detect_linux_target() {
  local platform
  if command -v arch >/dev/null 2>&1; then
    platform="$(arch)"
  else
    platform="$(uname -m)"
  fi

  case "$platform" in
    amd64|x86_64)
      printf '%s' "x86_64-unknown-linux-gnu"
      ;;
    *)
      die "Unsupported Linux architecture: ${platform}. Use --archive-path or --archive-url for a custom build."
      ;;
  esac
}

resolve_latest_release_tag() {
  local latest_url effective_url
  latest_url="$(apply_gh_proxy "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest")"
  effective_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' "$latest_url")"
  python3 - "$effective_url" <<'PY'
import re
import sys

match = re.search(r"/releases/(?:tag|download)/([^/?#]+)", sys.argv[1])
if not match:
    raise SystemExit(1)

print(match.group(1))
PY
}

build_archive_url() {
  local release_tag="$1"
  local archive_name="$2"
  local direct_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${release_tag}/${archive_name}"
  apply_gh_proxy "$direct_url"
}

download_archive() {
  local url="$1"
  local output_path="$2"
  curl -fL --retry 5 --retry-all-errors "$url" -o "$output_path"
}

normalize_component() {
  case "$1" in
    all|relay|agent)
      printf '%s' "$1"
      ;;
    *)
      die "Unsupported component: $1. Allowed values: all, relay, agent."
      ;;
  esac
}

selected_binaries() {
  case "$INSTALL_COMPONENT" in
    all)
      printf '%s\n' "vibe-relay" "vibe-agent"
      ;;
    relay)
      printf '%s\n' "vibe-relay"
      ;;
    agent)
      printf '%s\n' "vibe-agent"
      ;;
  esac
}

binary_path() {
  printf '%s' "${BIN_DIR%/}/$1"
}

print_post_install_notes() {
  local docs_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/blob/main/docs/relay-startup.md"
  local readme_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/blob/main/README.en.md"
  local binary

  success "Installed CLI binaries to ${BIN_DIR}"
  for binary in "$@"; do
    printf 'Installed: %s\n' "$(binary_path "$binary")"
  done
  printf 'Relay startup guide: %s\n' "$docs_url"
  printf 'General usage guide: %s\n' "$readme_url"
  if ! $NO_GH_PROXY; then
    printf 'Relay startup guide (accelerated): %s\n' "$(apply_gh_proxy "$docs_url")"
    printf 'General usage guide (accelerated): %s\n' "$(apply_gh_proxy "$readme_url")"
  fi
  warn "This script only installs or updates binaries. Configure environment variables and startup separately."
}

install_or_update() {
  local command="$1"
  local target_triple archive_name release_tag archive_path temp_archive_path extract_dir existing_count
  local binary
  local -a binaries=()

  require_command curl
  require_command python3
  require_command tar
  require_command install
  require_command mktemp

  target_triple="$(detect_linux_target)"
  while IFS= read -r binary; do
    binaries+=("$binary")
  done < <(selected_binaries)

  if [[ "$command" == "update" ]]; then
    existing_count=0
    for binary in "${binaries[@]}"; do
      if [[ -f "$(binary_path "$binary")" ]]; then
        existing_count=$((existing_count + 1))
      fi
    done
    if [[ "$existing_count" -eq 0 ]]; then
      die "No selected CLI binaries were found under ${BIN_DIR}. Run install first or choose a different --bin-dir."
    fi
  fi

  release_tag="$VIBE_RELEASE_TAG"
  archive_path="$VIBE_CLI_ARCHIVE_PATH"

  if [[ -z "$release_tag" && -z "$VIBE_CLI_ARCHIVE_URL" && -z "$archive_path" ]]; then
    info "Resolving latest release tag..."
    release_tag="$(resolve_latest_release_tag)" || die "Failed to resolve the latest release tag. Try --release-tag, --archive-url, or --no-gh-proxy."
  fi

  if [[ -z "$archive_path" ]]; then
    if [[ -z "$VIBE_CLI_ARCHIVE_URL" ]]; then
      archive_name="vibe-everywhere-cli-${release_tag}-${target_triple}.tar.gz"
      VIBE_CLI_ARCHIVE_URL="$(build_archive_url "$release_tag" "$archive_name")"
    fi

    temp_archive_path="$(mktemp /tmp/vibe-cli.XXXXXX.tar.gz)"
    info "Downloading CLI archive..."
    info "Archive URL: ${VIBE_CLI_ARCHIVE_URL}"
    download_archive "$VIBE_CLI_ARCHIVE_URL" "$temp_archive_path"
    archive_path="$temp_archive_path"
  fi

  extract_dir="$(mktemp -d)"
  tar -xzf "$archive_path" -C "$extract_dir"

  for binary in "${binaries[@]}"; do
    if [[ ! -f "$extract_dir/$binary" ]]; then
      rm -rf "$extract_dir"
      die "Archive does not contain ${binary} at the expected path."
    fi
  done

  if ! mkdir -p "$BIN_DIR"; then
    rm -rf "$extract_dir"
    die "Failed to create ${BIN_DIR}. Use sudo or choose a writable --bin-dir."
  fi

  for binary in "${binaries[@]}"; do
    if ! install -m 0755 "$extract_dir/$binary" "$(binary_path "$binary")"; then
      rm -rf "$extract_dir"
      die "Failed to install ${binary} to $(binary_path "$binary"). Use sudo or choose a writable --bin-dir."
    fi
  done

  rm -rf "$extract_dir"
  if [[ -n "${temp_archive_path:-}" ]]; then
    rm -f "$temp_archive_path"
  fi

  print_post_install_notes "${binaries[@]}"
}

uninstall_binaries() {
  local removed_count=0
  local binary binary_full_path
  local -a binaries=()

  while IFS= read -r binary; do
    binaries+=("$binary")
  done < <(selected_binaries)

  for binary in "${binaries[@]}"; do
    binary_full_path="$(binary_path "$binary")"
    if [[ -e "$binary_full_path" ]]; then
      if ! rm -f "$binary_full_path"; then
        die "Failed to remove ${binary_full_path}. Use sudo or choose a writable --bin-dir."
      fi
      success "Removed ${binary_full_path}"
      removed_count=$((removed_count + 1))
    else
      warn "No binary found at ${binary_full_path}."
    fi
  done

  if [[ "$removed_count" -eq 0 ]]; then
    warn "No selected CLI binaries were removed."
  fi
  warn "This command removes only the selected CLI binaries. It does not remove service definitions, startup scripts, environment files, or runtime state."
}

COMMAND="${1:-help}"
if [[ "$COMMAND" == "help" || "$COMMAND" == "--help" || "$COMMAND" == "-h" ]]; then
  help
  exit 0
fi
shift || true

if [[ "$#" -ge 1 && "$1" != --* ]]; then
  BIN_DIR="$1"
  shift
fi

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --bin-dir)
      [[ "$#" -ge 2 ]] || die "Option --bin-dir requires a value."
      BIN_DIR="$2"
      shift 2
      ;;
    --component)
      [[ "$#" -ge 2 ]] || die "Option --component requires a value."
      INSTALL_COMPONENT="$(normalize_component "$2")"
      shift 2
      ;;
    --release-tag)
      [[ "$#" -ge 2 ]] || die "Option --release-tag requires a value."
      VIBE_RELEASE_TAG="$2"
      shift 2
      ;;
    --archive-url)
      [[ "$#" -ge 2 ]] || die "Option --archive-url requires a value."
      VIBE_CLI_ARCHIVE_URL="$2"
      shift 2
      ;;
    --archive-path)
      [[ "$#" -ge 2 ]] || die "Option --archive-path requires a value."
      VIBE_CLI_ARCHIVE_PATH="$2"
      shift 2
      ;;
    --gh-proxy)
      [[ "$#" -ge 2 ]] || die "Option --gh-proxy requires a value."
      GH_PROXY="$2"
      shift 2
      ;;
    --no-gh-proxy)
      NO_GH_PROXY=true
      shift
      ;;
    --repo-owner)
      [[ "$#" -ge 2 ]] || die "Option --repo-owner requires a value."
      REPO_OWNER="$2"
      shift 2
      ;;
    --repo-name)
      [[ "$#" -ge 2 ]] || die "Option --repo-name requires a value."
      REPO_NAME="$2"
      shift 2
      ;;
    *)
      die "Unknown option: $1"
      ;;
  esac
done

INSTALL_COMPONENT="$(normalize_component "$INSTALL_COMPONENT")"

case "$COMMAND" in
  install|update)
    install_or_update "$COMMAND"
    ;;
  uninstall)
    uninstall_binaries
    ;;
  *)
    die "Unknown command: ${COMMAND}. Allowed commands: install, update, uninstall, help."
    ;;
esac
