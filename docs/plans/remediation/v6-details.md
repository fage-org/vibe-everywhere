# Vibe Everywhere Windows EasyTier Runtime Packaging Remediation Plan v6

Last updated: 2026-03-28

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v6`.
- The concise lookup view lives in [`v6-summary.md`](./v6-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the Windows runtime-packaging defect discovered
after adding the required Windows relay smoke workflow.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current Windows packaging path is inconsistent in three places:

1. the Windows smoke harness validates `target/debug` directly instead of a packaged side-by-side
   layout that matches how EasyTier distributes runtime files
2. the Windows CLI release archive stages the runtime files, but the repository does not use one
   shared packaging path for smoke and release
3. `scripts/install-relay.ps1` extracts only `vibe-relay.exe`, which can leave required EasyTier
   runtime files out of the installed directory

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Windows EasyTier Runtime Packaging Alignment | completed | none |

## Shared Remediation Guardrails

- Do not assume a Windows EasyTier-dependent binary is self-contained when required runtime
  DLL/SYS files are distributed separately upstream.
- Do not validate Windows smoke against a layout that is materially different from the shipped
  archive/install layout.
- Do not let the Windows relay installer copy only `vibe-relay.exe` when the packaged archive
  contains side-by-side runtime files needed by the binary family.

## R1: Windows EasyTier Runtime Packaging Alignment

### Problem

The repository already stages EasyTier runtime files for Windows release archives, but the smoke
script, packaging path, and relay installer do not consistently treat those files as a packaged
side-by-side unit.

### Goal

Align Windows smoke validation, release packaging, and relay installation to the same
EasyTier-style side-by-side runtime layout.

### Repair Modes

- Mode A: `Keep raw target-dir smoke and patch only the installer`
  Repairs one operator path but leaves smoke and release packaging drift in place.
- Mode B: `Unify smoke, archive staging, and installer around side-by-side packaging`
  Matches EasyTier's runtime-distribution model and keeps verification aligned with what ships.
- Mode C: `Rely on delay-load only`
  Avoids some startup failures but does not repair the packaging/install mismatch.

### Recommended Mode

- Mode B

Reason:

- it matches the user's explicit request to repair this using the EasyTier packaging model
- it fixes the operator install path instead of only masking startup behavior
- it keeps smoke verification closer to the layout operators actually download and run

### Planned Scope

- extend the Windows staging script so it can build a package-style destination directory that
  includes binaries plus side-by-side EasyTier runtime files
- make the Windows smoke harness launch relay and agent from that packaged directory
- make Windows CLI release packaging use the same staging script for its archive contents
- make `scripts/install-relay.ps1` install the side-by-side runtime files along with
  `vibe-relay.exe`
- update operator docs, release notes, testing guidance, plan records, and durable repository
  guardrails to reflect the packaging rule

### Acceptance Criteria

- Windows smoke launches `vibe-relay.exe` and `vibe-agent.exe` from a package-style directory that
  also contains the EasyTier runtime files
- Windows CLI release archives are staged through the shared packaging path
- `scripts/install-relay.ps1` fails fast on incomplete Windows archives and installs the runtime
  files beside `vibe-relay.exe` when they are present
- operator documentation and testing guidance explicitly describe the side-by-side runtime rule

### Validation

- `cargo fmt --all`
- `cargo check --locked -p vibe-relay -p vibe-agent`
- `pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/stage-windows-runtime.ps1',[ref]$null,[ref]$null)"`
- `pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/dual-process-smoke.ps1',[ref]$null,[ref]$null)"`
- `pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/install-relay.ps1',[ref]$null,[ref]$null)"`
- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`

### Item Record

- Chosen repair mode:
  `Mode B (user-specified EasyTier-style side-by-side packaging)`
- Implementation notes:
  upgraded the shared Windows runtime staging script to assemble package-style directories with the
  requested binaries plus EasyTier runtime files, switched the Windows smoke harness to launch
  from that packaged directory, routed the Windows CLI release archive through the same staging
  script, and changed the Windows relay installer to copy the side-by-side runtime files instead
  of extracting only `vibe-relay.exe`.
- Validation results:
  `2026-03-28`: `cargo check --locked -p vibe-relay -p vibe-agent` succeeded after the Windows
  packaging-script and installer changes.
  `2026-03-28`: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after the
  release-note update.
  `2026-03-28`: local PowerShell parser validation for `scripts/stage-windows-runtime.ps1`,
  `scripts/dual-process-smoke.ps1`, and `scripts/install-relay.ps1` could not be run because
  `pwsh` was not installed in the local environment; Windows GitHub Actions validation is still
  required after push.
