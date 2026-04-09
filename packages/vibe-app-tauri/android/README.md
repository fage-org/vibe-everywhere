This directory is the canonical, repository-owned Android project for `vibe-app-tauri`.

Tauri CLI still expects `src-tauri/gen/android`, so the package uses a bridge symlink:

- source of truth: `packages/vibe-app-tauri/android`
- Tauri bridge: `packages/vibe-app-tauri/src-tauri/gen/android`

Use `yarn mobile:android:prepare` to restore the bridge if it is missing.
