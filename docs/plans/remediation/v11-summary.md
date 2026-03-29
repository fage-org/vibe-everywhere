# Remediation Plan v11 Summary

Last updated: 2026-03-29

## Scope

Version 11 covers the Linux CLI ABI-compatibility repair discovered after Debian 12 startup
failures from the hosted-runner `gnu` release build:

- replacing the default Linux CLI release target with a statically linked `musl` target
- aligning Linux installer asset resolution with the shipped `musl` archive
- making `CI`, release packaging, testing guidance, and operator docs verify the same Linux target

Full implementation detail lives in [`v11-details.md`](./v11-details.md).

## Status

| Item | Title | Status | Depends On | Recommended Mode |
| --- | --- | --- | --- | --- |
| R1 | Linux CLI ABI Compatibility And musl Packaging | completed locally | none | Mode A; implemented as Mode B (user-specified) |

## Current Target

- Active item:
  `completed locally`
- Required next step:
  push the Linux musl packaging change set, monitor the next `CI` run on `main`, then cut and
  monitor the `v0.1.9` release tag before treating the repair as remotely delivered
- Previous completed plan:
  remediation `v10`, which split agent enrollment from the human control-plane token and issued
  device credentials

## Lookup Notes

- Need the full problem statement, repair modes, acceptance criteria, and validation rules:
  read [`v11-details.md`](./v11-details.md).
- Need the mandatory execution workflow before starting an item:
  read [`../process.md`](../process.md).
