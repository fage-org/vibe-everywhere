# Vibe Everywhere Relay And Agent Auth Boundary Hardening Remediation Plan v10

Last updated: 2026-03-29

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v10`.
- The concise lookup view lives in [`v10-summary.md`](./v10-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the auth-boundary gaps that remained after the
Iteration 11 tenant/user/role baseline.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The Iteration 11 baseline introduced tenant/user/role structures, but the runtime auth model still
left several practical gaps:

1. agents still reused the same relay access token as human control clients by default
2. the relay still trusted external `x-vibe-*` actor headers for tenant/user/role identity
3. several device-only routes needed stronger device-credential enforcement, especially around
   workspace, Git, preview, and bridge traffic
4. install scripts, operator docs, and smoke harnesses still described or exercised the older
   single-token shape instead of the safer split-token model

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Relay And Agent Auth Boundary Hardening | completed locally | Iteration 11 baseline |

## Shared Remediation Guardrails

- Do not trust client-supplied `x-vibe-*` tenant, user, or role headers for relay identity.
- Do not keep the human control-plane token on agent hosts when a dedicated enrollment token plus
  issued device credential flow is available.
- Do not weaken device-only route auth or overlay-bridge auth just to make tests easier.

## R1: Relay And Agent Auth Boundary Hardening

### Problem

The old auth path mixed human control-plane access and device registration/device polling into the
same secret boundary, while also leaving room for actor spoofing through external headers. That
model was too soft for self-hosted deployments that want one shared control token for people today
without copying that same credential onto every device.

### Goal

Harden the current self-hosted auth model without jumping straight into a full user/session system:

- keep the human control plane on one relay control token for now
- introduce a dedicated enrollment token for first-time agent registration
- issue and reuse device credentials for later device traffic
- stop trusting external actor headers
- make installers, docs, and smoke coverage match the delivered model

### Repair Modes

- Mode A: `Split agent enrollment from control access and issue device credentials`
  Keep a single human control token for now, add a dedicated enrollment token, issue device
  credentials for post-registration traffic, remove external actor-header trust, and tighten route
  auth boundaries.
- Mode B: `Keep one shared token and only remove actor-header trust`
  Reduce spoofing risk, but keep the control-plane secret on every agent host and preserve a wider
  blast radius.
- Mode C: `Jump directly to per-user API keys or OIDC`
  Move straight to a larger auth program, but add significantly more operator and product
  complexity than this repair needs right now.

### Recommended Mode

- Mode A

Reason:

- it removes the biggest practical exposure without forcing full enterprise auth in the same turn
- it keeps operator configuration simple enough for the current self-hosted product
- it closes the actor-header spoofing path and the device-route auth gaps together instead of
  leaving half the problem in place

### Planned Scope

- add relay `VIBE_RELAY_ENROLLMENT_TOKEN` support and installer inputs for the same setting
- issue a device credential during registration and persist agent identity under
  `<working-root>/.vibe-agent/identity.json`
- use issued device credentials for heartbeats, polling, workspace/Git requests, preview bridge
  traffic, and overlay bridge handoff
- remove trust in external `x-vibe-*` actor headers
- tighten control-only and device-only route auth boundaries, including workspace, Git,
  event-stream, task, shell, and preview-related flows
- update README, self-hosted docs, testing guidance, release notes, active plan files, and smoke
  harnesses to match the split-token model

### Acceptance Criteria

- relay control routes use the configured control-plane token when present
- agent registration succeeds with the enrollment token and returns an issued device credential
- post-registration device traffic uses issued device credentials rather than the human
  control-plane token
- external `x-vibe-*` headers are no longer accepted as an auth mechanism
- workspace, Git, shell, preview, and task device routes reject missing or mismatched device
  credentials
- operator docs, installers, and smoke tests all describe and exercise the split-token flow

### Validation

- `cargo fmt --all --check`
- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm run build`
- `bash -n scripts/install-relay.sh`
- `bash -n scripts/dual-process-smoke.sh`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/dual-process-smoke.sh overlay`
- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`
- local PowerShell parser validation for `scripts/install-relay.ps1` and
  `scripts/dual-process-smoke.ps1` when `pwsh` is available

### Item Record

- Chosen repair mode:
  `Mode A (user-specified)`
- Implementation notes:
  split agent enrollment from the human control-plane token, issued and persisted device
  credentials, removed trust in external `x-vibe-*` actor headers, tightened device-route auth on
  workspace/Git/task/shell/preview flows, updated overlay bridge token usage, and aligned
  installers, operator docs, release notes, and smoke harnesses with the delivered model.
- Validation results:
  `2026-03-29`: `cargo fmt --all --check` succeeded after the auth-boundary hardening changes.
  `2026-03-29`: `cargo check -p vibe-relay -p vibe-agent -p vibe-app` succeeded after the
  auth-boundary hardening changes.
  `2026-03-29`: `cargo test --workspace --all-targets -- --nocapture` succeeded after the
  auth-boundary hardening changes.
  `2026-03-29`: `cd apps/vibe-app && npm run build` succeeded after the auth-boundary hardening
  changes.
  `2026-03-29`: `bash -n scripts/install-relay.sh` succeeded after adding enrollment-token
  installer support.
  `2026-03-29`: `bash -n scripts/dual-process-smoke.sh` succeeded after the split-token smoke
  harness updates.
  `2026-03-29`: `./scripts/dual-process-smoke.sh relay_polling` succeeded with relay control auth,
  agent enrollment, issued device credentials, task execution, and relay-tunnel preview
  validation.
  `2026-03-29`: `./scripts/dual-process-smoke.sh overlay` succeeded with relay control auth,
  agent enrollment, issued device credentials, overlay task/shell traffic, and overlay preview
  validation.
  `2026-03-29`: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after the release
  note updates.
  `2026-03-29`: local PowerShell parser validation for `scripts/install-relay.ps1` and
  `scripts/dual-process-smoke.ps1` could not be run because `pwsh` was not installed in the local
  environment.
