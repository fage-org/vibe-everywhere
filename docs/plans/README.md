# Vibe Everywhere Planning Index

Last updated: 2026-03-29

## Purpose

This directory is the single entry point for roadmap, remediation, and planning-process lookup.

The planning system is intentionally split into:

- one concise index file for fast lookup
- one shared process/governance file for mandatory workflow rules
- versioned `summary` files for quick status reading
- versioned `details` files for full implementation guidance and acceptance criteria

## Active Plan Set

| Plan Type | Active Version | Summary | Details | Status |
| --- | --- | --- | --- | --- |
| Iteration roadmap | v4 | [`iterations/v4-summary.md`](./iterations/v4-summary.md) | [`iterations/v4-details.md`](./iterations/v4-details.md) | active |
| Problem remediation | v10 | [`remediation/v10-summary.md`](./remediation/v10-summary.md) | [`remediation/v10-details.md`](./remediation/v10-details.md) | active |
| Process governance | shared | [`process.md`](./process.md) | n/a | mandatory |

## Query Guide

- Want the current roadmap or remediation status fast:
  read the active `summary` file.
- Want the full implementation and acceptance detail:
  read the matching `details` file.
- Want the mandatory workflow for how plans must be created, versioned, updated, and approved:
  read [`process.md`](./process.md).
- Want the top-level repository execution record:
  read [`../../PLAN.md`](../../PLAN.md).

## Versioning Rule

- `v1`, `v2`, `v3`, and later versions represent planning epochs, not code releases.
- A new version is created when the current phase is materially complete or the existing plan pair
  becomes too large to be practical.
- Old versions stay readable as historical references and must not be overwritten into unrelated
  new phases.

## Update Rule

- Every active iteration or remediation change must update:
  - the matching active `summary` file
  - the matching active `details` file
  - [`../../PLAN.md`](../../PLAN.md)
- Any newly discovered long-term rule must also be written into
  [`../../AGENTS.md`](../../AGENTS.md).
- After code is pushed to GitHub, the triggered Actions workflows must be checked and their outcome
  reported before the delivery is considered complete.
- Release-affecting work must also update the next-release note source under
  `docs/releases/` in the same change set.
