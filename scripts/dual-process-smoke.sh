#!/usr/bin/env bash
set -euo pipefail

MODE=${1:-relay_polling}
case "$MODE" in
  relay_polling|overlay)
    ;;
  *)
    echo "usage: $0 [relay_polling|overlay]" >&2
    exit 2
    ;;
esac

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/vibe-everywhere-dual-process.XXXXXX")
RELAY_PID=""
AGENT_PID=""
TARGET_SERVER_PID=""

stop_process() {
  local pid=${1:-}
  if [[ -z "$pid" ]]; then
    return 0
  fi
  if ! kill -0 "$pid" 2>/dev/null; then
    return 0
  fi
  kill -INT "$pid" 2>/dev/null || true
  for _ in $(seq 1 50); do
    if ! kill -0 "$pid" 2>/dev/null; then
      break
    fi
    sleep 0.1
  done
  if kill -0 "$pid" 2>/dev/null; then
    kill "$pid" 2>/dev/null || true
  fi
  wait "$pid" 2>/dev/null || true
}

cleanup() {
  local exit_code=$?
  set +e
  stop_process "$TARGET_SERVER_PID"
  stop_process "$AGENT_PID"
  stop_process "$RELAY_PID"
  if [[ $exit_code -ne 0 ]]; then
    echo "dual-process smoke test failed (mode=$MODE)" >&2
    if [[ -f "$TMP_DIR/relay.log" ]]; then
      echo "--- relay.log ---" >&2
      sed -n '1,300p' "$TMP_DIR/relay.log" >&2
    fi
    if [[ -f "$TMP_DIR/agent.log" ]]; then
      echo "--- agent.log ---" >&2
      sed -n '1,300p' "$TMP_DIR/agent.log" >&2
    fi
    if [[ -f "$TMP_DIR/target-server.log" ]]; then
      echo "--- target-server.log ---" >&2
      sed -n '1,200p' "$TMP_DIR/target-server.log" >&2
    fi
    echo "artifacts kept at $TMP_DIR" >&2
  elif [[ "${KEEP_SMOKE_ARTIFACTS:-0}" == "1" ]]; then
    echo "artifacts kept at $TMP_DIR"
  else
    rm -rf "$TMP_DIR"
  fi
  exit "$exit_code"
}
trap cleanup EXIT

if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

HOST=${VIBE_TEST_TCP_HOST:-$(python3 - <<'PY2'
import socket
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.bind(("0.0.0.0", 0))
sock.connect(("8.8.8.8", 53))
print(sock.getsockname()[0])
PY2
)}
OVERLAY_BOOTSTRAP_HOST=""
OVERLAY_AGENT_NODE_IP=""

pick_port() {
  python3 - "$1" <<'PY2'
import socket, sys
host = sys.argv[1]
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
sock.bind((host, 0))
print(sock.getsockname()[1])
PY2
}

RELAY_PORT=$(pick_port "$HOST")
BASE_URL="http://$HOST:$RELAY_PORT"
DEVICE_ID="smoke-agent"
DEVICE_NAME="Dual Process Smoke Agent"
FAKE_CODEX="$TMP_DIR/fake-codex.sh"
EXPECTED_TRANSPORT="relay_polling"
RELAY_EXTRA_ENV=()
AGENT_EXTRA_ENV=()

