# Self-Hosted Deployment Guide

Last updated: 2026-03-29

This guide describes the self-hosted deployment flow for `vibe-relay` and `vibe-agent`. It focuses
on binary installation, runtime addressing, authentication, startup boundaries, and operator
checks. The relay install scripts now manage binaries only. Service creation, environment-file
layout, and auto-start configuration are documented separately.

## Deployment Model

A standard self-hosted deployment contains:

- one `vibe-relay` instance reachable by control clients and agents
- one or more `vibe-agent` instances on target execution machines
- one or more control clients such as desktop, Android, or a self-hosted Web client

Recommended order:

1. Install or update the `vibe-relay` binary.
2. Configure relay runtime environment variables.
3. Start the relay by foreground process, `systemd`, or Windows Scheduled Task.
4. Start one or more agents against the relay.
5. Connect control clients with the control-plane token.

## Required Runtime Decisions

Prepare these values before deployment:

- the client-facing relay origin, for example `https://relay.example.com` or `http://203.0.113.10:8787`
- the relay listener address and listener port
- the control-plane token for desktop, Android, Web, and operator API access
- the enrollment token used by agents during first registration
- the host used for preview or forwarded links
- the relay state-file location

## Listener Address vs Client-Facing Origin

These settings are related, but they are not interchangeable:

- `VIBE_RELAY_HOST`
  - local bind address used by the relay process
- `VIBE_RELAY_PORT`
  - local TCP listener port used by the relay process
- `VIBE_PUBLIC_RELAY_BASE_URL`
  - client-facing relay origin exposed to clients and used when the relay builds public links
- `VIBE_RELAY_FORWARD_HOST`
  - public host used for preview and forwarding links when it cannot be derived from the public relay origin

Operational rules:

- the relay listens on `0.0.0.0:8787` by default
- setting `VIBE_PUBLIC_RELAY_BASE_URL=http://203.0.113.10` does not change the listener to port `80`
- if the relay really listens on `8787` and clients connect directly to that port, use `http://203.0.113.10:8787`
- `0.0.0.0` is valid as a bind address but not as a client-facing URL host
- `127.0.0.1` and `localhost` are appropriate only for same-machine local development

Recommended patterns:

- direct public-IP access
  - `VIBE_RELAY_HOST=0.0.0.0`
  - keep the real port in `VIBE_PUBLIC_RELAY_BASE_URL`
- reverse proxy or load balancer
  - bind to `127.0.0.1` or a private interface
  - set `VIBE_PUBLIC_RELAY_BASE_URL` to the external origin presented to users

Examples:

```bash
# Direct public IP access on port 8787
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=http://203.0.113.10:8787
```

```bash
# Reverse proxy terminates TLS and forwards to the local relay
export VIBE_RELAY_HOST=127.0.0.1
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
```

## HTTP vs HTTPS

The relay process itself does not require HTTPS. The scheme choice depends on the trust boundary:

- same-machine local development
  - `http` is acceptable
- controlled private-LAN testing
  - `http` can be acceptable when plaintext transport is understood and accepted
- internet-facing or shared-user deployments
  - use `https`

Do not expose the following as client-facing origins outside local development:

- `http://127.0.0.1:8787`
- `http://localhost:8787`
- `http://0.0.0.0:8787`

## CLI Binary Installation

The repository provides the following binary installers:

- Linux: [`scripts/install-relay.sh`](../scripts/install-relay.sh)
- Windows: [`scripts/install-relay.ps1`](../scripts/install-relay.ps1)

Supported commands:

- `install`
  - download and install selected CLI binaries
- `update`
  - download and replace selected CLI binaries
- `uninstall`
  - remove selected CLI binaries

Default component scope:

- `vibe-relay`
- `vibe-agent`

Optional component scope:

- `relay`
  - manage `vibe-relay` only
- `agent`
  - manage `vibe-agent` only

Current boundaries:

- the scripts do not create `systemd` services
- the scripts do not create Windows Scheduled Tasks
- the scripts do not write relay environment files
- the scripts do not start relay or agent processes
- there is no repository-provided one-click relay installer for macOS at this time

### GitHub Access And Mainland-China Acceleration

Both install scripts default to `https://ghfast.top/` as a GitHub URL prefix. This affects:

- the initial `releases/latest` resolution path
- release archive download URLs
- the startup-guide link printed after installation

Operators can override this behavior:

- Linux direct GitHub access: `--no-gh-proxy`
- Windows direct GitHub access: `-NoGhProxy`
- Linux custom proxy prefix: `--gh-proxy <url>`
- Windows custom proxy prefix: `-GhProxy <url>`

