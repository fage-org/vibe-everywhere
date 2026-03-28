# Vibe Everywhere Verification Integrity Remediation Plan v2

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v2`.
- The concise lookup view lives in [`v2-summary.md`](./v2-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the current release-verification integrity issues
identified after remediation plan `v1` completed.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, product, and testing
  guardrails

This remediation track exists because the product/runtime fixes are already shipped locally, but one
release-critical verification path still relied on a best-effort bypass and therefore weakened the
meaning of a green release workflow.

## Execution Rule

Every remediation item below has a `Repair Modes` subsection.

Before implementing any item:

1. present the available repair modes to the user
2. ask the user which mode to use
3. do not start coding until that item-level choice is confirmed

After an item is completed and verified:

1. update the item status in this file
2. record the chosen repair mode
3. record validation results
4. append a concise completion note to [`../../../PLAN.md`](../../../PLAN.md)

## Problem Summary

The current problems fall into two closely related buckets:

1. the release workflow still treats overlay smoke coverage as best-effort, so a broken overlay
   verification path can still publish a green release
2. the repository needs an explicit audit of other tests or verification checks that may have been
   softened in a similar way, with a distinction between true test compromises and explicit
   build-mode fallbacks

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Overlay Smoke Stabilization And Release Gate Restoration | completed | none |
| R2 | Verification Compromise Audit | completed | R1 |

## Shared Remediation Guardrails

- Do not solve release verification by silently deleting the overlay smoke coverage.
- Do not reintroduce product-facing loopback defaults while stabilizing a test-only harness.
- Do not keep a release-critical test as best-effort once a stable harness configuration exists.
- Do not classify non-test configuration fallbacks as test compromises; document them accurately.
- Do not leave future debugging opaque; smoke failures must report enough state to diagnose why the
  harness failed.

## R1: Overlay Smoke Stabilization And Release Gate Restoration

### Problem

The release workflow currently wraps `./scripts/dual-process-smoke.sh overlay` in a `set +e`
section that always exits `0`. The latest release showed that overlay smoke could fail on shared CI
runners during same-host EasyTier startup while the release still published successfully.

### Goal

Make the overlay smoke harness reliable enough for shared CI runners and restore overlay smoke as a
blocking release verification step.

### Repair Modes

- Mode A: `Keep best-effort gating and improve diagnostics only`
  Preserve the current release bypass, but emit richer logs so failures are easier to inspect.
- Mode B: `Stabilize same-host overlay smoke and restore blocking gate`
  Make the smoke topology deterministic for CI, keep the verification path meaningful, and remove
  the release bypass.
- Mode C: `Move overlay smoke out of release into a separate workflow`
  Avoid release flakiness by relocating overlay coverage to nightly or manual verification.

### Recommended Mode

- Mode B

Reason:

- the product claims overlay-assisted transport as part of the supported runtime model
- a green release should mean overlay verification actually passed, not merely that the packaging
  jobs succeeded
- the observed failure came from same-host harness instability, which can be solved without
  weakening product/runtime guardrails

### Planned Scope

- stabilize overlay smoke bootstrap behavior for same-host CI runs
- make overlay node IP publication deterministic in the smoke harness
- improve overlay timeout diagnostics to show the last observed overlay summary
- remove the `best-effort` release bypass and let overlay smoke fail the release when it is broken
- update release documentation so it no longer advertises overlay smoke as best-effort

### Acceptance Criteria

- `./scripts/dual-process-smoke.sh overlay` passes locally under the stabilized same-host harness
- the release workflow runs overlay smoke as a blocking step
- release docs no longer describe overlay smoke as best-effort
- a future overlay smoke failure reports the last observed overlay state, node IP, relay URL, and
  last error

### Validation

- `./scripts/dual-process-smoke.sh overlay`
- `./scripts/dual-process-smoke.sh relay_polling`
- inspect `.github/workflows/release.yml` to confirm overlay smoke no longer forces `exit 0`

### Item Record

- Chosen repair mode:
  `Mode B`, based on the user's request to fix the underlying issue rather than keep the release
  bypass
- Implementation notes:
  stabilized the same-host overlay harness by keeping the relay/API host behavior unchanged while
  forcing the overlay bootstrap path to use a test-only loopback bootstrap host and a deterministic
  static overlay node IP. This prevents shared-runner topology drift from leaving the agent stuck
  at `connected` with no usable `nodeIp`, and the smoke script now prints the last observed overlay
  summary before timing out.
- Validation results:
  `2026-03-28`: `./scripts/dual-process-smoke.sh overlay` succeeded after the harness change.
  `2026-03-28`: inspection of [`.github/workflows/release.yml`](../../../.github/workflows/release.yml)
  confirmed that overlay smoke no longer forces `exit 0`.

## R2: Verification Compromise Audit

### Problem

Once one release-critical test is found to be best-effort, the repository needs a deliberate audit
to confirm whether similar compromises exist elsewhere.

### Goal

Identify and classify all comparable verification compromises, remove true test compromises where
possible, and clearly document any remaining explicit non-test fallbacks.

### Repair Modes

- Mode A: `Document-only audit`
  List current compromises and keep the existing behavior unchanged.
- Mode B: `Eliminate true test compromises, document explicit build fallbacks`
  Remove softened test gates that should be blocking, while preserving clearly documented
  non-testing fallbacks such as unsigned release-signing behavior.
- Mode C: `Split compromised checks into informational workflows`
  Keep the main workflows strict by moving all soft checks into separate informational runs.

### Recommended Mode

- Mode B

Reason:

- it keeps the main workflow trustworthy without conflating release packaging fallbacks with test
  integrity
- it gives future maintainers a clean distinction between acceptable build-mode degradation and
  unacceptable soft-failed verification

### Planned Scope

- search workflow files, scripts, and release/testing docs for best-effort or forced-success
  verification patterns
- classify each finding as either:
  - a true compromised test/check that should be blocking
  - or an explicit build/configuration fallback that is not a test compromise
- update docs and guardrails to record the final classification

### Acceptance Criteria

- the audit result is written into the remediation record
- no release-critical smoke test remains intentionally non-blocking
- remaining `exit 0` or fallback behavior is explicitly documented as non-test behavior

### Validation

- repository search for `continue-on-error`, `set +e`, forced `exit 0`, `best-effort`, and similar
  markers
- documentation review of workflow descriptions in README and TESTING guidance

### Item Record

- Chosen repair mode:
  `Mode B`
- Implementation notes:
  searched workflow files, scripts, README, TESTING guidance, and planning/guardrail docs for
  `continue-on-error`, `set +e`, forced-success `exit 0`, `best-effort`, `non-blocking`, and
  similar markers. The only true compromised release-critical verification path was the release
  workflow's overlay smoke wrapper, which has now been removed. The remaining notable forced-success
  path in [`.github/workflows/release.yml`](../../../.github/workflows/release.yml) is the Android
  signing precondition branch, which is a build-mode fallback for missing signing secrets rather
  than a softened test.
- Validation results:
  `2026-03-28`: repository audit found no remaining release-critical smoke tests intentionally left
  as best-effort or forced-success checks after the overlay smoke gate restoration.
