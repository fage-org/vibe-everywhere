# Iteration Plan v3 Summary

Last updated: 2026-03-28

## Scope

Version 3 covers the post-stabilization workflow-normalization epoch after `v2` and remediation
`v8` proved the hosted Linux overlay path can be exercised truthfully on GitHub-hosted runners.

This version focuses on:

- restoring hosted Linux `overlay` smoke as a blocking gate in both `CI` and `Release`
- keeping the hosted gate truthful through the harness-only EasyTier `no_tun` path instead of
  changing product defaults
- aligning release publish dependencies with the separate Linux overlay gate
- auditing workflow, testing, planning, and release-note docs so they no longer advertise obsolete
  non-blocking overlay behavior

Full implementation detail lives in [`v3-details.md`](./v3-details.md).

## Status

| Iteration | Title | Status |
| --- | --- | --- |
| 13 | Workflow And Release Verification Normalization | completed |

## Current State

- Iteration 13 restores GitHub-hosted Linux `overlay` smoke as a blocking gate in both `CI` and
  `Release`, while keeping the hosted runner on the harness-only `VIBE_TEST_EASYTIER_NO_TUN=1`
  path.
- The release publish path now depends on the separate Linux overlay gate instead of depending only
  on packaging jobs.
- Repository docs are being reconciled so workflow names, testing guidance, release notes, and
  plan records all describe the same hosted-runner behavior.
- Local validation and GitHub-hosted `CI` verification are complete in this change set. Iteration
  13 closed after `CI` run `23689144327` succeeded.

## Lookup Notes

- Need detailed acceptance or implementation notes:
  read [`v3-details.md`](./v3-details.md).
- Need the baseline roadmap history:
  read [`v1-summary.md`](./v1-summary.md).
- Need the previous verification-hardening phase:
  read [`v2-summary.md`](./v2-summary.md).
- Need the active remediation track:
  read [`../remediation/v9-summary.md`](../remediation/v9-summary.md).
