# Module Plan: vibe-app-tauri/web-export-and-browser-runtime

## Purpose

Preserve the retained static browser export capability while `packages/vibe-app-tauri` serves as
the active Wave 9 replacement package and moves toward the default app path.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-unified-replacement-plan.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`
- `/root/happy/packages/happy-app/package.json`
- `/root/happy/packages/happy-app/app.config.js`
- `/root/happy/packages/happy-app/sources/app/_layout.tsx`
- `/root/happy/packages/happy-app/sources/app/(app)/_layout.tsx`
- `/root/happy/packages/happy-app/sources/theme.css`
- `/root/happy/packages/happy-app/sources/unistyles.ts`
- `/root/happy/packages/happy-app/sources/components/web/**`

## Wave 9 Canonical Inputs

- package-local browser build/export config
- browser entry HTML and browser-specific asset metadata
- package-local output path for retained static export artifacts

## Target Location

- `packages/vibe-app-tauri`
- retained static browser export config
- retained browser build/export scripts and output path
- browser-specific UI or adapter seams where the web runtime needs them

## Responsibilities

- retained static export boot verification
- retained static browser export generation
- browser-safe provider and route bootstrap integration
- asset, favicon, and browser metadata handling needed for export
- web-specific environment and path handling

## Non-Goals

- inventing a new standalone web product
- SSR/SEO work beyond what the retained export path requires
- replacing the desktop Tauri shell with the retained static browser export surface

## Dependencies

- `universal-bootstrap-and-runtime`
- `shared-core-from-happy`
- `mobile-shell-and-navigation`

## Implementation Steps

1. Port the retained browser export bootstrap requirements from Happy into `packages/vibe-app-tauri`.
2. Keep the retained static export aligned with the route and provider semantics it is expected to
   preserve.
3. Define a stable retained static browser export path for the replacement package that is distinct
   from Tauri desktop outputs.
4. Audit browser-only helpers, metadata, and asset-path handling explicitly.
5. Validate that retained static browser export continues to work after shared-core and route migration.

## Edge Cases And Failure Modes

- retained static export silently diverging from the route/provider contract it is expected to
  preserve
- asset paths or favicons breaking during static export
- font, theme, or hydration timing drifting on web-only boot
- retained static browser export relying on deprecated `packages/vibe-app` files implicitly

## Tests

- retained static export smoke test
- retained static browser export smoke test
- browser asset and metadata sanity check
- browser-only provider and metadata affordance check against the Wave 9 route matrix

## Acceptance Criteria

- `packages/vibe-app-tauri` can generate the retained static browser export output
- retained static browser export can be generated from `packages/vibe-app-tauri`
- retained static browser export no longer depends on `packages/vibe-app`

## Locked Decisions

- retained static browser export is an explicit Wave 9 scope item, not an implied side effect
- retained static browser export must remain tied to the replacement package rather than the
  deprecated legacy app