if [[ "$MODE" == "overlay" ]]; then
  EASYTIER_PORT=$(pick_port "$HOST")
  EASYTIER_NETWORK_NAME="vibe-smoke-net"
  EASYTIER_SECRET="vibe-smoke-secret"
  OVERLAY_RECOVERY_COOLDOWN_MS=${VIBE_TEST_OVERLAY_RECOVERY_COOLDOWN_MS:-250}
  OVERLAY_PROBE_INTERVAL_MS=${VIBE_TEST_OVERLAY_PROBE_INTERVAL_MS:-250}
  # Keep the bootstrap path deterministic for same-host CI while leaving product defaults untouched.
  OVERLAY_BOOTSTRAP_HOST=${VIBE_TEST_OVERLAY_BOOTSTRAP_HOST:-127.0.0.1}
  OVERLAY_AGENT_NODE_IP=${VIBE_TEST_OVERLAY_NODE_IP:-10.44.0.2/24}
  EXPECTED_TRANSPORT="overlay_proxy"
  RELAY_EXTRA_ENV=(
    VIBE_EASYTIER_RELAY_ENABLED=1
    VIBE_EASYTIER_NETWORK_NAME="$EASYTIER_NETWORK_NAME"
    VIBE_EASYTIER_NETWORK_SECRET="$EASYTIER_SECRET"
    VIBE_EASYTIER_LISTENERS="tcp:$EASYTIER_PORT"
    VIBE_OVERLAY_BRIDGE_RECOVERY_COOLDOWN_MS="$OVERLAY_RECOVERY_COOLDOWN_MS"
    VIBE_OVERLAY_BRIDGE_PROBE_INTERVAL_MS="$OVERLAY_PROBE_INTERVAL_MS"
  )
  AGENT_EXTRA_ENV=(
    VIBE_EASYTIER_NETWORK_NAME="$EASYTIER_NETWORK_NAME"
    VIBE_EASYTIER_NETWORK_SECRET="$EASYTIER_SECRET"
    VIBE_EASYTIER_BOOTSTRAP_URL="tcp://$OVERLAY_BOOTSTRAP_HOST:$EASYTIER_PORT"
    VIBE_EASYTIER_NODE_IP="$OVERLAY_AGENT_NODE_IP"
    VIBE_EASYTIER_NO_LISTENER=true
  )
fi

echo "building vibe-agent and vibe-relay binaries"
cargo build -p vibe-relay -p vibe-agent >/dev/null

cat >"$FAKE_CODEX" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "--version" ]]; then
  echo "fake-codex 0.1.0"
  exit 0
fi
printf '%s\n' '{"type":"thread.started","thread_id":"thread_smoke"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"item_0","type":"agent_message","text":"dual-process smoke ok"}}'
EOF
chmod +x "$FAKE_CODEX"

echo "starting vibe-relay on $BASE_URL (mode=$MODE)"
(
  cd "$ROOT_DIR"
  env \
    VIBE_RELAY_HOST="$HOST" \
    VIBE_RELAY_PORT="$RELAY_PORT" \
    VIBE_PUBLIC_RELAY_BASE_URL="$BASE_URL" \
    VIBE_RELAY_STATE_FILE="$TMP_DIR/relay-state.json" \
    VIBE_RELAY_FORWARD_HOST="$HOST" \
    VIBE_RELAY_FORWARD_BIND_HOST="$HOST" \
    "${RELAY_EXTRA_ENV[@]}" \
    target/debug/vibe-relay
) >"$TMP_DIR/relay.log" 2>&1 &
RELAY_PID=$!

for _ in $(seq 1 100); do
  if curl -fsS "$BASE_URL/api/health" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done
curl -fsS "$BASE_URL/api/health" >"$TMP_DIR/health.json"

echo "starting vibe-agent"
(
  cd "$ROOT_DIR"
  env \
    VIBE_RELAY_URL="$BASE_URL" \
    VIBE_DEVICE_ID="$DEVICE_ID" \
    VIBE_DEVICE_NAME="$DEVICE_NAME" \
    VIBE_WORKING_ROOT="$ROOT_DIR" \
    VIBE_POLL_INTERVAL_MS=200 \
    VIBE_HEARTBEAT_INTERVAL_MS=500 \
    VIBE_CODEX_COMMAND="$FAKE_CODEX" \
    "${AGENT_EXTRA_ENV[@]}" \
    target/debug/vibe-agent
) >"$TMP_DIR/agent.log" 2>&1 &
AGENT_PID=$!

echo "waiting for agent registration"
registered=0
for _ in $(seq 1 100); do
  curl -fsS "$BASE_URL/api/devices" >"$TMP_DIR/devices.json"
  if python3 - "$DEVICE_ID" "$TMP_DIR/devices.json" >"$TMP_DIR/device.json" <<'PY2'
import json, sys
needle = sys.argv[1]
devices = json.load(open(sys.argv[2], 'r', encoding='utf-8'))
for device in devices:
    if device.get("id") == needle and device.get("online"):
        providers = device.get("providers", [])
        if any(provider.get("kind") == "codex" and provider.get("available") for provider in providers):
            print(json.dumps(device))
            raise SystemExit(0)
raise SystemExit(1)
PY2
  then
    registered=1
    break
  fi
  sleep 0.2
