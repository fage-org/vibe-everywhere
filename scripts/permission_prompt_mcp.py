#!/usr/bin/env python3
import json
import os
import sys
import time
import uuid
from pathlib import Path


SERVER_NAME = "ve_daemon"
TOOL_NAME = "permission_prompt"
DEFAULT_TIMEOUT_SECS = 120


def send(message: dict) -> None:
    sys.stdout.write(json.dumps(message) + "\n")
    sys.stdout.flush()


def make_error_response(request_id, code: int, message: str) -> dict:
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {"code": code, "message": message},
    }


def make_success_response(request_id, result: dict) -> dict:
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


def permission_tool_schema() -> dict:
    return {
        "name": TOOL_NAME,
        "description": "Permission bridge for Claude Code tool approval requests.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "tool_name": {"type": "string"},
                "input": {"type": "object"},
            },
            "required": ["tool_name", "input"],
            "additionalProperties": True,
        },
    }


def bridge_dirs() -> tuple[Path, Path]:
    root = Path(os.environ["VE_PERMISSION_BRIDGE_DIR"])
    requests_dir = root / "requests"
    responses_dir = root / "responses"
    requests_dir.mkdir(parents=True, exist_ok=True)
    responses_dir.mkdir(parents=True, exist_ok=True)
    return requests_dir, responses_dir


def handle_permission_prompt(arguments: dict) -> dict:
    session_id = os.environ["VE_PERMISSION_SESSION_ID"]
    timeout_secs = int(
        os.environ.get("VE_PERMISSION_BRIDGE_TIMEOUT_SECS", str(DEFAULT_TIMEOUT_SECS))
    )
    requests_dir, responses_dir = bridge_dirs()
    request_id = str(uuid.uuid4())

    request_path = requests_dir / f"{request_id}.json"
    response_path = responses_dir / f"{request_id}.json"

    payload = {
        "request_id": request_id,
        "session_id": session_id,
        "tool_name": arguments.get("tool_name"),
        "input": arguments.get("input", {}),
    }
    request_path.write_text(json.dumps(payload))

    deadline = time.time() + timeout_secs
    while time.time() < deadline:
        if response_path.exists():
            response = json.loads(response_path.read_text())
            response_path.unlink(missing_ok=True)
            return response
        time.sleep(0.25)

    return {
        "behavior": "deny",
        "message": "Timed out waiting for permission response from vibe-daemon bridge.",
    }


def main() -> int:
    for raw_line in sys.stdin:
        raw_line = raw_line.strip()
        if not raw_line:
            continue

        try:
            request = json.loads(raw_line)
        except json.JSONDecodeError:
            continue

        method = request.get("method")
        request_id = request.get("id")

        if method == "initialize":
            result = {
                "protocolVersion": request.get("params", {}).get(
                    "protocolVersion", "2024-11-05"
                ),
                "capabilities": {"tools": {}},
                "serverInfo": {"name": SERVER_NAME, "version": "0.1.0"},
            }
            send(make_success_response(request_id, result))
            continue

        if method == "notifications/initialized":
            continue

        if method == "tools/list":
            send(
                make_success_response(
                    request_id,
                    {
                        "tools": [permission_tool_schema()],
                    },
                )
            )
            continue

        if method == "tools/call":
            params = request.get("params", {})
            name = params.get("name")
            arguments = params.get("arguments", {})
            if name != TOOL_NAME:
                send(make_error_response(request_id, -32601, f"Unknown tool: {name}"))
                continue

            result_payload = handle_permission_prompt(arguments)
            send(
                make_success_response(
                    request_id,
                    {
                        "content": [
                            {"type": "text", "text": json.dumps(result_payload)}
                        ]
                    },
                )
            )
            continue

        if request_id is not None:
            send(make_error_response(request_id, -32601, f"Unsupported method: {method}"))

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