### Linux Install, Update, And Uninstall

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
bash install-relay.sh update --release-tag v0.1.8
bash install-relay.sh uninstall
bash install-relay.sh uninstall --component agent
```

Optional installer inputs:

- `--bin-dir`
- `--component`
- `--release-tag`
- `--archive-url`
- `--archive-path`
- `--gh-proxy`
- `--no-gh-proxy`
- `--repo-owner`
- `--repo-name`

Default installed paths:

- `/usr/local/bin/vibe-relay`
- `/usr/local/bin/vibe-agent`

Path note:

- `/usr/local/bin` follows the common Linux/Unix convention for administrator-installed local executables.

### Windows Install, Update, And Uninstall

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
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command update -ReleaseTag v0.1.8
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command uninstall
powershell -ExecutionPolicy Bypass -File .\install-relay.ps1 -Command uninstall -Component agent
```

Optional installer inputs:

- `-InstallDir`
- `-Component`
- `-ReleaseTag`
- `-ArchiveUrl`
- `-ArchivePath`
- `-GhProxy`
- `-NoGhProxy`
- `-RepoOwner`
- `-RepoName`

Default installed paths:

- `C:\Program Files\Vibe Everywhere\vibe-relay.exe`
- `C:\Program Files\Vibe Everywhere\vibe-agent.exe`
- `C:\Program Files\Vibe Everywhere\Packet.dll`
- `C:\Program Files\Vibe Everywhere\wintun.dll`
- `C:\Program Files\Vibe Everywhere\WinDivert64.sys`
- `C:\Program Files\Vibe Everywhere\WinDivert.dll` when present in the archive

Windows packaging note:

- keep the side-by-side runtime files from the release archive beside both `vibe-relay.exe` and `vibe-agent.exe`
- runtime files are removed during `uninstall` only when no CLI binaries remain in the install directory

## Relay Startup

Relay startup instructions are maintained separately:

- English: [relay-startup.md](./relay-startup.md)
- Chinese: [relay-startup.zh-CN.md](./relay-startup.zh-CN.md)

Supported startup shapes in the startup guide:

- foreground process
- Linux `systemd`
- Windows PowerShell launcher and Scheduled Task

Minimum foreground example:

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

Health-check examples:

```bash
curl http://127.0.0.1:8787/api/health
curl http://203.0.113.10:8787/api/health
curl https://relay.example.com/api/health
```

## Recommended Authentication Split

Use separate relay secrets for humans and device enrollment:

- `VIBE_RELAY_ACCESS_TOKEN`
  - control-plane token for desktop, Android, Web, and operator API access
- `VIBE_RELAY_ENROLLMENT_TOKEN`
  - bootstrap token used by `vibe-agent` during initial registration

After successful registration, the agent stores the issued device credential in
`<working-root>/.vibe-agent/identity.json` and reuses that credential for later heartbeats,
task claiming, shell polling, workspace requests, Git requests, and preview-bridge traffic.

Compatibility note:

- if `VIBE_RELAY_ENROLLMENT_TOKEN` is omitted, the relay can still accept the control-plane token
  for registration as a compatibility path
- the recommended deployment shape is still to keep the control-plane token off agent hosts

## Start An Agent

Once the relay is reachable, start an agent on the target machine:

```bash
VIBE_RELAY_URL=https://relay.example.com \
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
VIBE_DEVICE_NAME=build-node-01 \
vibe-agent
```

Operational notes:

- install at least one provider CLI such as `codex`, `claude`, or `opencode` on the target machine
- if the installer script was used with default settings, the Linux agent path is `/usr/local/bin/vibe-agent`
- if the installer script was used with default settings, the Windows agent path is `C:\Program Files\Vibe Everywhere\vibe-agent.exe`
- `VIBE_RELAY_URL` must point to the relay origin reachable from that machine
- the agent does not require a fixed public listener port in default relay-polling mode

## Overlay And EasyTier Notes

When EasyTier overlay mode is enabled, the agent opens internal bridge listeners. Default ports are:

- `19090` for shell bridge
- `19091` for port-forward bridge
- `19092` for task bridge

These ports are overlay-internal transport endpoints. They are not the normal browser or mobile
client entry point.

## Operator Checklist

- verify `/api/health` before connecting clients
- verify `VIBE_PUBLIC_RELAY_BASE_URL` matches the address users actually reach
- verify the control-plane token stays on user devices only
- verify agent hosts use the enrollment token or a previously issued device credential
- verify reverse-proxy or firewall rules expose the correct client-facing port