done
if [[ "$registered" != "1" ]]; then
  echo "agent did not register in time" >&2
  exit 1
fi

if [[ "$MODE" == "overlay" ]]; then
  echo "waiting for overlay connectivity"
  overlay_ready=0
  for _ in $(seq 1 120); do
    curl -fsS "$BASE_URL/api/devices" >"$TMP_DIR/devices.json"
    if python3 - "$DEVICE_ID" "$TMP_DIR/devices.json" >"$TMP_DIR/overlay-summary.json" <<'PY2'
import json, sys
needle = sys.argv[1]
devices = json.load(open(sys.argv[2], 'r', encoding='utf-8'))
for device in devices:
    if device.get("id") != needle:
        continue
    overlay = device.get("overlay", {})
    node_ip = overlay.get("nodeIp")
    summary = {
        "deviceId": device.get("id"),
        "overlayState": overlay.get("state"),
        "nodeIp": node_ip,
        "relayUrl": overlay.get("relayUrl"),
        "lastError": overlay.get("lastError"),
        "networkName": overlay.get("networkName"),
    }
    print(json.dumps(summary, ensure_ascii=False))
    if overlay.get("state") == "connected" and isinstance(node_ip, str) and node_ip.strip():
        raise SystemExit(0)
raise SystemExit(1)
PY2
    then
      cp "$TMP_DIR/overlay-summary.json" "$TMP_DIR/device.json"
      overlay_ready=1
      break
    fi
    sleep 0.5
  done
  if [[ "$overlay_ready" != "1" ]]; then
    if [[ -f "$TMP_DIR/overlay-summary.json" ]]; then
      echo "last overlay summary:" >&2
      cat "$TMP_DIR/overlay-summary.json" >&2
    fi
    echo "overlay did not become ready in time" >&2
    exit 1
  fi
fi

create_smoke_task() {
  local prompt=$1
  local title=$2
  local create_path=$3
  local detail_path=$4
  local task_payload task_id status

  task_payload=$(python3 - "$DEVICE_ID" "$prompt" "$title" <<'PY2'
import json, sys
print(json.dumps({
    "deviceId": sys.argv[1],
    "provider": "codex",
    "prompt": sys.argv[2],
    "cwd": None,
    "model": None,
    "title": sys.argv[3],
}))
PY2
)
  curl -fsS -H 'content-type: application/json' -d "$task_payload" "$BASE_URL/api/tasks" >"$create_path"
  task_id=$(python3 - "$create_path" <<'PY2'
import json, sys
print(json.load(open(sys.argv[1], 'r', encoding='utf-8'))["task"]["id"])
PY2
)

  echo "waiting for task $task_id"
  status=""
  for _ in $(seq 1 180); do
    curl -fsS "$BASE_URL/api/tasks/$task_id" >"$detail_path"
    status=$(python3 - "$detail_path" <<'PY2'
import json, sys
print(json.load(open(sys.argv[1], 'r', encoding='utf-8'))["task"]["status"])
PY2
)
    case "$status" in
      succeeded)
        break
        ;;
      failed|canceled)
        echo "task reached terminal failure state: $status" >&2
        cat "$detail_path" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$status" != "succeeded" ]]; then
    echo "task did not finish successfully in time" >&2
    cat "$detail_path" >&2
    exit 1
  fi

}

task_transport_from_detail() {
  python3 - "$1" <<'PY2'
import json, sys
print(json.load(open(sys.argv[1], 'r', encoding='utf-8'))["task"]["transport"])
PY2
}

assert_task_succeeded() {
  python3 - "$1" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
task = payload["task"]
events = payload["events"]
assert task["status"] == "succeeded", task
assert task["exitCode"] == 0, task
messages = [event["message"] for event in events]
assert "dual-process smoke ok" in messages, messages
print(json.dumps({
    "taskId": task["id"],
    "transport": task["transport"],
    "eventCount": len(events),
}, ensure_ascii=False))
PY2
}

echo "creating task"
create_smoke_task \
  "Say hello from the dual-process smoke test" \
  "Dual process smoke task" \
  "$TMP_DIR/create-task.json" \
  "$TMP_DIR/task-detail.json"
assert_task_succeeded "$TMP_DIR/task-detail.json"
TASK_TRANSPORT=$(task_transport_from_detail "$TMP_DIR/task-detail.json")

