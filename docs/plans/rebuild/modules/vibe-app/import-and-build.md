# Module Plan: vibe-app/import-and-build

## Purpose

Import `happy-app` into `packages/vibe-app` and establish a repeatable buildable baseline before
any significant adaptation work starts.

## Happy Source Of Truth

- `packages/happy-app/*`
- root Happy workspace configuration relevant to the app

## Target Rust/Vibe Location

- package: `packages/vibe-app`

## Responsibilities

- import source tree
- preserve initial package structure
- make the package installable and buildable in this repository

## Non-Goals

- branding changes
- endpoint changes
- protocol changes

## Public Types And Interfaces

- app package scripts
- workspace integration points

## Data Flow

- source files are copied/imported from Happy
- workspace package metadata is renamed to Vibe
- build scripts are adjusted only as needed to run in this repo

## Dependencies

- `shared/naming.md`
- `projects/vibe-app.md`

## Implementation Steps

1. Import the Happy app tree with minimal structural changes.
2. Rename package metadata to `vibe-app` while keeping code changes minimal.
3. Restore working package manager configuration and baseline scripts.
4. Record any source-level `happy` references left for later adaptation.

## Edge Cases And Failure Modes

- workspace path assumptions pointing back to Happy repo
- build scripts requiring root-level files not yet imported

## Tests

- install verification
- baseline build verification
- script resolution smoke test

## Acceptance Criteria

- `packages/vibe-app` exists as a buildable imported baseline

## Open Questions

- None.

## Locked Decisions

- import first, adapt second
- preserve Happy directory layout initially
