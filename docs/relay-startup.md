# Relay Startup Guide

Last updated: 2026-03-29

This guide describes how to start `vibe-relay` after the CLI binaries have been installed. The
install scripts in `scripts/install-relay.sh` and `scripts/install-relay.ps1` manage only binary
installation, update, or removal. They do not create services, write environment files, or start
the relay process.

## Startup Methods

You can start `vibe-relay` in any of the following ways:

- foreground process for local testing or manual operation
- `systemd` service on Linux
- Windows PowerShell launcher and Scheduled Task

## Minimum Environment

The minimum practical relay configuration for a shared deployment is:

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

If the relay is intended for same-machine local development only, `VIBE_PUBLIC_RELAY_BASE_URL`
may be omitted or set to a loopback URL. For remote desktop, Android, or multi-machine access, use
an address that clients can actually reach.

## Linux Startup

### Foreground Start

```bash
export VIBE_RELAY_HOST=0.0.0.0
export VIBE_RELAY_PORT=8787
export VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
export VIBE_RELAY_ACCESS_TOKEN=change-control-token
export VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
vibe-relay
```

### `systemd` Example

Example environment file:

```ini
# /etc/vibe-relay/relay.env
VIBE_RELAY_HOST=0.0.0.0
VIBE_RELAY_PORT=8787
VIBE_PUBLIC_RELAY_BASE_URL=https://relay.example.com
VIBE_RELAY_ACCESS_TOKEN=change-control-token
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token
VIBE_RELAY_STATE_FILE=/var/lib/vibe-relay/relay-state.json
```

Example service file:

```ini
# /etc/systemd/system/vibe-relay.service
[Unit]
Description=Vibe Everywhere Relay
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=/etc/vibe-relay/relay.env
WorkingDirectory=/var/lib/vibe-relay
ExecStart=/usr/local/bin/vibe-relay
Restart=always
RestartSec=2
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

Activation:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now vibe-relay
sudo systemctl status vibe-relay
```

## Windows Startup

### Foreground Start

```powershell
$env:VIBE_RELAY_HOST = "0.0.0.0"
$env:VIBE_RELAY_PORT = "8787"
$env:VIBE_PUBLIC_RELAY_BASE_URL = "https://relay.example.com"
$env:VIBE_RELAY_ACCESS_TOKEN = "change-control-token"
$env:VIBE_RELAY_ENROLLMENT_TOKEN = "change-agent-enrollment-token"
& "C:\Program Files\Vibe Everywhere\vibe-relay.exe"
```

### PowerShell Launcher Example

```powershell
# C:\ProgramData\Vibe Everywhere\relay-env.ps1
$env:VIBE_RELAY_HOST = "0.0.0.0"
$env:VIBE_RELAY_PORT = "8787"
$env:VIBE_PUBLIC_RELAY_BASE_URL = "https://relay.example.com"
$env:VIBE_RELAY_ACCESS_TOKEN = "change-control-token"
$env:VIBE_RELAY_ENROLLMENT_TOKEN = "change-agent-enrollment-token"
$env:VIBE_RELAY_STATE_FILE = "$env:ProgramData\Vibe Everywhere\state\relay-state.json"
```

```powershell
# C:\ProgramData\Vibe Everywhere\Start-VibeRelay.ps1
. "C:\ProgramData\Vibe Everywhere\relay-env.ps1"
& "C:\Program Files\Vibe Everywhere\vibe-relay.exe"
```

### Scheduled Task Example

```powershell
$action = New-ScheduledTaskAction `
  -Execute "powershell.exe" `
  -Argument '-NoProfile -ExecutionPolicy Bypass -File "C:\ProgramData\Vibe Everywhere\Start-VibeRelay.ps1"'
$trigger = New-ScheduledTaskTrigger -AtStartup
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

Register-ScheduledTask -TaskName VibeRelay -Action $action -Trigger $trigger -Principal $principal -Force
Start-ScheduledTask -TaskName VibeRelay
```

## Core Environment Variables

