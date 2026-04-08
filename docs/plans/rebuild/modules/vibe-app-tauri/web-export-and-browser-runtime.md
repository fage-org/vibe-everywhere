# Module Plan: vibe-app-tauri/web-export-and-browser-runtime

## Purpose

Preserve the browser runtime and static web export capabilities while `packages/vibe-app-tauri`
serves as the active Wave 9 replacement package and moves toward the default app path.

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

## Target Location

- `packages/vibe-app-tauri`
- browser runtime bootstrap and config
- static web export scripts and output path
- browser-specific UI or adapter seams where the web runtime needs them

## Responsibilities

- browser-safe Expo runtime boot
- static web export generation
- browser-safe provider and route bootstrap integration
- asset, favicon, and browser metadata handling needed for export
- web-specific environment and path handling

## Non-Goals

- inventing a new standalone web product
- SSR/SEO work beyond what the retained export path requires
- replacing the desktop Tauri shell with the browser runtime

## Dependencies

- `universal-bootstrap-and-runtime`
- `shared-core-from-happy`
- `mobile-shell-and-navigation`

## Implementation Steps

1. Port the browser runtime bootstrap requirements from Happy into `packages/vibe-app-tauri`.
2. Keep the browser runtime aligned with the same route and provider semantics as the mobile shell.
3. Define a stable `expo export --platform web --output-dir dist` path for the replacement package.
4. Audit browser-only helpers, metadata, and asset-path handling explicitly.
5. Validate that retained browser export continues to work after shared-core and route migration.

## Edge Cases And Failure Modes

- browser runtime silently diverging from mobile route/provider behavior
- asset paths or favicons breaking during static export
- font, theme, or hydration timing drifting on web-only boot
- web export relying on deprecated `packages/vibe-app` files implicitly

## Tests

- browser runtime boot smoke test
- `expo export --platform web --output-dir dist` smoke test
- browser asset and metadata sanity check
- browser-only provider and metadata affordance check against the Wave 9 route matrix

## Acceptance Criteria

- `packages/vibe-app-tauri` can boot in the browser runtime
- static web export can be generated from `packages/vibe-app-tauri`
- retained browser export no longer depends on `packages/vibe-app`

## Locked Decisions

- retained browser export is an explicit Wave 9 scope item, not an implied side effect
- browser export must remain tied to the replacement package rather than the deprecated legacy app