if [[ "$MODE" != "overlay" ]]; then
  if [[ "$TASK_TRANSPORT" != "$EXPECTED_TRANSPORT" ]]; then
    echo "task used unexpected transport in relay polling mode: $TASK_TRANSPORT" >&2
    cat "$TMP_DIR/task-detail.json" >&2
    exit 1
  fi
else
  case "$TASK_TRANSPORT" in
    overlay_proxy)
      ;;
    relay_polling)
      echo "overlay task fell back to relay polling; waiting for task bridge recovery"
      overlay_task_recovered=0
      for attempt in $(seq 1 5); do
        sleep 2
        echo "creating overlay recovery task attempt $attempt"
        create_smoke_task \
          "Say hello from the overlay recovery smoke test attempt $attempt" \
          "Dual process smoke recovery task $attempt" \
          "$TMP_DIR/create-recovery-task.json" \
          "$TMP_DIR/task-recovery-detail.json"
        assert_task_succeeded "$TMP_DIR/task-recovery-detail.json"
        recovery_transport=$(task_transport_from_detail "$TMP_DIR/task-recovery-detail.json")
        if [[ "$recovery_transport" == "overlay_proxy" ]]; then
          cp "$TMP_DIR/task-recovery-detail.json" "$TMP_DIR/task-detail.json"
          overlay_task_recovered=1
          break
        fi
        echo "overlay recovery task attempt $attempt still used $recovery_transport"
      done
      if [[ "$overlay_task_recovered" != "1" ]]; then
        echo "overlay task bridge did not recover to overlay_proxy in time" >&2
        cat "$TMP_DIR/task-recovery-detail.json" >&2
        exit 1
      fi
      ;;
    *)
      echo "task used unexpected transport in overlay mode: $TASK_TRANSPORT" >&2
      cat "$TMP_DIR/task-detail.json" >&2
      exit 1
      ;;
  esac
fi

if [[ "$MODE" == "overlay" ]]; then
  echo "creating overlay shell session"
  SHELL_PAYLOAD=$(python3 - "$DEVICE_ID" <<'PY2'
import json, sys
print(json.dumps({
    "deviceId": sys.argv[1],
    "cwd": None,
}))
PY2
  )
  curl -fsS -H 'content-type: application/json' -d "$SHELL_PAYLOAD" "$BASE_URL/api/shell/sessions" >"$TMP_DIR/create-shell.json"
  SHELL_ID=$(python3 - "$TMP_DIR/create-shell.json" <<'PY2'
import json, sys
print(json.load(open(sys.argv[1], 'r', encoding='utf-8'))["session"]["id"])
PY2
  )

  echo "waiting for shell session $SHELL_ID to become active"
  shell_status=""
  shell_transport=""
  for _ in $(seq 1 150); do
    curl -fsS "$BASE_URL/api/shell/sessions/$SHELL_ID" >"$TMP_DIR/shell-detail.json"
    readarray -t shell_info < <(python3 - "$TMP_DIR/shell-detail.json" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
session = payload["session"]
print(session["status"])
print(session["transport"])
PY2
    )
    shell_status=${shell_info[0]}
    shell_transport=${shell_info[1]}
    if [[ "$shell_transport" != "overlay_proxy" ]]; then
      echo "shell session fell back to unexpected transport: $shell_transport" >&2
      cat "$TMP_DIR/shell-detail.json" >&2
      exit 1
    fi
    case "$shell_status" in
      active)
        break
        ;;
      failed|closed)
        echo "shell session reached unexpected terminal state before input: $shell_status" >&2
        cat "$TMP_DIR/shell-detail.json" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$shell_status" != "active" ]]; then
    echo "shell session did not become active in time" >&2
    cat "$TMP_DIR/shell-detail.json" >&2
    exit 1
  fi

  echo "sending shell input"
  SHELL_INPUT_PAYLOAD=$(python3 - <<'PY2'
import json
print(json.dumps({
    "data": "printf '__VIBE_SHELL_SMOKE__\n'; exit\n",
}))
PY2
  )
  curl -fsS -H 'content-type: application/json' -d "$SHELL_INPUT_PAYLOAD" "$BASE_URL/api/shell/sessions/$SHELL_ID/input" >"$TMP_DIR/append-shell-input.json"

  echo "waiting for shell session $SHELL_ID output and completion"
  shell_status=""
  shell_transport=""
  shell_marker_found=0
  for _ in $(seq 1 180); do
    curl -fsS "$BASE_URL/api/shell/sessions/$SHELL_ID" >"$TMP_DIR/shell-detail.json"
    if python3 - "$TMP_DIR/shell-detail.json" >"$TMP_DIR/shell-summary.json" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