| Variable | Default | Required | Description |
| --- | --- | --- | --- |
| `VIBE_RELAY_HOST` | `0.0.0.0` | No | Local bind address for the relay HTTP listener |
| `VIBE_RELAY_PORT` | `8787` | No | Local bind port for the relay HTTP listener |
| `VIBE_PUBLIC_RELAY_BASE_URL` | none in production | Recommended | Client-facing relay origin used in app config and generated public links |
| `VIBE_RELAY_ACCESS_TOKEN` | none | Recommended | Control-plane token used by desktop, Android, Web clients, and operator API calls |
| `VIBE_RELAY_ENROLLMENT_TOKEN` | none | Recommended | Bootstrap token used by `vibe-agent` during first registration |
| `VIBE_RELAY_STATE_FILE` | platform default | No | Relay state file path |
| `VIBE_RELAY_DEPLOYMENT_MODE` | `self_hosted` | No | Deployment mode metadata exposed to clients |
| `VIBE_RELAY_FORWARD_HOST` | derived from `VIBE_PUBLIC_RELAY_BASE_URL` when possible | No | Public host used for preview and forwarded-link generation |
| `VIBE_RELAY_FORWARD_BIND_HOST` | same as `VIBE_RELAY_HOST` | No | Local bind host for relay-managed preview and forwarding listeners |
| `VIBE_RELAY_FORWARD_PORT_START` | `39000` | No | Start of the relay-managed forwarding port range |
| `VIBE_RELAY_FORWARD_PORT_END` | `39999` | No | End of the relay-managed forwarding port range |

## Identity And Default Actor Variables

These values are usually left at defaults in single-tenant self-hosted deployments:

| Variable | Default | Description |
| --- | --- | --- |
| `VIBE_DEFAULT_TENANT_ID` | repository default tenant ID | Default tenant identifier used by relay-owned records |
| `VIBE_DEFAULT_USER_ID` | repository default user ID | Default user identifier used by relay-owned records |
| `VIBE_DEFAULT_USER_ROLE` | `owner` | Default role exposed for the relay default actor |

## Overlay And EasyTier Variables

Use these only when you are enabling embedded EasyTier overlay support:

| Variable | Default | Description |
| --- | --- | --- |
| `VIBE_EASYTIER_RELAY_ENABLED` | `false` unless network name is set | Enables embedded EasyTier on the relay |
| `VIBE_EASYTIER_NETWORK_NAME` | none | Overlay network name |
| `VIBE_EASYTIER_NETWORK_SECRET` | none | Overlay network secret |
| `VIBE_EASYTIER_BOOTSTRAP_URL` | none | Comma-separated EasyTier peer URLs |
| `VIBE_EASYTIER_LISTENERS` | TCP/UDP `11010` on relay if unset | Listener definitions for embedded EasyTier |
| `VIBE_EASYTIER_PRIVATE_MODE` | derived from network-name usage | EasyTier private mode |
| `VIBE_EASYTIER_NO_TUN` | `false` | Disables TUN usage in special environments |
| `VIBE_EASYTIER_INSTANCE_NAME` | relay default | EasyTier instance name |
| `VIBE_EASYTIER_HOSTNAME` | relay default | EasyTier node hostname |
| `VIBE_AGENT_SHELL_BRIDGE_PORT` | `19090` | Shell bridge port expected on the agent when overlay mode is active |
| `VIBE_AGENT_PORT_FORWARD_BRIDGE_PORT` | `19091` | Port-forward bridge port expected on the agent when overlay mode is active |
| `VIBE_AGENT_TASK_BRIDGE_PORT` | `19092` | Task bridge port expected on the agent when overlay mode is active |

## Addressing Rules

- `VIBE_RELAY_HOST` and `VIBE_RELAY_PORT` control where the relay listens locally.
- `VIBE_PUBLIC_RELAY_BASE_URL` controls which address clients should use.
- `0.0.0.0` is valid as a bind address but not as a client-facing URL host.
- `127.0.0.1` and `localhost` are valid only for same-machine local development.
- If the relay listens on `8787` and clients connect directly to that port, include `:8787` in
  `VIBE_PUBLIC_RELAY_BASE_URL`.

## Health Check

Typical health-check commands:

```bash
curl http://127.0.0.1:8787/api/health
curl http://203.0.113.10:8787/api/health
curl https://relay.example.com/api/health
```

## Related Documents

- [README.md](../README.md)
- [README.en.md](../README.en.md)
- [Self-Hosted Deployment Guide](./self-hosted.md)
