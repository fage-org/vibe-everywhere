# Vibe Everywhere Iteration Specs v3

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed iteration plan for roadmap epoch `v3`.
- The concise lookup view lives in [`v3-summary.md`](./v3-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file records the workflow-normalization epoch that follows `v2` and the hosted Linux
EasyTier `no_tun` repair in remediation `v8`.

The current epoch is intentionally narrow: it turns the repaired hosted Linux overlay path back
into a required verification gate, aligns release publish dependencies with that gate, and removes
stale documentation that still describes the old non-blocking diagnostic state.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, completion log, verification
  log, and decision log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, workflow, release,
  testing, and documentation guardrails

## Roadmap Overview

| Iteration | Title | Status | Depends On |
| --- | --- | --- | --- |
| 13 | Workflow And Release Verification Normalization | completed | Iteration roadmap `v2`, remediation `v8` |

## Shared Guardrails

- Do not keep calling a repaired required gate `diagnostic` or `non-blocking` after the harness is
  stable enough to gate CI.
- Do not let a release publish job bypass a required verification gate merely because that gate is
  modeled as a separate workflow job.
- Hosted-runner workarounds such as EasyTier `no_tun` must stay harness-only and must not silently
  become product/runtime defaults.

## Iteration 13: Workflow And Release Verification Normalization

### Goal

Normalize the repository's GitHub Actions and documentation now that the hosted Linux overlay path
is stable enough to be a required gate again.

### User-Visible Outcome

- `CI` and `Release` both block on hosted Linux overlay smoke again.
- GitHub Release publishing no longer depends only on packaging jobs; it also depends on the
  required Linux overlay gate.
- Workflow docs, testing guidance, planning records, and release notes no longer claim that hosted
  Linux overlay coverage is merely a non-blocking diagnostic.

### In Scope

- restore hosted Linux overlay smoke as a blocking job in `.github/workflows/ci.yml`
- restore hosted Linux overlay smoke as a blocking job in `.github/workflows/release.yml`
- keep `VIBE_TEST_EASYTIER_NO_TUN=1` enabled only for the hosted Linux gate
- make the release publish job depend on the separate Linux overlay gate
- audit and update `TESTING.md`, release notes, plan indexes, active plan files, `PLAN.md`, and
  any stale historical plan pointers that still advertise the superseded workflow state

### Out Of Scope

- no product/runtime default changes for EasyTier or overlay behavior
- no new privileged runner infrastructure
- no attempt to claim direct hosted-runner overlay bridge or preview byte-path coverage that the
  GitHub-hosted no_tun environment still does not provide

### Acceptance Criteria

- `CI` and `Release` no longer treat hosted Linux overlay smoke as non-blocking
- the hosted Linux gate still uses the harness-only `VIBE_TEST_EASYTIER_NO_TUN=1` path rather than
  changing product defaults
- the release publish job cannot run unless the Linux overlay gate succeeds
- workflow, testing, planning, and release-note docs no longer describe the current hosted Linux
  overlay path as a non-blocking diagnostic

### Validation

- `cargo fmt --all --check`
- `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --locked --workspace --all-targets -- --nocapture`
- `bash -n scripts/dual-process-smoke.sh`
- `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay`
- `./scripts/dual-process-smoke.sh relay_polling`
- `./scripts/render-release-notes.sh v0.1.6 >/dev/null`
- monitor the next pushed `CI` workflow and the later tag-triggered `Release` workflow

### Iteration Record

- Chosen implementation mode:
  `user-specified blocking restoration with harness-only no_tun retained`
- Implementation notes:
  restored hosted Linux overlay smoke as a required job in both `CI` and `Release`, kept the
  hosted runner on the harness-only `VIBE_TEST_EASYTIER_NO_TUN=1` path, made release publishing
  depend on the separate Linux overlay job, and reconciled repository workflow/testing/release/plan
  docs with that restored gate.
- Validation results:
  `2026-03-28`: `cargo fmt --all --check` succeeded after the workflow/doc normalization changes.
  `2026-03-28`: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  the workflow/doc normalization changes.
  `2026-03-28`: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  the workflow/doc normalization changes.
  `2026-03-28`: `cd apps/vibe-app && npm run build` succeeded after the workflow/doc
  normalization changes.
  `2026-03-28`: `bash -n scripts/dual-process-smoke.sh` succeeded after the workflow/doc
  normalization changes.
  `2026-03-28`: `./scripts/dual-process-smoke.sh relay_polling` succeeded after the workflow/doc
  normalization changes.
  `2026-03-28`: `VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay` succeeded
  after the hosted Linux gate restoration changes.
  `2026-03-28`: `./scripts/render-release-notes.sh v0.1.6 >/dev/null` succeeded after the
  release-note updates.
  `2026-03-28`: GitHub Actions `CI` run `23689144327` completed successfully; jobs `69013607968`
  (`Verify`), `69013725596` (`Linux Overlay Smoke`), `69013725588`
  (`Windows Compatibility`), and `69013725582` (`Android Mobile`) all succeeded.