session = payload["session"]
outputs = payload["outputs"]
marker = any("__VIBE_SHELL_SMOKE__" in output.get("data", "") for output in outputs)
print(json.dumps({
    "status": session["status"],
    "transport": session["transport"],
    "marker": marker,
    "outputCount": len(outputs),
}, ensure_ascii=False))
if marker:
    raise SystemExit(0)
raise SystemExit(1)
PY2
    then
      shell_marker_found=1
    fi
    readarray -t shell_info < <(python3 - "$TMP_DIR/shell-detail.json" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
session = payload["session"]
print(session["status"])
print(session["transport"])
PY2
    )
    shell_status=${shell_info[0]}
    shell_transport=${shell_info[1]}
    if [[ "$shell_transport" != "overlay_proxy" ]]; then
      echo "shell session transport changed unexpectedly: $shell_transport" >&2
      cat "$TMP_DIR/shell-detail.json" >&2
      exit 1
    fi
    case "$shell_status" in
      succeeded)
        if [[ "$shell_marker_found" == "1" ]]; then
          break
        fi
        ;;
      failed|closed)
        echo "shell session reached terminal failure state: $shell_status" >&2
        cat "$TMP_DIR/shell-detail.json" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$shell_status" != "succeeded" || "$shell_marker_found" != "1" ]]; then
    echo "shell session did not produce marker and succeed in time" >&2
    cat "$TMP_DIR/shell-detail.json" >&2
    exit 1
  fi

  python3 - "$TMP_DIR/shell-detail.json" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
session = payload["session"]
outputs = payload["outputs"]
assert session["status"] == "succeeded", session
assert session["transport"] == "overlay_proxy", session
assert any("__VIBE_SHELL_SMOKE__" in output.get("data", "") for output in outputs), outputs
print(json.dumps({
    "shellSessionId": session["id"],
    "transport": session["transport"],
    "outputCount": len(outputs),
}, ensure_ascii=False))
PY2

  TARGET_PORT=$(pick_port "$HOST")
  echo "starting overlay port-forward target on $HOST:$TARGET_PORT"
  python3 - "$HOST" "$TARGET_PORT" "$TMP_DIR/target-server.log" <<'PY2' &
import socket, sys

host = sys.argv[1]
port = int(sys.argv[2])
log_path = sys.argv[3]

listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind((host, port))
listener.listen(1)

with open(log_path, "w", encoding="utf-8") as log_file:
    log_file.write(f"listening {host}:{port}\n")
    log_file.flush()
    conn, remote = listener.accept()
    with conn:
        log_file.write(f"accepted {remote[0]}:{remote[1]}\n")
        log_file.flush()
        chunks = []
        while True:
            data = conn.recv(65536)
            if not data:
                break
            chunks.append(data)
        payload = b"".join(chunks)
        log_file.write(f"received {payload!r}\n")
        log_file.flush()
        conn.sendall(b"__VIBE_PORT_FORWARD_REPLY__:" + payload)
        log_file.write("replied\n")
        log_file.flush()
