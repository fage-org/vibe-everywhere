# Vibe Everywhere Release Version-Source Synchronization And Publish Validation Remediation Plan v12

Last updated: 2026-03-30

Version note:

- This file is the versioned detailed remediation plan for repair epoch `v12`.
- The concise lookup view lives in [`v12-summary.md`](./v12-summary.md).
- The planning workflow rules live in [`../process.md`](../process.md).

## Purpose

This file is the authoritative repair plan for the release version-source drift that surfaced when
the published `v0.1.12` Android APK installed with internal version `0.1.11`.

Use this file together with:

- [`../../../PLAN.md`](../../../PLAN.md): concise execution record, status, completion log, and
  verification log
- [`../../../AGENTS.md`](../../../AGENTS.md): repository-level engineering, documentation,
  networking, release, and workflow guardrails

## Problem Summary

The current release process allowed a published release tag and asset filenames to move ahead of
the app's embedded version metadata:

1. `v0.1.12` was cut while the repository version sources still declared `0.1.11`
2. the `Release` workflow renamed Android artifacts with `${RELEASE_TAG}`, so the published asset
   appeared to be `v0.1.12` even though the APK metadata still reported `0.1.11`
3. neither `CI` nor the release verification path failed on version-source drift before publishing

## Remediation Overview

| Item | Title | Status | Depends On |
| --- | --- | --- | --- |
| R1 | Release Version-Source Synchronization And Publish Validation | completed locally | none |

## Shared Remediation Guardrails

- Do not publish a release tag whose filename/versioned asset labels can disagree with the version
  embedded in the shipped product metadata.
- Keep the Rust workspace version and the Tauri/Vite app version sources synchronized before
  cutting a release tag.
- Make version-source consistency a fail-fast workflow check rather than a manual convention alone.

## R1: Release Version-Source Synchronization And Publish Validation

### Problem

The release pipeline trusted the pushed tag as the external asset version but did not verify that
the APK/Tauri/Cargo version sources matched it.

### Goal

Ensure future release tags cannot publish assets whose filenames advertise one version while the
installed app reports another.

### Repair Modes

- Mode A: `Keep repository version files as the source of truth and fail fast on mismatch`
  Bump the repository version files to the next intended release and add a workflow/script check
  that rejects inconsistent sources or a mismatched release tag.
- Mode B: `Derive app version metadata from the release tag at build time`
  Keep source files less explicit and inject the version into build inputs from the pushed tag.
- Mode C: `Allow manual pre-release drift and only warn in CI`
  Preserve the current workflow shape but emit diagnostics without blocking publish.

### Recommended Mode

- Mode A

Reason:

- it keeps the version source explicit in the repository and visible in review
- it fixes both local builds and release builds instead of only tag-triggered packaging
- it is the lowest-risk change for Tauri/Android packaging because it avoids hidden build-time
  mutation

### Planned Scope

- bump the workspace/app/Tauri version sources from `0.1.11` to `0.1.13`
- add a reusable repository script that verifies all version sources agree and optionally match a
  supplied release tag
- run that script in `CI` and in the `Release` workflow before packaging/publishing
- update testing guidance, release-note source, plan records, and `AGENTS.md` to make the rule
  durable

### Acceptance Criteria

- `Cargo.toml`, `apps/vibe-app/package.json`, `apps/vibe-app/package-lock.json`, and
  `apps/vibe-app/src-tauri/tauri.conf.json` all declare `0.1.13`
- `./scripts/verify-release-version.sh` succeeds when sources match and fails when they diverge
- `CI` fails if repository version sources drift
- `Release` fails before packaging if the pushed tag version does not match the synchronized source
  version
- testing guidance and durable repository guardrails explicitly call out release-version-source
  synchronization

### Validation

- `./scripts/verify-release-version.sh`
- `./scripts/verify-release-version.sh v0.1.13`
- `cargo fmt --all --check`
- `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app`
- `cd apps/vibe-app && npm run build`
- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`

### Item Record

- Chosen repair mode:
  `Mode A (user-specified keep explicit repository version sources and fail fast on mismatch)`
- Implementation notes:
  bumped the Rust workspace, Vite app, npm lockfile, and Tauri config version sources to `0.1.13`;
  added `scripts/verify-release-version.sh` as the shared consistency check; ran it in both `CI`
  and `Release`; and updated testing, release notes, plan records, and durable guardrails so tag
  and embedded app versions cannot silently diverge again.
- Validation results:
  `2026-03-30`: `./scripts/verify-release-version.sh` succeeded and reported synchronized source
  version `0.1.13`.
  `2026-03-30`: `./scripts/verify-release-version.sh v0.1.13` succeeded and confirmed tag/source
  alignment for the next intended release.
  `2026-03-30`: `cargo fmt --all --check` succeeded after the release version-source sync changes.
  `2026-03-30`: `cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app` succeeded after
  updating `Cargo.lock` for the `0.1.13` workspace version bump.
  `2026-03-30`: `cd apps/vibe-app && npm run build` succeeded after the app/Tauri version-source
  synchronization changes.
  `2026-03-30`: `./scripts/render-release-notes.sh v0.0.0 >/dev/null` succeeded after the next
  release-note source update.
