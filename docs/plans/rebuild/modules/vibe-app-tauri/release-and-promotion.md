# Module Plan: vibe-app-tauri/release-and-promotion

## Purpose

Define release packaging, validation, rollout, and promotion criteria for `vibe-app-tauri`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-parity-checklist.md`
- `docs/plans/rebuild/vibe-app-tauri-coexistence-matrix.md`
- `docs/plans/rebuild/shared/repo-release-and-docs.md`
- repository packaging workflows
- current `packages/vibe-app` desktop release behavior

## Target Location

- `packages/vibe-app-tauri`
- repository-level app release automation
- promotion/deprecation notes

## Responsibilities

- package desktop release artifacts
- maintain the parity checklist and define promotion gates
- document coexistence with the current app
- collect and document realistic session-load performance and memory review artifacts before
  promotion
- define when and how the old desktop path is deprecated

## Non-Goals

- immediate replacement of `packages/vibe-app`
- mobile release changes

## Dependencies

- `bootstrap-and-package`
- `session-ui-parity`
- `secondary-surfaces`
- `desktop-platform-adapters`

## Implementation Steps

1. Define package-local release scripts.
2. Add desktop package validation and release automation.
3. Keep `docs/plans/rebuild/vibe-app-tauri-parity-checklist.md` updated as the sign-off artifact
   instead of creating a new checklist file.
4. Record explicit promotion criteria, including Linux, macOS, and Windows startup validation plus
   realistic session-load performance/memory review.
5. Record fallback/deprecation strategy for the old desktop path.

## Edge Cases And Failure Modes

- ambiguous default desktop package ownership
- shipping two desktop packages without clear naming/positioning
- promoting before parity is measured
- promoting before runtime performance and memory behavior are reviewed on realistic desktop loads

## Tests

- desktop release artifact smoke tests
- startup verification on Linux, macOS, and Windows
- realistic session-load performance and memory review
- parity checklist review before promotion

## Acceptance Criteria

- desktop release artifacts are produced reliably
- promotion criteria are explicit
- realistic session-load runtime review is documented before promotion
- coexistence with the old desktop path is documented

## Locked Decisions

- `packages/vibe-app` remains the default shipping path until explicit promotion
- release/promotion is gated on parity review, not package existence alone