PY2
  TARGET_SERVER_PID=$!

  target_ready=0
  for _ in $(seq 1 50); do
    if [[ -f "$TMP_DIR/target-server.log" ]]; then
      target_ready=1
      break
    fi
    sleep 0.1
  done
  if [[ "$target_ready" != "1" ]]; then
    echo "target server did not start in time" >&2
    exit 1
  fi

  echo "creating overlay port forward"
  PORT_FORWARD_PAYLOAD=$(python3 - "$DEVICE_ID" "$HOST" "$TARGET_PORT" <<'PY2'
import json, sys
print(json.dumps({
    "deviceId": sys.argv[1],
    "protocol": "tcp",
    "targetHost": sys.argv[2],
    "targetPort": int(sys.argv[3]),
}))
PY2
  )
  curl -fsS -H 'content-type: application/json' -d "$PORT_FORWARD_PAYLOAD" "$BASE_URL/api/port-forwards" >"$TMP_DIR/create-port-forward.json"
  readarray -t port_forward_info < <(python3 - "$TMP_DIR/create-port-forward.json" <<'PY2'
import json, sys
forward = json.load(open(sys.argv[1], 'r', encoding='utf-8'))["forward"]
print(forward["id"])
print(forward["relayHost"])
print(forward["relayPort"])
PY2
  )
  PORT_FORWARD_ID=${port_forward_info[0]}
  PORT_FORWARD_RELAY_HOST=${port_forward_info[1]}
  PORT_FORWARD_RELAY_PORT=${port_forward_info[2]}

  echo "waiting for port forward $PORT_FORWARD_ID to become active"
  port_forward_status=""
  port_forward_transport=""
  for _ in $(seq 1 150); do
    curl -fsS "$BASE_URL/api/port-forwards/$PORT_FORWARD_ID" >"$TMP_DIR/port-forward-detail.json"
    readarray -t port_forward_state < <(python3 - "$TMP_DIR/port-forward-detail.json" <<'PY2'
import json, sys
forward = json.load(open(sys.argv[1], 'r', encoding='utf-8'))["forward"]
print(forward["status"])
print(forward["transport"])
PY2
    )
    port_forward_status=${port_forward_state[0]}
    port_forward_transport=${port_forward_state[1]}
    if [[ "$port_forward_transport" != "overlay_proxy" ]]; then
      echo "port forward fell back to unexpected transport: $port_forward_transport" >&2
      cat "$TMP_DIR/port-forward-detail.json" >&2
      exit 1
    fi
    case "$port_forward_status" in
      active)
        break
        ;;
      failed|closed)
        echo "port forward reached unexpected terminal state before traffic: $port_forward_status" >&2
        cat "$TMP_DIR/port-forward-detail.json" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$port_forward_status" != "active" ]]; then
    echo "port forward did not become active in time" >&2
    cat "$TMP_DIR/port-forward-detail.json" >&2
    exit 1
  fi

  echo "verifying overlay port forward data path via $PORT_FORWARD_RELAY_HOST:$PORT_FORWARD_RELAY_PORT"
  python3 - "$PORT_FORWARD_RELAY_HOST" "$PORT_FORWARD_RELAY_PORT" >"$TMP_DIR/port-forward-client.json" <<'PY2'
import json, socket, sys

host = sys.argv[1]
port = int(sys.argv[2])
payload = b"__VIBE_PORT_FORWARD_SMOKE__"
expected = b"__VIBE_PORT_FORWARD_REPLY__:" + payload

with socket.create_connection((host, port), timeout=10) as sock:
    sock.settimeout(10)
    sock.sendall(payload)
    sock.shutdown(socket.SHUT_WR)
    chunks = []
    while True:
        data = sock.recv(65536)
        if not data:
            break
        chunks.append(data)

reply = b"".join(chunks)
if reply != expected:
    raise SystemExit(f"unexpected port-forward reply: {reply!r}")

print(json.dumps({
    "bytesSent": len(payload),
    "bytesReceived": len(reply),
    "replyPreview": reply.decode("utf-8"),
}, ensure_ascii=False))
PY2

  echo "closing port forward $PORT_FORWARD_ID"
  curl -fsS -X POST "$BASE_URL/api/port-forwards/$PORT_FORWARD_ID/close" >"$TMP_DIR/close-port-forward.json"

  port_forward_status=""
  port_forward_transport=""
  for _ in $(seq 1 150); do
    curl -fsS "$BASE_URL/api/port-forwards/$PORT_FORWARD_ID" >"$TMP_DIR/port-forward-detail.json"
    readarray -t port_forward_state < <(python3 - "$TMP_DIR/port-forward-detail.json" <<'PY2'
import json, sys
forward = json.load(open(sys.argv[1], 'r', encoding='utf-8'))["forward"]
print(forward["status"])
print(forward["transport"])
PY2
    )
    port_forward_status=${port_forward_state[0]}
    port_forward_transport=${port_forward_state[1]}
    if [[ "$port_forward_transport" != "overlay_proxy" ]]; then
      echo "port forward transport changed unexpectedly during close: $port_forward_transport" >&2
      cat "$TMP_DIR/port-forward-detail.json" >&2
      exit 1
    fi
    case "$port_forward_status" in
      closed)
        break
        ;;
      failed)
        echo "port forward failed during close" >&2
        cat "$TMP_DIR/port-forward-detail.json" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$port_forward_status" != "closed" ]]; then
    echo "port forward did not close in time" >&2
    cat "$TMP_DIR/port-forward-detail.json" >&2
    exit 1
  fi

  python3 - "$TMP_DIR/port-forward-detail.json" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
