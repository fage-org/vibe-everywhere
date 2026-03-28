# Vibe Everywhere Hosted Linux Overlay Gate Restoration Remediation Plan v9

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v9`.
- The concise lookup view lives in [`v9-summary.md`](./v9-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the remaining workflow and documentation mismatch
after remediation `v8` validated the hosted Linux EasyTier `no_tun` path.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

Remediation `v8` proved the hosted Linux `no_tun` path can be exercised meaningfully on
GitHub-hosted runners, and `CI` run `23688459204` succeeded with that repair. However the
repository still had several mismatches:

1. `CI` still named the job `Overlay Diagnostics (Linux, non-blocking)` and kept
   `continue-on-error: true`
2. `Release` still treated the Linux overlay job as non-blocking and also failed to enable
   `VIBE_TEST_EASYTIER_NO_TUN=1`, so it did not match `CI`
3. the release `publish` job did not depend on the separate Linux overlay job, so a failed overlay
   gate could still leave a path to published assets
4. testing, planning, and release-note docs still described the hosted Linux overlay path as a
   deferred non-blocking diagnostic

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Hosted Linux Overlay Gate Restoration And Workflow/Docs Reconciliation | completed | v8 |

## Shared Remediation Guardrails

- Do not keep a repaired required verification gate named or documented as a non-blocking
  diagnostic.
- Do not let the release publish job bypass a required gate because that gate is modeled as a
  separate workflow job.
- Do not turn hosted-runner EasyTier `no_tun` into a product/runtime default.

## R1: Hosted Linux Overlay Gate Restoration And Workflow/Docs Reconciliation

### Problem

The hosted Linux `no_tun` repair is already good enough to gate `CI`, but the workflows and docs
still represent the older stopgap state. Leaving that mismatch in place weakens release integrity
and makes the repository claim two incompatible stories at once.

### Goal

Promote the hosted Linux `no_tun` path into a required verification gate while keeping it truthful,
and make the repository's workflows and docs describe that same truth consistently.

### Repair Modes

- Mode A: `Restore blocking hosted gate and keep it as a separate job`
  Remove non-blocking treatment, keep the hosted Linux overlay gate explicit, add the missing
  release env/config alignment, and make release publishing depend on that separate gate.
- Mode B: `Fold hosted Linux overlay back into verify`
  Restore blocking behavior by merging the hosted Linux overlay smoke steps into the main verify
  job, but lose the clearer job boundary and dedicated failure artifacts.
- Mode C: `Keep a non-blocking diagnostic and rely on release/manual review`
  Preserve the mismatch and keep a weaker release-integrity story even though the hosted gate is
  now stable enough to require.

### Recommended Mode

- Mode A

Reason:

- the user explicitly asked to restore blocking behavior now that the hosted issue is repaired
- it keeps the gate explicit in GitHub Actions instead of hiding it inside the main verify job
- it fixes the release-bypass risk without giving up parallel packaging work

### Planned Scope

- rename the hosted Linux overlay jobs in `CI` and `Release` to required smoke-gate names
- remove `continue-on-error: true`
- enable `VIBE_TEST_EASYTIER_NO_TUN=1` for the hosted Linux release gate so it matches `CI`
- make the release publish job depend on the separate Linux overlay gate
- update testing docs, release notes, plan indexes, active plan files, `PLAN.md`, and stale
  historical navigation pointers that still describe the old non-blocking state

### Acceptance Criteria

- hosted Linux overlay smoke is blocking in both `CI` and `Release`
- `Release` uses the same hosted Linux no_tun harness input as `CI`
- release publishing cannot proceed unless the Linux overlay gate succeeds
- repository docs no longer advertise the current hosted Linux overlay gate as non-blocking or
  deferred

### Validation

- `cargo fmt --all --check`
- `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --locked --workspace --all-targets -- --nocapture`
- `bash -n scripts/dual-process-smoke.sh`
- `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/render-release-notes.sh v0.1.6 >/dev/null`
- monitor the next pushed `CI` workflow, then the tag-triggered `Release` workflow

### Item Record

- Chosen repair mode:
  `Mode A (user-specified)`
- Implementation notes:
  restored hosted Linux overlay smoke as a required job in both workflows, aligned the release job
  with the hosted `VIBE_TEST_EASYTIER_NO_TUN=1` harness path, made GitHub Release publishing depend
  on the separate Linux overlay gate, and rewrote workflow/testing/release/plan docs that still
  described the older non-blocking diagnostic phase.
- Validation results:
  `2026-03-28`: `cargo fmt --all --check` succeeded after the hosted Linux gate-restoration
  workflow/doc changes.
  `2026-03-28`: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  the hosted Linux gate-restoration workflow/doc changes.
  `2026-03-28`: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  the hosted Linux gate-restoration workflow/doc changes.
  `2026-03-28`: `cd apps/vibe-app && npm run build` succeeded after the hosted Linux
  gate-restoration workflow/doc changes.
  `2026-03-28`: `bash -n scripts/dual-process-smoke.sh` succeeded after the hosted Linux
  gate-restoration workflow/doc changes.
  `2026-03-28`: `./scripts/dual-process-smoke.sh relay_polling` succeeded after the hosted Linux
  gate-restoration workflow/doc changes.
  `2026-03-28`: `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay` succeeded
  after the hosted Linux gate-restoration workflow/doc changes.
  `2026-03-28`: `./scripts/render-release-notes.sh v0.1.6 >/dev/null` succeeded after the
  release-note updates.
  `2026-03-28`: GitHub Actions `CI` run `23689144327` completed successfully; jobs `69013607968`
  (`Verify`), `69013725596` (`Linux Overlay Smoke`), `69013725588`
  (`Windows Compatibility`), and `69013725582` (`Android Mobile`) all succeeded.
