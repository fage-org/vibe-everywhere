# Vibe Everywhere

[![CI](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml)
[![Release](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml/badge.svg)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[中文](./README.md) | [English](./README.en.md)

Vibe Everywhere is a self-hosted remote AI control plane. The system consists of `vibe-relay`,
`vibe-agent`, and control clients. It is used to run long-lived AI coding conversations on remote
machines and to manage device registration, conversation routing, workspace browsing, Git
inspection, preview access, and advanced connection paths through one control surface.

This document is written for end users and operators. It provides a system overview, binary
installation entry points, relay startup references, key configuration semantics, and the standard
usage flow.

## Overview

The standard workflow is:

1. Deploy `vibe-relay` on a host reachable by clients and agents.
2. Start `vibe-agent` on target execution nodes.
3. Connect a desktop, Android, or self-hosted Web client to the relay.
4. Select a device and start a long-lived AI conversation.
5. Continue follow-up turns, answer tool choices, and review workspace state, Git state, previews,
   and execution results from the same control plane.

## Components

| Component | Purpose | Typical Location |
| --- | --- | --- |
| `vibe-relay` | Control-plane entry point for auth, device registration, conversation / task routing, aggregation, and public APIs | Server, workstation, cloud host |
| `vibe-agent` | Runtime on the target machine for provider execution, workspace access, Git inspection, preview bridging, and advanced connections | Machine that runs AI work |
| Control client | Connects to the relay and manages long-lived AI conversations, devices, and results | Desktop, Android, self-hosted Web client |

## Supported Capabilities

The current release supports:

- creation, continuation, cancellation, and streaming of long-lived AI conversations
- device registration, presence reporting, and provider availability display
- workspace browsing and text-file preview
- Git status, changed-file, and recent-commit inspection
- preview access, shell sessions, and advanced connection capabilities
- inline provider choice prompts with preset options plus custom text input
- English and Simplified Chinese UI
- light, dark, and system theme modes

## Quick Start

### Prerequisites

Prepare the following values before deployment:

- the client-facing relay address, for example `https://relay.example.com` or `http://203.0.113.10:8787`
- the control-plane token used by human users: `VIBE_RELAY_ACCESS_TOKEN`
- the enrollment token used by agents during first registration: `VIBE_RELAY_ENROLLMENT_TOKEN`
- at least one provider CLI on each target machine, such as `codex`, `claude`, or `opencode`

### 1. Download or Update the CLI Binaries

`scripts/install-relay.sh` and `scripts/install-relay.ps1` install, update, or uninstall the CLI
binaries. By default they install both `vibe-relay` and `vibe-agent`, and they can manage a single
component through `--component` or `-Component`.

#### Linux

Direct GitHub access:

```bash
curl -fsSL https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
bash install-relay.sh install --no-gh-proxy
```

Recommended for mainland China network paths:

```bash
curl -fsSL https://ghfast.top/https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
bash install-relay.sh install
```

Common commands:

```bash
bash install-relay.sh install
bash install-relay.sh install --component relay
bash install-relay.sh install --component agent
bash install-relay.sh update --release-tag v0.1.10
bash install-relay.sh uninstall
bash install-relay.sh uninstall --component agent
```

Default installed paths:

- `/usr/local/bin/vibe-relay`
- `/usr/local/bin/vibe-agent`

Notes:

- The published Linux x86_64 CLI archive uses statically linked `x86_64-unknown-linux-musl`
  binaries, so it does not require the target host to provide a matching `glibc` version.

#### Windows

Direct GitHub access:

```powershell
Invoke-WebRequest `
  -Uri "https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.ps1" `
  -OutFile ".\install-relay.ps1"
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install -NoGhProxy
```

Recommended for mainland China network paths:

```powershell
Invoke-WebRequest `
  -Uri "https://ghfast.top/https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.ps1" `
  -OutFile ".\install-relay.ps1"
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install
```

Common commands:

```powershell
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install -Component relay
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command install -Component agent
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command update -ReleaseTag v0.1.10
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command uninstall
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command uninstall -Component agent
```

Default installed paths:

- `C:\Program Files\Vibe Everywhere\vibe-relay.exe`
- `C:\Program Files\Vibe Everywhere\vibe-agent.exe`
- `C:\Program Files\Vibe Everywhere\Packet.dll`
- `C:\Program Files\Vibe Everywhere\wintun.dll`
- `C:\Program Files\Vibe Everywhere\WinDivert64.sys`
- `C:\Program Files\Vibe Everywhere\WinDivert.dll` when present in the archive

Notes:

- On Windows, `vibe-relay.exe` and `vibe-agent.exe` must stay beside the side-by-side runtime files.

Acceleration notes:

- both install scripts default to `https://ghfast.top/` as the GitHub URL prefix, so the internal
  release resolution and archive download paths are accelerated as well
- if direct GitHub access is preferable in your environment, use `--no-gh-proxy` on Linux or
  `-NoGhProxy` on Windows
- to use a different accelerator, pass `--gh-proxy <url>` on Linux or `-GhProxy <url>` on Windows

### 2. Configure and Start the Relay

Dedicated startup guides:

- Chinese: [docs/relay-startup.zh-CN.md](./docs/relay-startup.zh-CN.md)
- English: [docs/relay-startup.md](./docs/relay-startup.md)

Minimum foreground startup example:

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

Health check:

```bash
curl https://relay.example.com/api/health
```

### 3. Start an Agent

If the CLI was installed with the installer script, start the installed `vibe-agent` directly. If
you do not use the installer script, you can still download the CLI package from Releases and run
the extracted binary.

```bash
VIBE_RELAY_URL=https://relay.example.com \
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
VIBE_DEVICE_NAME=build-node-01 \
vibe-agent
```

Operational notes:

- The default Linux path is `/usr/local/bin/vibe-agent`
- The default Windows path is `C:\Program Files\Vibe Everywhere\vibe-agent.exe`
- On Windows, keep the side-by-side runtime files from the archive; do not copy only `vibe-agent.exe`
- `VIBE_RELAY_URL` must be reachable from the agent host
- After first registration, the agent writes `.vibe-agent/identity.json` in its working directory
- Later restarts reuse the issued device credential instead of reusing the human control token

### 4. Connect a Control Client

Recommended first-use sequence:

1. Open the desktop or Android client.
2. Enter the relay address.
3. Enter `VIBE_RELAY_ACCESS_TOKEN`.
4. Confirm that at least one device is online and that at least one provider is available.
5. Create a long-lived AI conversation.

## Configuration Semantics

### Relay Bind Address vs Public Address

`bind` settings and `public origin` settings serve different purposes.

| Setting | Purpose | Default | Notes |
| --- | --- | --- | --- |
| `VIBE_RELAY_HOST` | Local relay bind address | `0.0.0.0` | Selects the interface address used by the relay process |
| `VIBE_RELAY_PORT` | Local relay listener port | `8787` | Selects the TCP port used by the relay process |
| `VIBE_PUBLIC_RELAY_BASE_URL` | Client-facing relay origin | No production default | Used for client connection information, previews, and generated public links |
| `VIBE_RELAY_FORWARD_HOST` | Preview and forwarding public host | Derived from `VIBE_PUBLIC_RELAY_BASE_URL` when possible | Used for preview URLs exposed to clients |

Key rules:

- `VIBE_PUBLIC_RELAY_BASE_URL` does not change the actual relay listener port.
- If the relay listens on `8787` and clients connect directly to that port, `VIBE_PUBLIC_RELAY_BASE_URL` must include `:8787`.
- `0.0.0.0` is valid as a bind host but not as a client-facing URL host.
- `127.0.0.1` and `localhost` are valid only for same-machine local development.

## Authentication Model

The recommended deployment separates human access from machine enrollment.

| Setting or File | Purpose | Used By |
| --- | --- | --- |
| `VIBE_RELAY_ACCESS_TOKEN` | Control-plane authentication | Desktop, Android, self-hosted Web clients |
| `VIBE_RELAY_ENROLLMENT_TOKEN` | Initial device registration | `vibe-agent` |
| `.vibe-agent/identity.json` | Persisted issued device credential | `vibe-agent` restarts |

Behavior notes:

- After successful registration, the agent uses the issued device credential for heartbeats, task
  claiming, workspace requests, and preview bridging.
- Deleting `.vibe-agent/identity.json` forces the next start to perform registration again.
- If `VIBE_RELAY_ENROLLMENT_TOKEN` is omitted, the relay can still accept the control-plane token
  as a compatibility registration path, but that mode is not recommended for normal deployments.

## Agent, Overlay, and EasyTier Ports

### Default Mode

In the default relay-polling mode, the agent does not expose one fixed public control-plane port.
Its normal workflow is driven primarily by outbound requests to the relay.

### Overlay Mode

When `VIBE_EASYTIER_NETWORK_NAME` is set and EasyTier overlay is enabled, the agent starts the
following bridge listeners:

| Port | Function | Override Variable |
| --- | --- | --- |
| `19090` | shell bridge | `VIBE_AGENT_SHELL_BRIDGE_PORT` |
| `19091` | port-forward bridge | `VIBE_AGENT_PORT_FORWARD_BRIDGE_PORT` |
| `19092` | task bridge | `VIBE_AGENT_TASK_BRIDGE_PORT` |

These ports are part of the internal relay-to-agent overlay path. They are not intended to be used
as public browser or mobile entry points.

### EasyTier Listener Defaults

| Side | Condition | Default Behavior |
| --- | --- | --- |
| relay | Embedded EasyTier enabled and `VIBE_EASYTIER_LISTENERS` unset | Listens on TCP/UDP `11010` |
| agent | Embedded EasyTier enabled | Defaults to `VIBE_EASYTIER_NO_LISTENER=true`; no inbound EasyTier peer listener |
| agent | `VIBE_EASYTIER_NO_LISTENER=false` and `VIBE_EASYTIER_LISTENERS` unset | Listens on TCP/UDP `11010` |

## Standard Usage Flow

Recommended operating sequence:

1. Configure the relay address and control-plane token.
2. Confirm that the target device is online.
3. Check provider availability on the target device.
4. Create or continue a long-lived AI conversation.
5. Review the transcript, tool-choice prompts, and execution results.
6. Use workspace browsing, Git inspection, and previews to validate output.
7. Use shell or advanced connection capabilities only when manual intervention is required.

## Troubleshooting

| Condition | First Checks |
| --- | --- |
| Agent is running but no device appears | Verify `VIBE_RELAY_URL`, `VIBE_RELAY_ENROLLMENT_TOKEN`, relay `/api/health`, and network reachability |
| Device is online but no provider is available | Verify that the provider CLI is installed and visible in the agent process `PATH` |
| Device must be enrolled again | Delete `.vibe-agent/identity.json` and restart the agent |
| Preview links are unreachable | Verify `VIBE_PUBLIC_RELAY_BASE_URL`, `VIBE_RELAY_FORWARD_HOST`, and client reachability to that address |

## System Layout

```text
┌──────────────────────────────────────────────────────────┐
│                     Control App                          │
│      Vue 3.5 Web UI / Tauri Desktop + Android Shell    │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP / SSE / WebSocket
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-relay                          │
│  device registry · AI conversations · workspace · preview│
│        auth · config · transport selection               │
└───────────────────────────┬──────────────────────────────┘
                            │ polling / stream / tunnel
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-agent                          │
│ provider adapters · workspace/git runtime · shell       │
│      preview / forward runtime · overlay support         │
└───────────────────────────┬──────────────────────────────┘
                            │ local process / local TCP
                    ┌───────▼────────┐
                    │ target machine │
                    └────────────────┘
```

## Related Documents

- self-hosted deployment guide: [docs/self-hosted.md](./docs/self-hosted.md)
- relay startup guide (Chinese): [docs/relay-startup.zh-CN.md](./docs/relay-startup.zh-CN.md)
- relay startup guide (English): [docs/relay-startup.md](./docs/relay-startup.md)
- published downloads: [GitHub Releases](https://github.com/fage-ac-org/vibe-everywhere/releases)