forward = payload["forward"]
assert forward["status"] == "closed", forward
assert forward["transport"] == "overlay_proxy", forward
print(json.dumps({
    "portForwardId": forward["id"],
    "transport": forward["transport"],
    "status": forward["status"],
    "relayPort": forward["relayPort"],
}, ensure_ascii=False))
PY2
fi

if [[ "$MODE" != "overlay" ]]; then
  TARGET_PORT=$(pick_port "$HOST")
  echo "starting relay-tunnel port-forward target on $HOST:$TARGET_PORT"
  python3 - "$HOST" "$TARGET_PORT" "$TMP_DIR/target-server.log" <<'PY2' &
import socket, sys

host = sys.argv[1]
port = int(sys.argv[2])
log_path = sys.argv[3]

listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind((host, port))
listener.listen(1)

with open(log_path, "w", encoding="utf-8") as log_file:
    log_file.write(f"listening {host}:{port}\n")
    log_file.flush()
    conn, remote = listener.accept()
    with conn:
        log_file.write(f"accepted {remote[0]}:{remote[1]}\n")
        log_file.flush()
        payload = conn.recv(65536)
        log_file.write(f"received {payload!r}\n")
        log_file.flush()
        conn.sendall(b"__VIBE_PORT_FORWARD_REPLY__:" + payload)
        log_file.write("replied\n")
        log_file.flush()
PY2
  TARGET_SERVER_PID=$!

  target_ready=0
  for _ in $(seq 1 50); do
    if [[ -f "$TMP_DIR/target-server.log" ]]; then
      target_ready=1
      break
    fi
    sleep 0.1
  done
  if [[ "$target_ready" != "1" ]]; then
    echo "relay-tunnel target server did not start in time" >&2
    exit 1
  fi

  echo "creating relay-tunnel port forward"
  PORT_FORWARD_PAYLOAD=$(python3 - "$DEVICE_ID" "$HOST" "$TARGET_PORT" <<'PY2'
import json, sys
print(json.dumps({
    "deviceId": sys.argv[1],
    "protocol": "tcp",
    "targetHost": sys.argv[2],
    "targetPort": int(sys.argv[3]),
}))
PY2
  )
  curl -fsS -H 'content-type: application/json' -d "$PORT_FORWARD_PAYLOAD" "$BASE_URL/api/port-forwards" >"$TMP_DIR/create-port-forward.json"
  readarray -t port_forward_info < <(python3 - "$TMP_DIR/create-port-forward.json" <<'PY2'
import json, sys
forward = json.load(open(sys.argv[1], 'r', encoding='utf-8'))["forward"]
print(forward["id"])
print(forward["relayHost"])
print(forward["relayPort"])
print(forward["transport"])
PY2
  )
  PORT_FORWARD_ID=${port_forward_info[0]}
  PORT_FORWARD_RELAY_HOST=${port_forward_info[1]}
  PORT_FORWARD_RELAY_PORT=${port_forward_info[2]}
  PORT_FORWARD_TRANSPORT=${port_forward_info[3]}
  if [[ "$PORT_FORWARD_TRANSPORT" != "relay_tunnel" ]]; then
    echo "port forward was created with unexpected transport: $PORT_FORWARD_TRANSPORT" >&2
    cat "$TMP_DIR/create-port-forward.json" >&2
    exit 1
  fi

  echo "waiting for port forward $PORT_FORWARD_ID to become active"
  port_forward_status=""
  port_forward_transport=""
  for _ in $(seq 1 150); do
    curl -fsS "$BASE_URL/api/port-forwards/$PORT_FORWARD_ID" >"$TMP_DIR/port-forward-detail.json"
    readarray -t port_forward_state < <(python3 - "$TMP_DIR/port-forward-detail.json" <<'PY2'
import json, sys
forward = json.load(open(sys.argv[1], 'r', encoding='utf-8'))["forward"]
print(forward["status"])
print(forward["transport"])
PY2
    )
    port_forward_status=${port_forward_state[0]}
    port_forward_transport=${port_forward_state[1]}
    if [[ "$port_forward_transport" != "relay_tunnel" ]]; then
      echo "port forward transport changed unexpectedly before traffic: $port_forward_transport" >&2
      cat "$TMP_DIR/port-forward-detail.json" >&2
      exit 1
    fi
    case "$port_forward_status" in
      active)
        break
        ;;
      failed|closed)
        echo "relay-tunnel port forward reached unexpected terminal state before traffic: $port_forward_status" >&2
        cat "$TMP_DIR/port-forward-detail.json" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$port_forward_status" != "active" ]]; then
    echo "relay-tunnel port forward did not become active in time" >&2
    cat "$TMP_DIR/port-forward-detail.json" >&2
    exit 1
  fi

  echo "verifying relay-tunnel port forward data path via $PORT_FORWARD_RELAY_HOST:$PORT_FORWARD_RELAY_PORT"
  python3 - "$PORT_FORWARD_RELAY_HOST" "$PORT_FORWARD_RELAY_PORT" >"$TMP_DIR/port-forward-client.json" <<'PY2'
