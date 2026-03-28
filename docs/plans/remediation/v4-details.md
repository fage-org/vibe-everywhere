# Vibe Everywhere Overlay Runtime, Documentation, And CI Remediation Plan v4

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v4`.
- The concise lookup view lives in [`v4-summary.md`](./v4-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the runtime, documentation, and workflow problems
identified after remediation plan `v3` completed.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current problems fall into three related buckets:

1. overlay smoke revealed that the relay still trusted overlay connectivity too early and could
   stall work on bridge startup instead of falling back promptly
2. top-level README files drifted back toward repository-governance language and contributor
   content instead of staying focused on operators and end users
3. Android CI and release jobs still spend too much time rebuilding Gradle dependency state that
   could be restored safely from dependency-aware caches

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Overlay Bridge Runtime Fallback And Auto-Recovery | completed | none |
| R2 | README User-Facing Rewrite And Developer Guide Split | completed | none |
| R3 | CI And Release Cache Optimization | completed | R2 |

## Shared Remediation Guardrails

- Do not hide runtime transport defects behind smoke-test heuristics alone when the production relay
  can be fixed directly.
- Do not let overlay preference depend only on overlay membership when bridge reachability is still
  unknown.
- Do not reintroduce repository-governance or planning language into top-level user documentation.
- Do not keep contributor/build instructions in the top-level README once a dedicated developer
  entry document exists.
- Do not add broad CI caches with unclear invalidation just to reduce build time.

## R1: Overlay Bridge Runtime Fallback And Auto-Recovery

### Problem

The relay currently treats `overlay connected + node IP present` as enough evidence to select
overlay transport. If the task, shell, or port-forward bridge is unreachable or slow to start, the
relay can stall instead of falling back quickly. The smoke failure showed this most clearly on the
task path, but the same class of weakness exists on the other overlay bridge paths.

### Goal

Move overlay resilience into relay runtime behavior:

- fail fast on bridge connect or startup timeout
- fall back to relay-native transport when overlay bridge startup is not healthy
- probe for bridge recovery in the background
- return to overlay automatically for future work after recovery

### Repair Modes

- Mode A: `Smoke-only ready-check tightening`
  Improve CI detection, but leave product runtime behavior unchanged.
- Mode B: `Runtime bridge health management plus auto-recovery`
  Add connect/start timeouts, fallback, temporary bridge suppression, and background recovery
  probes inside the relay.
- Mode C: `Disable overlay transport for tasks until a larger redesign`
  Avoid the failure mode, but regress product capability and user experience.

### Recommended Mode

- Mode B

Reason:

- it fixes the product path instead of only making CI fail faster
- it matches the user's requested behavior of silent fallback plus automatic return to overlay
- it scales across task, shell, and port-forward bridge types instead of solving one path in
  isolation

### Planned Scope

- add relay-side overlay bridge health tracking by device and bridge kind
- apply explicit connect and startup-ack timeouts for overlay task, shell, and port-forward bridge
  sessions
- mark unhealthy bridges as temporarily unavailable and route new work through relay-native
  transports while unhealthy
- start background probes that clear the temporary unavailability when the bridge becomes reachable
  again
- add focused tests for fallback, startup timeout, and automatic health recovery

### Acceptance Criteria

- overlay task startup no longer hangs indefinitely when the bridge accepts but never starts
- task, shell, and port-forward transport selection respect temporary bridge unavailability
- a recovered bridge becomes eligible for overlay again without user action
- no product-facing relay defaults are reintroduced while implementing the fix

### Validation

- `cargo test -p vibe-relay -- --nocapture`
- `./scripts/dual-process-smoke.sh overlay`
- review the relay runtime selection paths in `tasks.rs`, `shell.rs`, and `port_forwards.rs`

### Item Record

- Chosen repair mode:
  `Mode B (user-specified)`
- Implementation notes:
  added relay-side overlay bridge health tracking keyed by device plus bridge kind, introduced
  connect/startup timeouts and background recovery probes, and wired task, shell, and port-forward
  selection plus bridge startup paths through the shared runtime health logic.
- Validation results:
  `2026-03-28`: `cargo test -p vibe-relay -- --nocapture` succeeded after adding overlay bridge
  health tracking, startup timeout handling, and recovery tests.
  `2026-03-28`: `./scripts/dual-process-smoke.sh overlay` succeeded after the runtime fallback and
  auto-recovery changes.

## R2: README User-Facing Rewrite And Developer Guide Split

### Problem

The top-level README files drifted back toward internal governance language and contributor details,
including content that belongs in process docs, release docs, or developer setup guidance instead
of user/operator entry pages.

### Goal

Restore the README surfaces to user/operator documentation and move contributor entry content into a
dedicated developer guide.

### Repair Modes

- Mode A: `Trim only the most obvious internal language`
  Reduces some noise, but still leaves README structure mixed.
- Mode B: `Rewrite README around users and split developer entry into DEVELOPMENT.md`
  Re-establish clear document ownership for operators versus contributors.
- Mode C: `Replace README with links only`
  Makes the repo entry page too thin for users evaluating the product.

### Recommended Mode

- Mode B

Reason:

- it directly matches the user's requirement
- it prevents recurring drift by giving developer content a stable home
- it keeps top-level docs readable for operators and evaluators

### Planned Scope

- rewrite `README.md` and `README.en.md` around product value, quick start, downloads, and doc
  entry points
- remove internal governance, anti-hardcoding, and release-process wording from the top-level
  README files
- move developer/source-build instructions into `DEVELOPMENT.md`
- tighten `docs/self-hosted.md` so it stays operational and user-facing
- codify the README/document-ownership rule in `AGENTS.md`, `docs/plans/process.md`, and
  `TESTING.md`

### Acceptance Criteria

- top-level README files read as user/operator entry pages
- contributor setup and source-build commands live in `DEVELOPMENT.md`
- documentation guardrails explicitly prevent future governance drift into the README files
- manual verification guidance checks this boundary going forward

### Validation

- inspect `README.md`, `README.en.md`, `DEVELOPMENT.md`, and `docs/self-hosted.md`
- confirm `TESTING.md`, `AGENTS.md`, and `docs/plans/process.md` reflect the documentation
  boundary

### Item Record

- Chosen repair mode:
  `Mode B (user-specified)`
- Implementation notes:
  rewrote `README.md` and `README.en.md` around user/operator entry points, moved contributor and
  source-build guidance into `DEVELOPMENT.md`, tightened `docs/self-hosted.md`, and codified the
  documentation-surface rule in `AGENTS.md`, `docs/plans/process.md`, and `TESTING.md`.
- Validation results:
  `2026-03-28`: file-content review confirmed the top-level README files no longer carry internal
  governance or developer-entry content.
  `2026-03-28`: `cd apps/vibe-app && npm run build` succeeded after the documentation split and
  README rewrite.

## R3: CI And Release Cache Optimization

### Problem

Android CI and release builds still spend avoidable time recreating Gradle dependency state from
scratch even though the relevant inputs are dependency-manifest-driven and safe to cache with
bounded invalidation.

### Goal

Reduce build latency without introducing opaque or hard-to-debug caches.

### Repair Modes

- Mode A: `Leave workflows unchanged`
  Lowest risk, but no latency improvement.
- Mode B: `Add dependency-aware Gradle caches to Android jobs`
  Improve latency while keeping invalidation tied to repository-owned inputs.
- Mode C: `Cache broad Android SDK directories`
  Potentially faster, but higher risk of stale or hard-to-diagnose failures.

### Recommended Mode

- Mode B

Reason:

- it matches the user's request for faster CI while explicitly respecting cache-expiry concerns
- it is the bounded cache shape most compatible with release reliability
- it improves the slowest Android jobs without broad SDK snapshots

### Planned Scope

- enable Gradle dependency caching for Android jobs in `ci.yml`
- enable the same cache strategy for Android jobs in `release.yml`
- keep cache invalidation bound to Gradle dependency descriptors in the generated Android project
- codify cache-scope expectations in repository guardrails

### Acceptance Criteria

- Android CI and release jobs restore Gradle caches from dependency-aware inputs
- no broad Android SDK cache is introduced without explicit invalidation policy
- workflow docs and guardrails explain the intended cache boundary

### Validation

- inspect `.github/workflows/ci.yml` and `.github/workflows/release.yml`
- run local workflow-adjacent validation where feasible

### Item Record

- Chosen repair mode:
  `Mode B (user-specified)`
- Implementation notes:
  enabled Gradle dependency caching in the Android jobs for both `CI` and `Release`, with cache
  invalidation scoped to generated Android Gradle descriptors and wrapper inputs instead of broad
  SDK-directory caching.
- Validation results:
  `2026-03-28`: workflow review confirmed Android jobs in both `.github/workflows/ci.yml` and
  `.github/workflows/release.yml` now restore Gradle caches from dependency-aware inputs.
  `2026-03-28`: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after the release
  workflow and notes updates.
