# Vibe Everywhere Overlay Semantics, Documentation Boundary, And Workflow Throughput Remediation Plan v5

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v5`.
- The concise lookup view lives in [`v5-summary.md`](./v5-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the residual runtime-truthfulness, documentation,
and workflow-latency problems discovered after remediation plan `v4` was marked complete.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current problems fall into three related buckets:

1. the agent can still report overlay `connected` while embedded EasyTier RPC is not ready, which
   misleads the relay, the UI, and the smoke harness
2. the top-level README files still expose developer/test-oriented navigation in places where the
   user asked for a stricter user/operator surface boundary
3. Android workflows still spend avoidable time redownloading SDK components that can be cached
   with explicit version-based invalidation

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Overlay Connectivity Truthfulness And Smoke Alignment | completed | none |
| R2 | README Operator Surface Tightening | completed | none |
| R3 | Android Workflow Throughput Optimization | completed | R2 |

## Shared Remediation Guardrails

- Do not represent overlay bootstrap progress as confirmed overlay connectivity.
- Do not repair smoke-only symptoms while leaving misleading product state semantics in place.
- Do not turn the top-level README files into navigation hubs for contributor, testing, planning, or
  governance documents.
- Do not add Android workflow caches unless their invalidation inputs are explicit and bounded.

## R1: Overlay Connectivity Truthfulness And Smoke Alignment

### Problem

The embedded EasyTier agent currently reports `overlay.state = connected` too early when bootstrap
peers are configured but the runtime RPC is still not ready. This leaks into relay transport
decisions, device/UI state, and smoke readiness checks.

### Goal

Make overlay state truthful and align the smoke harness with that truthful signal.

### Repair Modes

- Mode A: `Tighten smoke polling only`
  Makes CI less optimistic, but leaves misleading product state in the relay and UI.
- Mode B: `Fix agent overlay-state semantics and keep smoke bridge gating`
  Use product-correct state semantics first, then let smoke wait on that state plus bridge
  reachability.
- Mode C: `Disable overlay smoke in required CI`
  Reduces signal and hides the defect instead of fixing it.

### Recommended Mode

- Mode B

Reason:

- it matches the prior user direction to prefer product/runtime fixes over smoke-only compromises
- it improves relay and UI truthfulness, not just CI behavior
- it keeps overlay smoke meaningful once the state signal is corrected

### Planned Scope

- change agent overlay-state reporting so bootstrap configuration alone never yields `connected`
- report `degraded` while EasyTier RPC is not ready
- keep post-RPC `connected` semantics aligned with the existing configured-bootstrap model
- retain smoke bridge-listener reachability checks after the truthful `connected` state appears
- add or update focused agent tests for the overlay-state decision

### Acceptance Criteria

- agent heartbeats no longer report `connected` while EasyTier RPC is still not ready
- relay overlay path selection no longer sees premature `connected` state from the agent
- overlay smoke waits until truthful connectivity is present before bridge probes begin
- local overlay smoke still passes after the change

### Validation

- `cargo test -p vibe-agent -- --nocapture`
- `bash -n scripts/dual-process-smoke.sh`
- `./scripts/dual-process-smoke.sh overlay`

### Item Record

- Chosen repair mode:
  `Mode B (consistent with prior user-specified runtime-first direction)`
- Implementation notes:
  changed the embedded EasyTier agent so `connected` is no longer reported while the runtime RPC is
  still unavailable, kept post-RPC overlay state aligned with the existing configured-bootstrap
  semantics, added agent-side EasyTier lifecycle logging for CI diagnosis, and preserved the smoke
  harness bridge reachability gate after truthful overlay readiness with a dedicated harness-only
  agent listener plus faster harness-only EasyTier restart/poll cadence for same-host CI stability.
- Validation results:
  `2026-03-28`: `cargo test -p vibe-agent -- --nocapture` succeeded after the overlay-state
  reporting change and the new EasyTier state test.
  `2026-03-28`: `./scripts/dual-process-smoke.sh overlay` succeeded after the overlay-state fix
  and bridge reachability gate remained intact.

## R2: README Operator Surface Tightening

### Problem

Even after the README rewrite in `v4`, the top-level README files still expose developer/test
navigation that the user explicitly asked to keep out of the main user/operator entry surface.

### Goal

Keep top-level README files strictly user/operator facing and make the boundary explicit enough to
avoid future drift.

### Repair Modes

- Mode A: `Remove only the DEVELOPMENT link`
  Fixes the most obvious issue, but leaves the boundary ambiguous.
- Mode B: `Remove developer/test/governance links and codify the exclusion rule`
  Keeps README ownership clear and reviewable.
- Mode C: `Move all documentation links out of README`
  Over-corrects and makes user onboarding thinner than necessary.

### Recommended Mode

- Mode B

Reason:

- it matches the user's requirement that developer entry not live in README
- it avoids future drift by turning the boundary into a reviewable rule
- it still allows operator-facing docs to remain discoverable

### Planned Scope

- remove developer/test-oriented documentation entry points from `README.md` and `README.en.md`
- keep only user/operator-facing onboarding links on the top-level README surfaces
- tighten guardrails in `AGENTS.md`, `docs/plans/process.md`, and `TESTING.md` so this rule is
  explicit rather than implied

### Acceptance Criteria

- top-level README files no longer link to `DEVELOPMENT.md`, `TESTING.md`, `AGENTS.md`, or plan
  docs as primary documentation entry points
- developer/source-build guidance remains available in `DEVELOPMENT.md`
- manual verification guidance explicitly checks the stricter README boundary

### Validation

- inspect `README.md`, `README.en.md`, `DEVELOPMENT.md`, `AGENTS.md`, `docs/plans/process.md`, and
  `TESTING.md`

### Item Record

- Chosen repair mode:
  `Mode B (user-specified)`
- Implementation notes:
  removed developer/test documentation links from the top-level README surfaces and tightened the
  permanent guardrails so future README reviews can reject contributor/test/governance navigation
  drift mechanically.
- Validation results:
  `2026-03-28`: content review confirmed `README.md` and `README.en.md` now keep only
  user/operator-facing documentation entry points.
  `2026-03-28`: `TESTING.md`, `AGENTS.md`, and `docs/plans/process.md` now explicitly forbid
  top-level README navigation to `DEVELOPMENT.md`, `TESTING.md`, `AGENTS.md`, `PLAN.md`, and
  versioned planning docs.

## R3: Android Workflow Throughput Optimization

### Problem

The Android CI and release jobs already cache Gradle dependencies, but they still spend avoidable
time reacquiring fixed-version SDK components on every fresh runner.

### Goal

Reduce Android workflow setup time further without introducing stale, opaque, or overbroad caches.

### Repair Modes

- Mode A: `Keep only Gradle cache`
  Safe, but leaves repeated SDK downloads untouched.
- Mode B: `Cache fixed-version Android SDK components with explicit keys`
  Improves setup time while keeping invalidation tied to versioned inputs.
- Mode C: `Cache the whole Android SDK tree`
  Faster in the happy path, but too broad and harder to reason about when stale.

### Recommended Mode

- Mode B

Reason:

- it respects the user's request to speed up Actions while keeping cache expiry explicit
- it stays aligned with the repository guardrail against broad SDK snapshots
- it targets the expensive setup work that remains after Gradle caching

### Planned Scope

- add bounded SDK-component caches to the Android jobs in `ci.yml` and `release.yml`
- key those caches on runner OS plus API/build-tools/NDK versions
- keep the cached paths limited to versioned Android SDK components actually installed by the job
- document the cache boundary in workflow/process guardrails and release notes

### Acceptance Criteria

- Android CI and release jobs restore fixed-version SDK components from explicit cache keys when
  available
- cache scope remains limited to versioned SDK component directories and licenses, not the whole
  SDK tree
- workflow and guardrail docs make the cache boundary discoverable

### Validation

- inspect `.github/workflows/ci.yml` and `.github/workflows/release.yml`
- run local workflow-adjacent validation where feasible

### Item Record

- Chosen repair mode:
  `Mode B (user-specified)`
- Implementation notes:
  added fixed-version Android SDK component caches for CI and release Android jobs, keyed by runner
  OS plus API/build-tools/NDK versions, while keeping the install step as a correctness backstop.
- Validation results:
  `2026-03-28`: workflow review confirmed `.github/workflows/ci.yml` and
  `.github/workflows/release.yml` now restore bounded Android SDK component caches with explicit
  invalidation inputs.
  `2026-03-28`: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after the workflow
  and release-note updates.
