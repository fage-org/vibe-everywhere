# Remediation Plan v7 Summary

Last updated: 2026-03-28

## Scope

Version 7 covers the GitHub-hosted Linux overlay diagnostic instability that remained after the
Windows EasyTier runtime-packaging repair was closed:

- stabilizing same-runner overlay harness port allocation across relay, agent, and target helpers
- preserving the raw embedded EasyTier stop reason so future hosted-runner failures are diagnosable
  without guessing from a generic wrapper message

Full implementation detail lives in [`v7-details.md`](./v7-details.md).

## Status

| Item | Title | Status | Depends On | Recommended Mode |
| --- | --- | --- | --- | --- |
| R1 | Linux Overlay Diagnostic Harness Stabilization | completed | none | Mode A (user-confirmed) |

## Current Target

- Active item:
  `completed`
- Required next step:
  remediation plan `v7` is complete; the next hosted-runner repair tranche is tracked in
  remediation plan `v8` instead of expanding this file into a different phase
- Last completed item:
  remediation `v7` closed after GitHub-hosted `CI` run `23687951251` confirmed the harness-port
  and raw-error changes, exposed the distinct Linux hosted-runner TUN-permission limit, and
  handed that new issue to remediation `v8`

## Lookup Notes

- Need the full problem statement, repair modes, acceptance criteria, and validation rules:
  read [`v7-details.md`](./v7-details.md).
- Need the mandatory execution workflow before starting an item:
  read [`../process.md`](../process.md).
