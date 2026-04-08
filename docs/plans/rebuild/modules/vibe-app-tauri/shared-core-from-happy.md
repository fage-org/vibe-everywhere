# Module Plan: vibe-app-tauri/shared-core-from-happy

## Purpose

Port the reusable Happy app logic into package-local shared modules inside `packages/vibe-app-tauri`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `/root/happy/packages/happy-app/sources/auth/**`
- `/root/happy/packages/happy-app/sources/sync/**`
- `/root/happy/packages/happy-app/sources/realtime/**`
- `/root/happy/packages/happy-app/sources/encryption/**`
- `/root/happy/packages/happy-app/sources/text/**`
- `/root/happy/packages/happy-app/sources/changelog/**`
- `/root/happy/packages/happy-app/sources/constants/**`
- `/root/happy/packages/happy-app/sources/utils/**`
- `/root/happy/packages/happy-app/sources/hooks/**`

## Target Location

- `packages/vibe-app-tauri/sources/shared/**`

## Responsibilities

- auth domain helpers
- sync and reducer logic
- realtime state helpers
- encryption helpers
- text, changelog, and constant ownership
- utility and hook extraction where UI-independent

## Non-Goals

- broad screen migration
- creating a new repo-wide shared package
- pulling over UI host primitives as part of the shared layer

## Dependencies

- `universal-bootstrap-and-runtime`
- `crates/vibe-wire`
- shared protocol and validation plans

## Implementation Steps

1. Sort Happy modules into reusable-as-is, reusable-after-adapter, and UI-only categories.
2. Port the reusable logic into package-local shared modules.
3. Replace Happy imports, names, and endpoint assumptions with Vibe equivalents.
4. Keep protocol compatibility aligned to `vibe-wire` instead of local app-specific types.
5. Record any logic that still needs platform adapters before mobile or desktop shells consume it.

## Edge Cases And Failure Modes

- hidden mobile-host assumptions inside sync or utility modules
- logic copied without matching current `vibe-wire` contract ownership
- accidental imports from `packages/vibe-app`
- over-extracting modules that should stay screen-local

## Tests

- unit tests for extracted auth/sync/realtime logic
- reducer/parser compatibility tests
- import guard checks to keep screen-level RN code out of shared core

## Acceptance Criteria

- desktop and mobile shells can both consume package-local shared core modules
- shared core no longer depends on `packages/vibe-app`
- protocol-critical logic stays aligned with shared crate contracts

## Locked Decisions

- shared logic lives in `packages/vibe-app-tauri` first
- no new shared package is created during early Wave 9 work
