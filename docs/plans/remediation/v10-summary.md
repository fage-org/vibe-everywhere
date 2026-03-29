# Remediation Plan v10 Summary

Last updated: 2026-03-29

## Scope

Version 10 covers the relay and agent auth-boundary hardening work that remained after the
tenant/user/role baseline:

- separating agent enrollment from the human control-plane token
- issuing and persisting device credentials for post-registration device traffic
- removing trust in external `x-vibe-*` actor headers
- enforcing control-only versus device-only auth boundaries across task, shell, preview,
  workspace, and Git routes
- aligning installers, release notes, operator docs, and smoke harnesses with the split-token
  model

Full implementation detail lives in [`v10-details.md`](./v10-details.md).

## Status

| Item | Title | Status | Depends On | Recommended Mode |
| --- | --- | --- | --- | --- |
| R1 | Relay And Agent Auth Boundary Hardening | completed locally | Iteration 11 baseline | Mode A (user-specified) |

## Current Target

- Active item:
  `completed locally`
- Required next step:
  push the auth-hardening change set and monitor the next GitHub Actions runs before treating it as
  fully delivered remotely
- Previous completed plan:
  remediation `v9`, which restored the hosted Linux overlay gate and reconciled workflow/docs

## Lookup Notes

- Need the full problem statement, repair modes, acceptance criteria, and validation rules:
  read [`v10-details.md`](./v10-details.md).
- Need the mandatory execution workflow before starting an item:
  read [`../process.md`](../process.md).
