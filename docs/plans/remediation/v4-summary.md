# Remediation Plan v4 Summary

Last updated: 2026-03-28

## Scope

Version 4 covers the next remediation tranche after release hygiene `v3`:

- moving overlay fallback from smoke-only expectations into relay runtime bridge health management
- restoring user-facing top-level documentation and splitting developer entry points into a
  dedicated guide
- reducing Android CI and release latency with bounded dependency caches

Full implementation detail lives in [`v4-details.md`](./v4-details.md).

## Status

| Item | Title | Status | Depends On | Recommended Mode |
| --- | --- | --- | --- | --- |
| R1 | Overlay Bridge Runtime Fallback And Auto-Recovery | completed | none | Mode B (user-specified) |
| R2 | README User-Facing Rewrite And Developer Guide Split | completed | none | Mode B (user-specified) |
| R3 | CI And Release Cache Optimization | completed | R2 | Mode B (user-specified) |

## Current Target

- Active item:
  `completed`
- Required next step:
  push the verified changes, then monitor the triggered GitHub Actions workflows according to the
  process rules
- Last completed plan:
  remediation plan `v4`, covering overlay runtime resilience, README ownership cleanup, and
  Android workflow cache optimization

## Lookup Notes

- Need the full problem statements, repair modes, acceptance criteria, and validation rules:
  read [`v4-details.md`](./v4-details.md).
- Need the mandatory execution workflow before starting an item:
  read [`../process.md`](../process.md).
