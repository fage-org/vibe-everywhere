# Self-Hosted Deployment Guide

Last updated: 2026-03-29

This guide is for operators who want to deploy the relay, keep it running after reboot, and let
Web, desktop, or Android clients connect to the same control plane.

## Before You Start

Decide these runtime values first:

- the relay address your users and agents will actually access
- the control-plane token used by Web, desktop, and Android clients
- the enrollment token used by agents during first registration
- which host should be used for preview links
- where relay state should be stored on disk

If the control plane will be accessed from phones or other machines, use a reachable domain or IP
address for the relay origin.

## Supported Install Automation

The repository currently ships automation for:

- Linux: [`scripts/install-relay.sh`](../scripts/install-relay.sh)
  - downloads the published Linux CLI archive unless you point it to a local archive
  - installs `vibe-relay`
  - writes `/etc/vibe-relay/relay.env`
  - creates and optionally starts a `systemd` service
- Windows: [`scripts/install-relay.ps1`](../scripts/install-relay.ps1)
  - downloads the published Windows CLI archive unless you point it to a local archive
  - installs `vibe-relay.exe`
  - writes `relay-env.ps1` plus a launcher script
  - creates and optionally starts a Windows Scheduled Task for auto-start

Current boundary:

- there is no repository-provided one-click relay installer for macOS at this time

## Recommended Auth Split

Use two different relay secrets in self-hosted deployments:

- `VIBE_RELAY_ACCESS_TOKEN`
  - the human control-plane token for Web, desktop, Android, and operator API use
- `VIBE_RELAY_ENROLLMENT_TOKEN`
  - the bootstrap token used by `vibe-agent` during initial device registration

After a successful registration, the agent stores its issued device credential in
`<working-root>/.vibe-agent/identity.json` and uses that device identity for heartbeats, task
claiming, shell polling, workspace/Git requests, and preview bridge traffic on later restarts.

Compatibility note:

- if you omit `VIBE_RELAY_ENROLLMENT_TOKEN`, the relay still accepts the control-plane token for
  agent registration as an admin/compatibility path
- the recommended deployment shape is still to keep the control-plane token off the device and use
  a dedicated enrollment token instead
- relay identity is derived from configured tokens and issued device credentials; external
  `x-vibe-*` actor headers are not a supported auth mechanism

## Linux Quick Install

```bash
curl -fsSL https://raw.githubusercontent.com/fage-ac-org/vibe-everywhere/main/scripts/install-relay.sh -o install-relay.sh
sudo RELAY_PUBLIC_BASE_URL=https://relay.example.com \
  RELAY_ACCESS_TOKEN=change-control-token \
  RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
  bash install-relay.sh
```

Optional inputs:

- `VIBE_RELEASE_TAG`
- `RELAY_CLI_ARCHIVE_URL`
- `RELAY_CLI_ARCHIVE_PATH`
- `RELAY_BIND_HOST`
- `RELAY_PORT`
- `RELAY_FORWARD_HOST`
- `RELAY_ACCESS_TOKEN`
- `RELAY_ENROLLMENT_TOKEN`
- `CREATE_SYSTEMD_SERVICE`
- `ENABLE_AND_START_SERVICE`

Installed paths:

- binary: `/usr/local/bin/vibe-relay`
- env file: `/etc/vibe-relay/relay.env`
- state file default: `/var/lib/vibe-relay/relay-state.json`
- service file: `/etc/systemd/system/vibe-relay.service`

Useful commands:

```bash
sudo systemctl status vibe-relay
sudo journalctl -u vibe-relay -f
sudo systemctl restart vibe-relay
```

## Windows Quick Install

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-relay.ps1 `
  -PublicRelayBaseUrl https://relay.example.com `
  -RelayAccessToken change-control-token `
  -RelayEnrollmentToken change-agent-enrollment-token
```

Optional inputs:

- `-ReleaseTag`
- `-ArchiveUrl`
- `-ArchivePath`
- `-RelayBindHost`
- `-RelayPort`
- `-RelayForwardHost`
- `-RelayAccessToken`
- `-RelayEnrollmentToken`
- `-SkipStartupTask`

Installed paths:

- binary: `C:\Program Files\Vibe Everywhere\vibe-relay.exe`
- side-by-side runtime files: `Packet.dll`, `wintun.dll`, `WinDivert64.sys` and `WinDivert.dll`
  when present
- env script: `C:\Program Files\Vibe Everywhere\relay-env.ps1`
- launcher: `C:\Program Files\Vibe Everywhere\Start-VibeRelay.ps1`
- state file default: `%ProgramData%\Vibe Everywhere\state\relay-state.json`

Useful commands:

```powershell
Get-ScheduledTask -TaskName VibeRelay
Start-ScheduledTask -TaskName VibeRelay
Stop-Process -Name vibe-relay
```

## Start an Agent

Once the relay is reachable, start an agent on the target machine:

```bash
VIBE_RELAY_URL=https://relay.example.com \
VIBE_RELAY_ENROLLMENT_TOKEN=change-agent-enrollment-token \
VIBE_DEVICE_NAME=build-node-01 \
./vibe-agent
```

Notes:

- install at least one provider CLI such as `codex`, `claude`, or `opencode` if you want AI
  session execution
- on Windows, keep the extracted CLI runtime files beside `vibe-agent.exe`; do not copy only the
  executable out of the archive
- `VIBE_RELAY_URL` should point to the relay origin the agent can actually reach
- the first successful registration writes `<working-root>/.vibe-agent/identity.json`; later
  restarts reuse that issued device credential instead of needing the control-plane token on the
  agent host
- deleting the identity file forces a fresh enrollment on the next agent start
- `VIBE_PUBLIC_RELAY_BASE_URL` and `VIBE_RELAY_FORWARD_HOST` affect client-facing links generated
  by the relay

## Related Documents

- [README.md](../README.md)
- [README.en.md](../README.en.md)
- [DEVELOPMENT.md](../DEVELOPMENT.md)
- [TESTING.md](../TESTING.md)
