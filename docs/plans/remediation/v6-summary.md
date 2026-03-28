# Remediation Plan v6 Summary

Last updated: 2026-03-28

## Scope

Version 6 covers the Windows EasyTier runtime packaging repair discovered after the first required
Windows smoke rollout:

- aligning Windows smoke validation with a packaged side-by-side runtime layout
- keeping Windows CLI archives and installer output consistent with EasyTier-style runtime
  distribution
- preventing Windows relay installation from extracting only `vibe-relay.exe` while omitting the
  required EasyTier runtime files

Full implementation detail lives in [`v6-details.md`](./v6-details.md).

## Status

| Item | Title | Status | Depends On | Recommended Mode |
| --- | --- | --- | --- | --- |
| R1 | Windows EasyTier Runtime Packaging Alignment | completed | none | Mode B (user-specified EasyTier-style side-by-side packaging) |

## Current Target

- Active item:
  `completed locally`
- Required next step:
  push the packaging repair and monitor the triggered GitHub Actions workflows, especially `CI`
  on `main`, before treating remediation `v6` as fully closed
- Previous completed plan:
  remediation plan `v5`, covering overlay truthfulness, README boundary tightening, and Android
  workflow throughput

## Lookup Notes

- Need the full problem statement, repair modes, acceptance criteria, and validation rules:
  read [`v6-details.md`](./v6-details.md).
- Need the mandatory execution workflow before starting an item:
  read [`../process.md`](../process.md).
