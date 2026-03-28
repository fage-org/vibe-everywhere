# Self-Hosted Deployment Notes

Last updated: 2026-03-28

This document describes the current self-hosted deployment shape for Vibe Everywhere.

## Product Assumptions

- Self-hosted is the default operating mode.
- Relay URL, access token, tenant, user, storage, and preview host are runtime configuration, not
  product constants.
- Web, Tauri Desktop, and Android must all be able to connect to the same relay without changing
  compiled frontend code.
- Clients must not assume `localhost`, `127.0.0.1`, or any fixed domain is reachable from the
  current device.

## Deployment Modes

The relay exposes deployment metadata through `app-config`.

- `self_hosted`: the default mode for personal, team, and private infrastructure deployments.
- `hosted_compatible`: a compatibility mode for future managed or semi-managed deployment surfaces.

The current codebase still runs the same relay binary in both modes. The difference is product
metadata and client guidance, not a separate backend fork.

## Relay Configuration

Important relay environment variables:

- `VIBE_RELAY_HOST`
- `VIBE_RELAY_PORT`
- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`
- `VIBE_RELAY_DEPLOYMENT_MODE`
- `VIBE_RELAY_DOCUMENTATION_URL`
- `VIBE_RELAY_STORAGE_KIND`
- `VIBE_RELAY_STATE_FILE`
- `VIBE_RELAY_FORWARD_HOST`
- `VIBE_RELAY_FORWARD_BIND_HOST`
- `VIBE_RELAY_FORWARD_PORT_START`
- `VIBE_RELAY_FORWARD_PORT_END`
- `VIBE_DEFAULT_TENANT_ID`
- `VIBE_DEFAULT_USER_ID`
- `VIBE_DEFAULT_USER_ROLE`

Guidance:

- `VIBE_PUBLIC_RELAY_BASE_URL` should point to the actual URL clients can reach. Do not rely on the
  bind address for mobile or cross-machine access.
- `VIBE_RELAY_FORWARD_HOST` must resolve correctly from the client opening Preview URLs. It must not
  be hardcoded in frontend components.
- `VIBE_RELAY_FORWARD_BIND_HOST` controls where the relay binds preview listeners. This can remain a
  local interface even when the public relay origin is external.

## Agent Configuration

Important agent environment variables:

- `VIBE_RELAY_URL`
- `VIBE_RELAY_ACCESS_TOKEN`
- `VIBE_DEVICE_ID`
- `VIBE_DEVICE_NAME`
- `VIBE_WORKING_ROOT`
- `VIBE_TENANT_ID`
- `VIBE_USER_ID`
- `VIBE_EASYTIER_NETWORK_NAME`
- `VIBE_EASYTIER_BOOTSTRAP_URL`
- `VIBE_EASYTIER_NODE_IP`

Guidance:

- Agents now send explicit tenant, user, and role headers to the relay. Do not rely on fallback
  personal-mode identities unless that is the intended deployment.
- `VIBE_TENANT_ID` and `VIBE_USER_ID` should be set by runtime configuration or deployment tooling,
  not compiled into the binary or frontend.
- `VIBE_WORKING_ROOT` remains the bounding root for workspace browse and preview features.

## Client Configuration Precedence

For relay URL and access token, clients must follow this order:

1. explicit user input
2. persisted client setting
3. relay or local `app-config`
4. safe fallback default

This precedence is already implemented in the app runtime and must be preserved in future
iterations.

## Storage Modes

Current storage options:

- `file`: persisted JSON snapshot on disk
- `memory`: in-memory runtime store for tests or ephemeral environments
- `external`: compatibility placeholder that currently reuses file-backed behavior

`external` exists to preserve configuration compatibility while the enterprise storage substrate is
still evolving. Do not treat it as a completed external database integration.

## Governance And Audit

Current governance foundations:

- actor identity is derived from relay default config or explicit request headers
- resources are filtered by tenant at the relay layer
- write operations are bounded by actor role
- audit records are persisted and queryable through `/api/audit/events`

This is an enterprise foundation, not a finished enterprise auth system. There is no full SSO,
directory sync, or signed user session model yet.

## Preview And Mobile Networking

- Preview URLs are derived from runtime relay origin plus relay-provided host and port data.
- Mobile clients should use a LAN or public relay origin, not loopback.
- Frontend code must not concatenate preview URLs from fixed hostnames.
- If Preview works on desktop but fails on mobile, check `VIBE_PUBLIC_RELAY_BASE_URL` and
  `VIBE_RELAY_FORWARD_HOST` first.

## Example Development Layout

Relay:

```bash
VIBE_RELAY_HOST=0.0.0.0 \
VIBE_RELAY_PORT=8787 \
VIBE_PUBLIC_RELAY_BASE_URL=http://192.168.1.20:8787 \
VIBE_RELAY_ACCESS_TOKEN=dev-token \
cargo run -p vibe-relay
```

Agent:

```bash
VIBE_RELAY_URL=http://192.168.1.20:8787 \
VIBE_RELAY_ACCESS_TOKEN=dev-token \
VIBE_TENANT_ID=personal \
VIBE_USER_ID=local-admin \
cargo run -p vibe-agent
```

These values are examples only. They must be replaced by environment-specific settings in real
deployments.
