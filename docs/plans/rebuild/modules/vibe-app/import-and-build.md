# Module Plan: vibe-app/import-and-build

## Purpose

Import `happy-app` into `packages/vibe-app` and establish a repeatable buildable baseline before
any significant adaptation work starts.

## Happy Source Of Truth

- `packages/happy-app/*`
- root Happy workspace configuration relevant to the app:
  - `package.json`
  - `yarn.lock`
  - `scripts/postinstall.cjs`
  - `patches/fix-pglite-prisma-bytes.cjs`
  - `environments/environments.ts` when package scripts still depend on it
  - root release/bootstrap helpers if imported app scripts still shell out to them

## Target Rust/Vibe Location

- package: `packages/vibe-app`

## Responsibilities

- import source tree
- preserve initial package structure
- make the package installable and buildable in this repository
- make all required root bootstrap files explicit instead of assuming a Happy monorepo exists

## Non-Goals

- branding changes
- endpoint changes
- protocol changes

## Public Types And Interfaces

- app package scripts
- workspace integration points
- root bootstrap files required to install or build the imported app

## Data Flow

- source files are copied/imported from Happy into `packages/vibe-app`
- the minimum required Happy root files are copied or localized into this repo
- workspace package metadata is renamed to Vibe
- imported scripts are patched only enough to run in this repo without pulling the rest of Happy

## Dependencies

- `shared/naming.md`
- `shared/source-crosswalk.md`
- `projects/vibe-app.md`

## Import Manifest

### Mandatory phase-one imports

- `packages/happy-app/**`
- root `package.json`
- root `yarn.lock`
- `scripts/postinstall.cjs`
- `patches/fix-pglite-prisma-bytes.cjs`

### Conditional imports

Import these only if the imported app still references them after the first package rename pass:

- `environments/environments.ts`
- root `scripts/release.cjs`
- `packages/happy-app/CHANGELOG.md`
- any additional root helper script transitively required by `packages/happy-app/package.json` or
  `packages/happy-app/release.cjs`

### Mandatory audit targets after import

- `packages/vibe-app/tsconfig.json`
  - remove or replace the stale `hello-world` path alias
- `packages/vibe-app/sources/scripts/parseChangelog.ts`
  - remove or localize the package-local `../../CHANGELOG.md` dependency
- `packages/vibe-app/src-tauri/**`
  - audit all `../../node_modules/...` schema or tooling references
- imported package scripts and release scripts
  - remove assumptions that the full Happy workspace exists at the repository root
- imported dependency on `@slopus/happy-wire`
  - replace with the Vibe-owned wire package or a temporary compatibility package path
- root `scripts/postinstall.cjs`
  - prevent it from blindly building `@slopus/happy-wire` in a missing Happy workspace

## Implementation Steps

1. Copy `packages/happy-app/**` into `packages/vibe-app` without structural rewrites.
2. Copy the mandatory phase-one root files listed above.
3. Rename package metadata from `happy-app` to `vibe-app` while keeping internal source changes
   minimal.
4. Recreate the minimum root workspace/package-manager configuration needed for the imported app to
   install.
5. Localize or patch root postinstall behavior so it applies required patches but does not assume
   the full Happy workspace still exists.
6. Audit and patch all root-relative references from inside `packages/vibe-app`.
7. Record every remaining `happy` identifier that is still required for compatibility.
8. Run install and build validation from the imported baseline before any branding or API changes.

## Edge Cases And Failure Modes

- workspace path assumptions pointing back to Happy repo
- build scripts requiring root-level files not yet imported
- app install mutating node_modules through an untracked patch step
- root postinstall attempting to build `@slopus/happy-wire` before the Vibe replacement exists
- Tauri config or release scripts resolving files outside the imported package boundary

## Tests

- install verification
- baseline build verification
- script resolution smoke test
- explicit smoke check that no remaining root-relative path points back to `/root/happy`

## Acceptance Criteria

- `packages/vibe-app` exists as a buildable imported baseline
- every required non-package Happy root file is listed explicitly in this plan
- every known root-relative import/build assumption has an owner and a cleanup action

## Open Questions

- None.

## Locked Decisions

- import first, adapt second
- preserve Happy directory layout initially
- phase one explicitly imports the Happy root `package.json`, `yarn.lock`, `scripts/postinstall.cjs`,
  and `patches/fix-pglite-prisma-bytes.cjs` as bootstrap artifacts
- root postinstall behavior must be localized before first stable Vibe install; imported app
  bootstrap may temporarily reference Happy-era script logic, but it must not keep a hard runtime
  dependency on the missing Happy workspace
