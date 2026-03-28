# Vibe Everywhere Linux Hosted Overlay No-TUN Diagnostic Remediation Plan v8

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v8`.
- The concise lookup view lives in [`v8-summary.md`](./v8-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the GitHub-hosted Linux overlay diagnostic issue
that remained after remediation `v7` exposed the real failure as hosted-runner TUN-permission
denial.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current hosted Linux overlay diagnostic problem has two linked facts:

1. the GitHub-hosted Linux runner cannot create the EasyTier TUN device, which now surfaces
   clearly as `rust tun error Operation not permitted (os error 1)`
2. switching EasyTier into `no_tun` mode keeps the embedded overlay control plane alive, but the
   hosted runner still does not gain a routable host path to the overlay node IP, so the existing
   direct overlay-bridge reachability check and strict hosted-runner overlay-proxy assumptions are
   no longer truthful

Local validation of the no_tun adaptation also showed one still-deferred gap:

- preview/port-forward byte-path validation after no_tun fallback is not stable enough to be a
  hard hosted-runner assertion yet; the target receives bytes and replies, but the relay-side
  websocket can still reset before the reply is observed by the test client

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Linux Hosted Overlay No-TUN Diagnostic Path | implemented locally | v7 |

## Shared Remediation Guardrails

- Do not silently enable EasyTier `no_tun` as a product/runtime default just because one CI
  environment lacks TUN capability.
- Do not keep a hosted-runner diagnostic asserting direct overlay bridge reachability when the same
  hosted runner has no host route to the overlay node in no_tun mode.
- Do not report hosted-runner preview byte-path coverage as verified when the local no_tun
  diagnostic can only prove transport selection and lifecycle today.

## R1: Linux Hosted Overlay No-TUN Diagnostic Path

### Problem

After remediation `v7` preserved the raw EasyTier stop reason, the next hosted `CI` run showed
that the Linux runner fails because embedded EasyTier cannot create its TUN device. A naive
`no_tun` switch was not sufficient on its own because the relay still cannot directly dial the
overlay node IP from the host network namespace, and the hosted-runner preview data path remains
timing-sensitive under no_tun.

### Goal

Keep the non-blocking Linux overlay job meaningful and stable on GitHub-hosted runners by using a
harness-scoped `no_tun` path that:

- verifies the overlay control plane can still come up
- verifies task and shell flows degrade truthfully to relay polling
- verifies preview selection/lifecycle can degrade to relay tunnel
- leaves product defaults untouched
- explicitly records that preview byte-path validation under hosted no_tun is deferred

### Repair Modes

- Mode A: `Harness-scoped no_tun with truthful fallback assertions`
  Expose EasyTier `no_tun` behind harness-only env vars, run the hosted Linux overlay diagnostic in
  that mode, skip host-only overlay bridge reachability checks, and verify only the fallback
  behaviors that the hosted runner can truthfully exercise today.
- Mode B: `Skip hosted Linux overlay diagnostics when TUN is unavailable`
  Avoid the hosted failure entirely, but lose most of the useful overlay control-plane and fallback
  signal.
- Mode C: `Move full overlay diagnostics to a privileged runner`
  Preserve full TUN and overlay-bridge coverage, but requires new CI infrastructure instead of
  repairing the current hosted-runner path.

### Recommended Mode

- Mode A

Reason:

- it keeps the current hosted-runner diagnostic useful instead of turning it into a skip
- it follows EasyTier's upstream `no_tun` capability rather than inventing another custom mode
- it keeps the deferred preview byte-path gap explicitly documented instead of pretending the
  hosted runner proves more than it actually can

### Planned Scope

- add explicit `VIBE_EASYTIER_NO_TUN` handling to relay and agent EasyTier config builders
- add a harness-only `VIBE_TEST_EASYTIER_NO_TUN` switch to the Linux overlay smoke path and enable
  it only from the GitHub-hosted Linux diagnostic job
- skip the direct overlay-bridge listener probe when the harness is intentionally in no_tun mode
- validate task and shell fallback behavior truthfully under hosted no_tun
- validate preview transport selection/lifecycle fallback to relay tunnel under hosted no_tun, but
  skip hosted preview byte-path verification until the relay-tunnel websocket reset is repaired
- update versioned remediation records, `PLAN.md`, and durable guardrails for harness-only
  capability workarounds

### Acceptance Criteria

- `cargo fmt --all --check` succeeds after the EasyTier no_tun and harness changes
- `cargo check --locked -p vibe-relay -p vibe-agent` succeeds after the EasyTier no_tun changes
- `bash -n scripts/dual-process-smoke.sh` succeeds after the smoke-script changes
- `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay` succeeds locally
- `./scripts/dual-process-smoke.sh relay_polling` still succeeds after the shared smoke-script
  changes
- the hosted Linux workflow runs the overlay diagnostic with harness-only `no_tun`, while product
  defaults remain unchanged
- the deferred hosted no_tun preview byte-path gap is recorded explicitly in planning and guardrail
  files

### Validation

- `cargo fmt --all --check`
- `cargo check --locked -p vibe-relay -p vibe-agent`
- `bash -n scripts/dual-process-smoke.sh`
- `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay`
- `./scripts/dual-process-smoke.sh relay_polling`
- monitor the next pushed `CI` workflow, especially `Overlay Diagnostics (Linux, non-blocking)`

### Item Record

- Chosen repair mode:
  `Mode A (user-confirmed recommended mode)`
- Implementation notes:
  exposed EasyTier `no_tun` as an explicit env-driven option in both relay and agent config
  builders, enabled it only from the hosted Linux overlay diagnostic job via
  `VIBE_TEST_EASYTIER_NO_TUN=1`, and adjusted the overlay smoke harness so that no_tun mode no
  longer asserts impossible host-route behavior. The hosted no_tun diagnostic now checks overlay
  control-plane readiness, task/shell fallback to relay polling, and preview transport/lifecycle
  fallback to relay tunnel. Preview byte-path verification under hosted no_tun remains deferred
  because local validation still shows a relay-side websocket reset after the target replies.
- Validation results:
  `2026-03-28`: `cargo fmt --all --check` succeeded after the EasyTier no_tun harness changes.
  `2026-03-28`: `cargo check --locked -p vibe-relay -p vibe-agent` succeeded after the EasyTier
  no_tun config additions.
  `2026-03-28`: `bash -n scripts/dual-process-smoke.sh` succeeded after the no_tun smoke-script
  path changes.
  `2026-03-28`: `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay` succeeded
  after the hosted no_tun overlay diagnostic changes.
  `2026-03-28`: `./scripts/dual-process-smoke.sh relay_polling` succeeded after the shared smoke
  script changes.
  `2026-03-28`: GitHub Actions validation is still required after push; `CI` monitoring remains the
  final close-out step for remediation `v8`.
