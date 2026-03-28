# Vibe Everywhere

[![CI](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml)
[![Release](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml/badge.svg)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[中文](./README.md) | [English](./README.en.md)

Vibe Everywhere is a self-hosted remote AI control plane. Run `vibe-relay` and `vibe-agent` on
your own infrastructure, then use the Web UI, desktop shell, or Android client to launch AI
sessions, inspect workspaces, review Git state, open previews, and only drop into terminal or
advanced connection tools when needed.

It is not a traditional remote desktop product. The focus is organizing remote AI development
workflows with one control surface across devices.

## Who It Is For

- people who want AI coding tasks to run on remote machines while keeping one control surface
- teams that prefer self-hosting over a managed service
- operators managing multiple devices, workspaces, and provider CLIs
- teams that want a practical MVP now and a path toward stronger team features later

## What It Can Do Today

- create, stream, and cancel AI sessions
- register devices, track presence, and show provider availability
- browse workspaces, preview text files, and inspect Git state
- expose preview flows plus terminal and advanced connection tools when needed
- connect through Web, Tauri desktop, and Android control clients
- ship bilingual UI support for English and Simplified Chinese, plus light / dark / system themes

## Quick Start

### 1. Deploy the Relay

Start with the self-hosted guide:

- [Self-Hosted Deployment Guide](./docs/self-hosted.md)

The repository currently ships two install scripts:

- Linux with `systemd`

```bash
curl -fsSL https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
sudo RELAY_PUBLIC_BASE_URL=https://relay.example.com \
  RELAY_ACCESS_TOKEN=change-me \
  bash install-relay.sh
```

- Windows with a startup task

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-relay.ps1 `
  -PublicRelayBaseUrl https://relay.example.com `
  -RelayAccessToken change-me
```

### 2. Start an Agent on the Target Machine

Download the CLI package from GitHub Releases, extract it, and start `vibe-agent`:

```bash
VIBE_RELAY_URL=https://relay.example.com \
VIBE_RELAY_ACCESS_TOKEN=change-me \
VIBE_DEVICE_NAME=build-node-01 \
./vibe-agent
```

On Windows, keep the extracted side-by-side runtime files next to `vibe-agent.exe` instead of
copying the executable out by itself.

To execute AI sessions, the target machine still needs at least one provider CLI:

- `codex`
- `claude`
- `opencode`

### 3. Open a Control Client

The same relay can be reached from:

- Web
- Tauri desktop
- Android

Recommended flow:

1. deploy the relay
2. start at least one agent
3. validate from Web or desktop first
4. add Android when mobile access is needed

## Downloads

- [GitHub Releases](https://github.com/fage-ac-org/vibe-everywhere/releases)

The release page provides the current CLI, desktop, and Android artifacts. Download the package
that matches your platform.

## Product Layout

```text
┌──────────────────────────────────────────────────────────┐
│                     Control App                          │
│      Vue 3.5 Web UI / Tauri Desktop + Android Shell    │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP / SSE / WebSocket
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-relay                          │
│  device registry · AI sessions · workspace · preview    │
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

## Operator Docs

- self-hosted deployment and install: [docs/self-hosted.md](./docs/self-hosted.md)
