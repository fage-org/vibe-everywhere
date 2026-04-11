# Module Plan: vibe-app-tauri/social-and-developer-surface-disposition

## Status

- planned for Wave 10

## Purpose

Resolve the product status of social and developer-only surfaces so they stop existing in an
ambiguous state between implemented code, planned routes, and customer-visible claims.

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/modules/vibe-app-tauri/validation-and-customer-capability-contract.md`
- `/root/happy/packages/happy-app/sources/app/(app)/friends/**`
- `/root/happy/packages/happy-app/sources/app/(app)/dev/**`

## Target Location

- social and developer-route visibility rules in `packages/vibe-app-tauri`
- active planning and top-level capability docs

## Responsibilities

- decide whether social surfaces are productized, hidden, or explicitly deferred
- decide whether developer-only routes remain exposed, hidden, or internal-only
- align active app wording and route visibility with those decisions

## Non-Goals

- speculative social feature expansion without a product decision
- exposing developer-only pages to customers by default

## Dependencies

- `platform-parity-and-browser-contract`

## Implementation Steps

1. Audit social and developer surfaces against current code and business value.
2. Choose one disposition for each family: productize, hide, or defer in writing.
3. Update route visibility and docs to match the chosen disposition.
4. Remove any residual wording that implies those surfaces are current commitments when they are not.

## Edge Cases And Failure Modes

- backend support existing without a product decision
- developer routes remaining discoverable in customer-facing builds
- deferred social surfaces lingering in summaries as if they are near-complete

## Tests

- route visibility review for all social and developer surfaces
- customer-facing capability list review

## Acceptance Criteria

- social and developer surfaces have explicit disposition decisions
- active docs and visible navigation reflect those decisions
- no ambiguous "present but not really supported" route family remains

## Locked Decisions

- hidden is preferable to ambiguous
- a deferred route family must be named as deferred, not simply left half-visible
