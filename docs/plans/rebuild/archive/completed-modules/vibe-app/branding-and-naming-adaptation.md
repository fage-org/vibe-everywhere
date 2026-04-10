# Module Plan: vibe-app/branding-and-naming-adaptation

## Purpose

Replace Happy-specific public naming in the imported app with Vibe naming while minimizing early
structural churn.

## Happy Source Of Truth

- `packages/happy-app/**` public app/package surfaces that are later imported into `packages/vibe-app`
- `shared/naming.md`

## Target Rust/Vibe Location

- package: `packages/vibe-app`
- focused edit areas:
  - app config
  - deep links
  - user-visible strings
  - package metadata

## Responsibilities

- rename app title, package identifiers, and visible labels
- switch public deep links from Happy to Vibe
- keep internal temporary source identifiers tracked until fully removed

## Non-Goals

- visual redesign

## Public Types And Interfaces

- app name and metadata
- deep-link scheme
- public strings

## Data Flow

- imported source is scanned for public `happy` surfaces
- public strings/config are updated to Vibe
- remaining internal identifiers are documented and deferred if harmless

## Dependencies

- `import-and-build`
- `shared/naming.md`

## Implementation Steps

1. Update package/app metadata and user-visible strings.
2. Update deep-link scheme and config names.
3. Record any intentionally deferred internal identifier cleanup.

## Edge Cases And Failure Modes

- changing internal keys that third-party services depend on
- partial rename leaving mixed public branding

## Tests

- string/config search audit
- deep-link config test
- build regression test

## Acceptance Criteria

- public app surfaces read as Vibe, not Happy

## Open Questions

- None.

## Locked Decisions

- prioritize public-surface rename first
- internal source identifier cleanup is secondary and must be deliberate
