# Remediation Plan v9 Summary

Last updated: 2026-03-28

## Scope

Version 9 covers the workflow/doc reconciliation work that remained after remediation `v8`
validated the hosted Linux EasyTier `no_tun` path:

- promoting the hosted Linux overlay path from a non-blocking diagnostic back to a required gate
- aligning the release publish dependency chain with that separate required gate
- removing stale documentation and release-note claims that still described the superseded
  diagnostic-only state

Full implementation detail lives in [`v9-details.md`](./v9-details.md).

## Status

| Item | Title | Status | Depends On | Recommended Mode |
| --- | --- | --- | --- | --- |
| R1 | Hosted Linux Overlay Gate Restoration And Workflow/Docs Reconciliation | completed | v8 | Mode A (user-specified) |

## Current Target

- Active item:
  `completed`
- Required next step:
  remediation plan `v9` is complete after GitHub-hosted `CI` run `23689144327` restored the
  hosted Linux overlay gate as a verified required path on `main`
- Previous completed plan:
  remediation `v8`, which validated the hosted Linux `no_tun` path on GitHub-hosted `CI`

## Lookup Notes

- Need the full problem statement, repair modes, acceptance criteria, and validation rules:
  read [`v9-details.md`](./v9-details.md).
- Need the mandatory execution workflow before starting an item:
  read [`../process.md`](../process.md).
