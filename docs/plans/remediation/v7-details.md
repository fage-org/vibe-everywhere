# Vibe Everywhere Linux Overlay Diagnostic Harness Stabilization Remediation Plan v7

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v7`.
- The concise lookup view lives in [`v7-summary.md`](./v7-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the GitHub-hosted Linux overlay diagnostic
instability discovered while validating the post-Windows-fix `CI` run.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current Linux overlay diagnostic problem has two coupled causes:

1. the smoke harness allocates multiple ports by probing anonymous TCP sockets independently, which
   does not guarantee that relay, agent, and target helpers will receive unique ports or that the
   chosen port is free for both TCP and UDP when EasyTier later binds it
2. when the embedded EasyTier agent runtime stops unexpectedly, the agent currently overwrites the
   upstream error string with a generic `embedded EasyTier agent instance stopped` wrapper, leaving
   hosted-runner failures opaque

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Linux Overlay Diagnostic Harness Stabilization | completed | none |

## Shared Remediation Guardrails

- Do not treat repeated `bind(..., 0)` probes as a reservation mechanism when one harness starts
  multiple cooperating processes on the same runner.
- Do not allocate EasyTier listener ports with TCP-only availability checks when the runtime later
  needs both TCP and UDP on the same port.
- Do not overwrite an embedded runtime's raw stop reason with a generic wrapper if that string is
  the only useful diagnostic signal available in CI artifacts.

## R1: Linux Overlay Diagnostic Harness Stabilization

### Problem

The latest hosted `CI` run showed `Overlay Diagnostics (Linux, non-blocking)` failing before task,
shell, or port-forward validation began. The smoke artifacts reported repeated embedded EasyTier
agent restarts, `overlayState = unavailable`, and `lastError = embedded EasyTier agent instance
stopped`, which is too generic to distinguish listener conflicts from other runtime failures.

### Goal

Make the overlay smoke harness more deterministic on same-runner Linux CI and preserve enough raw
error detail to diagnose any future embedded EasyTier stop cleanly.

### Repair Modes

- Mode A: `Reserve unique multi-protocol harness ports and preserve raw EasyTier errors`
  Fix the likely same-runner listener collision path and keep the next failure, if any, directly
  diagnosable.
- Mode B: `Preserve raw EasyTier errors only`
  Improve diagnosis, but keep the current port-allocation race in place.
- Mode C: `Force fixed CI-only overlay ports`
  Avoid the random-allocation race by pinning ports, but increase coupling to runner topology and
  create another fixed-port harness assumption to maintain.

### Recommended Mode

- Mode A

Reason:

- it addresses the most plausible hosted-runner failure mode instead of only improving logs
- it keeps the harness flexible instead of introducing new fixed-port assumptions
- it ensures the next CI result is informative even if a second issue remains

### Planned Scope

- replace the smoke harness's one-off TCP port probes with a harness-local allocator that tracks
  already assigned ports and validates TCP+UDP availability before use
- use that allocator for the relay HTTP listener, relay EasyTier listener, agent EasyTier listener,
  and smoke target server ports
- preserve the upstream embedded EasyTier agent error string when the runtime stops unexpectedly
- update remediation records, `PLAN.md`, and durable guardrails to capture the hosted-runner
  lesson

### Acceptance Criteria

- `./scripts/dual-process-smoke.sh overlay` succeeds locally after the harness-port change
- `./scripts/dual-process-smoke.sh relay_polling` still succeeds after the shared script change
- the agent overlay status keeps the raw EasyTier stop reason instead of replacing it with a
  generic wrapper message
- the repository planning and guardrail files record the harness-specific port-allocation rule

### Validation

- `bash -n scripts/dual-process-smoke.sh`
- `cargo fmt --all --check`
- `cargo check --locked -p vibe-relay -p vibe-agent`
- `./scripts/dual-process-smoke.sh overlay`
- `./scripts/dual-process-smoke.sh relay_polling`
- monitor the next pushed `CI` workflow, especially `Overlay Diagnostics (Linux, non-blocking)`

### Item Record

- Chosen repair mode:
  `Mode A (user-confirmed recommended mode)`
- Implementation notes:
  replaced the smoke harness's anonymous-port probing with a harness-local reservation helper that
  excludes already selected ports and verifies same-port TCP+UDP availability before assigning it
  to relay, agent, or target helper processes. Also changed the agent EasyTier monitor path to
  keep the original `get_latest_error_msg()` string when the embedded runtime stops, instead of
  overwriting it with a generic wrapper.
- Validation results:
  `2026-03-28`: `bash -n scripts/dual-process-smoke.sh` succeeded after the harness-port
  allocation change.
  `2026-03-28`: `cargo fmt --all --check` succeeded after the harness-port and EasyTier-error
  propagation changes.
  `2026-03-28`: `cargo check --locked -p vibe-relay -p vibe-agent` succeeded after the harness-port
  and EasyTier-error propagation changes.
  `2026-03-28`: `./scripts/dual-process-smoke.sh overlay` succeeded after the harness-port
  reservation change.
  `2026-03-28`: `./scripts/dual-process-smoke.sh relay_polling` succeeded after the shared
  script-path change.
  `2026-03-28`: GitHub Actions `CI` run `23687951251` preserved the raw EasyTier stop reason and
  exposed the hosted-runner Linux root cause as `rust tun error Operation not permitted (os error 1)`;
  this confirmed remediation `v7`'s diagnosis goal and moved the new TUN-capability issue into
  remediation `v8`.
