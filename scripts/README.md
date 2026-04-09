# scripts

This directory is reserved for validation, migration, packaging, and release helpers introduced
during the rebuild.

Do not add ad hoc one-off scripts here without documenting their owner project and validation role
in the relevant plan files.

Current validation helpers:

- `validate-vibe-wire-fixtures.mjs`
  - owner: `vibe-wire`
  - role: validate published `crates/vibe-wire/fixtures/*.json` against Happy source-of-truth
    schemas
  - prerequisites: a local Happy checkout at `HAPPY_ROOT` or the default `/root/happy`
- `record-vibe-app-tauri-promotion-baseline.mjs`
  - owner: `vibe-app-tauri/promotion-and-vibe-app-deprecation`
  - role: capture the current bundle-size snapshot and emit the promotion review scaffold used for
    startup, performance, and memory sign-off notes
  - prerequisites: run `yarn workspace vibe-app-tauri build` first if you want the latest dist
    asset sizes in the generated report; run `yarn workspace vibe-app-tauri tauri:build` first if
    you also want release bundle outputs captured
  - optional output path: pass a markdown path after the script name when you need per-platform CI
    snapshots, for example `yarn --cwd scripts metrics:vibe-app-tauri artifacts/vibe-app-tauri/promotion-baseline-linux.md`
  - root alias: `yarn app:metrics`
- `validate-vibe-app-tauri-promotion.mjs`
  - owner: `vibe-app-tauri/promotion-and-vibe-app-deprecation`
  - role: validate the active Wave 9 promotion module, release migration plan, and route/capability
    matrix structure plus the generated promotion baseline artifact; use `--promotion-ready` only
    when the active promotion module and baseline evidence have no placeholder sign-off text left
  - prerequisites: `docs/plans/rebuild/modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`,
    `docs/plans/rebuild/vibe-app-tauri-wave9-migration-and-release-plan.md`,
    `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`, and
    `artifacts/vibe-app-tauri/promotion-baseline.md` must exist
  - strict command: `yarn --cwd scripts validate:vibe-app-tauri-promotion:ready`
  - root alias: `yarn app:promotion-ready`
- `validate-vibe-app-tauri-release.mjs`
  - owner: `vibe-app-tauri/release-ota-and-store-migration`
  - role: verify package-local Wave 9 release inputs exist and that `.github/workflows/app-release.yml`
    still packages from `packages/vibe-app-tauri`
  - command: `yarn --cwd scripts validate:vibe-app-tauri-release`
