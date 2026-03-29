# Vibe Everywhere Linux CLI ABI Compatibility And musl Packaging Remediation Plan v11

Last updated: 2026-03-29

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v11`.
- The concise lookup view lives in [`v11-summary.md`](./v11-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the Linux CLI ABI-compatibility defect that surfaced
when Debian 12 operators attempted to run the hosted-runner-built Linux `gnu` binaries.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current Linux CLI release path is too tightly coupled to the GitHub-hosted runner baseline:

1. the `Release` workflow builds the default Linux CLI archive on `ubuntu-24.04` as
   `x86_64-unknown-linux-gnu`, which can drag in a newer `glibc` requirement than generic
   self-hosted operators have available
2. the Linux installer resolves that same `gnu` archive by default, so operator installs inherit
   the ABI mismatch instead of avoiding it
3. the repository's testing and release guidance does not explicitly verify the Linux CLI ABI
   target or assert that the published Linux binaries are statically linked

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Linux CLI ABI Compatibility And musl Packaging | completed locally | none |

## Shared Remediation Guardrails

- Do not ship the default generic Linux CLI archive from a hosted runner in a way that silently
  inherits a newer `glibc` baseline than common self-hosted operators can satisfy.
- Keep installer asset resolution, CI validation, and release packaging aligned to the same Linux
  target instead of documenting one ABI target and shipping another.
- Do not treat Windows EasyTier runtime packaging rules as evidence that Linux compatibility should
  stay on `gnu`; Windows side-by-side runtime files and Linux `glibc` compatibility are separate
  problems with different repair strategies.

## R1: Linux CLI ABI Compatibility And musl Packaging

### Problem

The generic Linux CLI archive is currently built as `x86_64-unknown-linux-gnu` on a modern hosted
runner, which caused Debian 12 startup failures with `GLIBC_2.39 not found` when launching
`vibe-agent`.

### Goal

Make the default Linux CLI release archive broadly runnable across self-hosted Linux distributions
without depending on the hosted-runner `glibc` baseline.

### Repair Modes

- Mode A: `Keep GNU packaging and lower the Linux build baseline`
  Build `x86_64-unknown-linux-gnu` on an older runner or container baseline so the required
  `glibc` version drops.
- Mode B: `Switch the default Linux CLI archive to musl static packaging`
  Publish a statically linked `x86_64-unknown-linux-musl` CLI archive, align installers and docs
  to that archive, and explicitly verify the static result in `CI` and `Release`.
- Mode C: `Dual-ship GNU and musl Linux CLI archives`
  Preserve the old `gnu` asset while adding a `musl` asset, at the cost of more packaging,
  documentation, and installer-selection complexity.

### Recommended Mode

- Mode A

Reason:

- it keeps the Linux target closest to the existing runtime behavior
- it avoids introducing a release-target switch at the same time as the compatibility repair
- it is the lower-risk packaging change if compatibility can be restored cleanly

### Planned Scope

- switch the default Linux CLI release asset from `x86_64-unknown-linux-gnu` to
  `x86_64-unknown-linux-musl`
- update `scripts/install-relay.sh` so Linux x86_64 installs resolve the shipped `musl` asset
  by default
- add explicit Linux CLI `musl` build and static-binary verification to `CI`
- add explicit static Linux CLI verification to the `Release` workflow before packaging the Linux
  archive
- update README, self-hosted deployment docs, testing guidance, release notes, planning records,
  and durable guardrails to reflect the new Linux CLI packaging model

### Acceptance Criteria

- the default Linux CLI release asset name is
  `vibe-everywhere-cli-<tag>-x86_64-unknown-linux-musl.tar.gz`
- the Linux installer resolves `x86_64-unknown-linux-musl` on x86_64 hosts by default
- `CI` builds the Linux CLI binaries for `x86_64-unknown-linux-musl` and verifies they are static
- the `Release` workflow packages Linux CLI binaries from
  `target/x86_64-unknown-linux-musl/release`
- user/operator docs explain that the published Linux x86_64 CLI archive is statically linked and
  no longer tied to the hosted-runner `glibc` baseline

### Validation

- `cargo fmt --all --check`
- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`
- `cargo test --workspace --all-targets -- --nocapture`
- `cd apps/vibe-app && npm run build`
- `cargo build --locked --release --target x86_64-unknown-linux-musl -p vibe-relay -p vibe-agent`
- `file target/x86_64-unknown-linux-musl/release/vibe-relay`
- `file target/x86_64-unknown-linux-musl/release/vibe-agent`
- `bash -n scripts/install-relay.sh`
- `./scripts/render-release-notes.sh v0.1.9 >/dev/null`

### Item Record

- Chosen repair mode:
  `Mode B (user-specified musl-only Linux CLI release packaging)`
- Implementation notes:
  switched the default Linux CLI release asset, installer target resolution, and workflow
  verification path to `x86_64-unknown-linux-musl`; kept Windows EasyTier side-by-side runtime
  packaging unchanged; and updated operator docs, release notes, testing guidance, and durable
  repository guardrails to describe the new Linux packaging model consistently.
- Validation results:
  `2026-03-29`: `cargo fmt --all --check` succeeded after the Linux musl packaging changes.
  `2026-03-29`: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  the Linux musl packaging changes.
  `2026-03-29`: `cargo test --locked --workspace --all-targets -- --nocapture` succeeded after
  the Linux musl packaging changes.
  `2026-03-29`: `cd apps/vibe-app && npm run build` succeeded after the Linux musl packaging
  changes.
  `2026-03-29`: `cargo build --locked --release --target x86_64-unknown-linux-musl -p vibe-relay -p vibe-agent`
  succeeded; `file` reported both binaries as `static-pie linked`, and `readelf -d` showed no
  `NEEDED` dynamic-library entries.
  `2026-03-29`: `bash -n scripts/install-relay.sh` succeeded after the Linux installer target
  switch.
  `2026-03-29`: `./scripts/render-release-notes.sh v0.1.9 >/dev/null` succeeded after the release
  note update.
  `2026-03-29`: local PowerShell parser validation for `scripts/install-relay.ps1` could not be
  run because `pwsh` was not installed in the local environment.
