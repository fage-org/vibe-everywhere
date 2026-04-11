# Module Plan: vibe-app-tauri/validation-and-customer-capability-contract

## Status

- planned for Wave 10

## Purpose

Define the active completion contract for `packages/vibe-app-tauri` so the repository stops
confusing route presence with feature completion.

## Source Of Truth

- `docs/plans/rebuild/projects/vibe-app-tauri.md`
- `docs/plans/rebuild/master-details.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `README.md`
- `TESTING.md`
- active app-facing docs under the repository root

## Target Location

- active planning docs
- active top-level documentation
- active validation documentation and command guidance

## Responsibilities

- define capability classes for app surfaces
- define customer-safe wording rules
- define which evidence types are required before a capability can be claimed as complete
- update validation docs to match the Wave 10 standard

## Non-Goals

- implementing product behavior directly
- broad UI redesign

## Dependencies

- none; this is the Wave 10 foundation module

## Implementation Steps

1. Create a capability classification model for visible app surfaces.
2. Define the evidence required for each class: code path, state path, tests, and platform scope.
3. Rewrite active top-level docs so they use the new contract.
4. Remove or downgrade any language that implies unsupported completion.

## Edge Cases And Failure Modes

- route tests being misused as evidence of product closure
- helper flows being misrepresented as first-class integrations
- platform downgrades disappearing from user-facing language

## Tests

- documentation consistency review against active app code
- validation command inventory review

## Acceptance Criteria

- the repository has one active definition of app completion
- active docs no longer overstate app product status
- later Wave 10 modules can classify their output using this module's rules

## Locked Decisions

- route presence alone is never enough
- customer wording must match platform scope
- validation guidance is part of the capability contract, not an afterthought
