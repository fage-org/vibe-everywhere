#!/usr/bin/env bash
set -euo pipefail

REPO_OWNER="${REPO_OWNER:-fage-ac-org}"
REPO_NAME="${REPO_NAME:-vibe-everywhere}"

INSTALL_ROOT="${INSTALL_ROOT:-/opt/vibe-everywhere}"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"
CONFIG_DIR="${CONFIG_DIR:-/etc/vibe-relay}"
STATE_DIR="${STATE_DIR:-/var/lib/vibe-relay}"
SERVICE_NAME="${SERVICE_NAME:-vibe-relay}"
SERVICE_FILE_PATH="/etc/systemd/system/${SERVICE_NAME}.service"

RELAY_BIND_HOST="${RELAY_BIND_HOST:-0.0.0.0}"
RELAY_PORT="${RELAY_PORT:-8787}"
RELAY_PUBLIC_BASE_URL="${RELAY_PUBLIC_BASE_URL:-}"
RELAY_FORWARD_HOST="${RELAY_FORWARD_HOST:-}"
RELAY_ACCESS_TOKEN="${RELAY_ACCESS_TOKEN:-}"
RELAY_ENROLLMENT_TOKEN="${RELAY_ENROLLMENT_TOKEN:-}"
RELAY_DEPLOYMENT_MODE="${RELAY_DEPLOYMENT_MODE:-self_hosted}"
STATE_FILE="${STATE_FILE:-${STATE_DIR}/relay-state.json}"

VIBE_RELEASE_TAG="${VIBE_RELEASE_TAG:-}"
RELAY_CLI_ARCHIVE_URL="${RELAY_CLI_ARCHIVE_URL:-}"
RELAY_CLI_ARCHIVE_PATH="${RELAY_CLI_ARCHIVE_PATH:-}"

CREATE_SYSTEMD_SERVICE="${CREATE_SYSTEMD_SERVICE:-1}"
ENABLE_AND_START_SERVICE="${ENABLE_AND_START_SERVICE:-1}"

require_root() {
  if [[ "${EUID}" -ne 0 ]]; then
    echo "This installer writes to system paths and must run as root." >&2
    exit 1
  fi
}

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

resolve_latest_release_tag() {
  curl -fsSL "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" | \
    python3 -c 'import json,sys; data=json.load(sys.stdin); print(data["tag_name"])'
}

derive_forward_host() {
  python3 - "$1" <<'PY'
import sys
from urllib.parse import urlparse

parsed = urlparse(sys.argv[1])
print(parsed.hostname or "")
PY
}

write_env_file() {
  local config_file="$1"

  mkdir -p "$(dirname "$config_file")"

  {
    printf 'VIBE_RELAY_HOST=%s\n' "$RELAY_BIND_HOST"
    printf 'VIBE_RELAY_PORT=%s\n' "$RELAY_PORT"
    printf 'VIBE_RELAY_DEPLOYMENT_MODE=%s\n' "$RELAY_DEPLOYMENT_MODE"
    printf 'VIBE_RELAY_STATE_FILE=%s\n' "$STATE_FILE"

    if [[ -n "$RELAY_PUBLIC_BASE_URL" ]]; then
      printf 'VIBE_PUBLIC_RELAY_BASE_URL=%s\n' "$RELAY_PUBLIC_BASE_URL"
    fi

    if [[ -n "$RELAY_FORWARD_HOST" ]]; then
      printf 'VIBE_RELAY_FORWARD_HOST=%s\n' "$RELAY_FORWARD_HOST"
    fi

    if [[ -n "$RELAY_ACCESS_TOKEN" ]]; then
      printf 'VIBE_RELAY_ACCESS_TOKEN=%s\n' "$RELAY_ACCESS_TOKEN"
    fi

    if [[ -n "$RELAY_ENROLLMENT_TOKEN" ]]; then
      printf 'VIBE_RELAY_ENROLLMENT_TOKEN=%s\n' "$RELAY_ENROLLMENT_TOKEN"
    fi
  } > "$config_file"
}

write_systemd_service() {
  local config_file="$1"
  local binary_path="$2"

  cat > "$SERVICE_FILE_PATH" <<EOF
[Unit]
Description=Vibe Everywhere Relay
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=${config_file}
WorkingDirectory=${STATE_DIR}
ExecStart=${binary_path}
Restart=always
RestartSec=2
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
EOF
}

main() {
  require_root
  require_command curl
  require_command python3
  require_command tar
  require_command install

  if [[ "$CREATE_SYSTEMD_SERVICE" == "1" ]]; then
    require_command systemctl
  fi

  if [[ -z "$RELAY_FORWARD_HOST" && -n "$RELAY_PUBLIC_BASE_URL" ]]; then
    RELAY_FORWARD_HOST="$(derive_forward_host "$RELAY_PUBLIC_BASE_URL")"
  fi

  local release_tag="$VIBE_RELEASE_TAG"
  if [[ -z "$release_tag" && -z "$RELAY_CLI_ARCHIVE_URL" && -z "$RELAY_CLI_ARCHIVE_PATH" ]]; then
    release_tag="$(resolve_latest_release_tag)"
  fi

  local archive_name=""
  local archive_source_path="$RELAY_CLI_ARCHIVE_PATH"
  if [[ -z "$archive_source_path" ]]; then
    if [[ -z "$RELAY_CLI_ARCHIVE_URL" ]]; then
      archive_name="vibe-everywhere-cli-${release_tag}-x86_64-unknown-linux-gnu.tar.gz"
      RELAY_CLI_ARCHIVE_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${release_tag}/${archive_name}"
    fi

    archive_source_path="$(mktemp /tmp/vibe-everywhere-cli.XXXXXX.tar.gz)"
    curl -fL --retry 5 --retry-all-errors "$RELAY_CLI_ARCHIVE_URL" -o "$archive_source_path"
  fi

  local extract_dir
  extract_dir="$(mktemp -d)"
  tar -xzf "$archive_source_path" -C "$extract_dir"

  mkdir -p "$INSTALL_ROOT" "$BIN_DIR" "$CONFIG_DIR" "$STATE_DIR"
  install -m 0755 "$extract_dir/vibe-relay" "${BIN_DIR}/vibe-relay"

  local config_file="${CONFIG_DIR}/relay.env"
  write_env_file "$config_file"

  if [[ "$CREATE_SYSTEMD_SERVICE" == "1" ]]; then
    write_systemd_service "$config_file" "${BIN_DIR}/vibe-relay"
    systemctl daemon-reload

    if [[ "$ENABLE_AND_START_SERVICE" == "1" ]]; then
      systemctl enable --now "$SERVICE_NAME"
    fi
  fi

  rm -rf "$extract_dir"

  printf 'Installed vibe-relay to %s\n' "${BIN_DIR}/vibe-relay"
  printf 'Environment file: %s\n' "$config_file"
  if [[ "$CREATE_SYSTEMD_SERVICE" == "1" ]]; then
    printf 'Systemd service: %s\n' "$SERVICE_FILE_PATH"
    if [[ "$ENABLE_AND_START_SERVICE" == "1" ]]; then
      printf 'Service status command: systemctl status %s\n' "$SERVICE_NAME"
      printf 'Service logs command: journalctl -u %s -f\n' "$SERVICE_NAME"
    fi
  fi

  if [[ -z "$RELAY_PUBLIC_BASE_URL" ]]; then
    echo "No RELAY_PUBLIC_BASE_URL was provided. Set VIBE_PUBLIC_RELAY_BASE_URL in ${config_file} before using remote/mobile clients."
  fi
}

main "$@"