import json, socket, sys

host = sys.argv[1]
port = int(sys.argv[2])
payload = b"__VIBE_PORT_FORWARD_SMOKE__"
expected = b"__VIBE_PORT_FORWARD_REPLY__:" + payload

with socket.create_connection((host, port), timeout=10) as sock:
    sock.settimeout(10)
    sock.sendall(payload)
    chunks = []
    total = 0
    while total < len(expected):
        data = sock.recv(65536)
        if not data:
            break
        chunks.append(data)
        total += len(data)

reply = b"".join(chunks)
if reply != expected:
    raise SystemExit(f"unexpected port-forward reply: {reply!r}")

print(json.dumps({
    "bytesSent": len(payload),
    "bytesReceived": len(reply),
    "replyPreview": reply.decode("utf-8"),
}, ensure_ascii=False))
PY2

  echo "closing port forward $PORT_FORWARD_ID"
  curl -fsS -X POST "$BASE_URL/api/port-forwards/$PORT_FORWARD_ID/close" >"$TMP_DIR/close-port-forward.json"

  port_forward_status=""
  port_forward_transport=""
  for _ in $(seq 1 150); do
    curl -fsS "$BASE_URL/api/port-forwards/$PORT_FORWARD_ID" >"$TMP_DIR/port-forward-detail.json"
    readarray -t port_forward_state < <(python3 - "$TMP_DIR/port-forward-detail.json" <<'PY2'
import json, sys
forward = json.load(open(sys.argv[1], 'r', encoding='utf-8'))["forward"]
print(forward["status"])
print(forward["transport"])
PY2
    )
    port_forward_status=${port_forward_state[0]}
    port_forward_transport=${port_forward_state[1]}
    if [[ "$port_forward_transport" != "relay_tunnel" ]]; then
      echo "port forward transport changed unexpectedly during close: $port_forward_transport" >&2
      cat "$TMP_DIR/port-forward-detail.json" >&2
      exit 1
    fi
    case "$port_forward_status" in
      closed)
        break
        ;;
      failed)
        echo "relay-tunnel port forward failed during close" >&2
        cat "$TMP_DIR/port-forward-detail.json" >&2
        exit 1
        ;;
    esac
    sleep 0.2
  done
  if [[ "$port_forward_status" != "closed" ]]; then
    echo "relay-tunnel port forward did not close in time" >&2
    cat "$TMP_DIR/port-forward-detail.json" >&2
    exit 1
  fi

  python3 - "$TMP_DIR/port-forward-detail.json" <<'PY2'
import json, sys
payload = json.load(open(sys.argv[1], 'r', encoding='utf-8'))
forward = payload["forward"]
assert forward["status"] == "closed", forward
assert forward["transport"] == "relay_tunnel", forward
print(json.dumps({
    "portForwardId": forward["id"],
    "transport": forward["transport"],
    "status": forward["status"],
    "relayPort": forward["relayPort"],
}, ensure_ascii=False))
PY2
fi

echo "dual-process smoke test passed (mode=$MODE)"
