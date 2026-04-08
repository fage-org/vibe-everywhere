# vibe-app (Deprecated)

Deprecated legacy reference-only package.

Do not use `packages/vibe-app` for active CI, release, or new feature work.
Use `packages/vibe-app-tauri` as the sole active app package.

Consult `packages/vibe-app` only when `/root/happy/packages/happy-app` cannot answer a
Vibe-specific historical continuity question.

## Historical Snapshot

This package is the imported Happy baseline previously adapted to Vibe naming, endpoint wiring, and
desktop metadata.

Former validation commands retained here for historical reference only:

- `yarn workspace vibe-app typecheck`
- `yarn workspace vibe-app test --exclude 'sources/**/*.integration.spec.ts'`
- `yarn workspace vibe-app expo export --platform web --output-dir dist`
- `yarn workspace vibe-app tauri:check`
- `yarn workspace vibe-app tauri:smoke`

Former primary app runtime environment variables:

- `EXPO_PUBLIC_VIBE_SERVER_URL`
- `EXPO_PUBLIC_VIBE_LOG_SERVER_URL`
- `EXPO_PUBLIC_VIBE_POSTHOG_KEY`
- `EXPO_PUBLIC_VIBE_REVENUE_CAT_APPLE`
- `EXPO_PUBLIC_VIBE_REVENUE_CAT_GOOGLE`
- `EXPO_PUBLIC_VIBE_REVENUE_CAT_STRIPE`
- `VIBE_APP_ENV`
- `VIBE_EAS_PROJECT_ID`
- `VIBE_EAS_UPDATE_URL`
- `VIBE_EAS_OWNER`
- `VIBE_GOOGLE_SERVICES_FILE`
- `VIBE_IOS_AUTO_SUBMIT_PROFILE`
- `VIBE_ANDROID_AUTO_SUBMIT_PROFILE`

Historical notes:

- Android release builds used `VIBE_GOOGLE_SERVICES_FILE` against
  `packages/vibe-app/google-services.example.json`.
- OTA and EAS ownership metadata were env-driven to avoid falling back to legacy release
  infrastructure.
- App version-check flows depended on server-side `VIBE_IOS_STORE_URL` and
  `VIBE_ANDROID_STORE_URL`.
- The old repository-level app packaging workflow lived at
  `/root/vibe-remote/.github/workflows/app-release.yml` before the active pipeline moved to
  `packages/vibe-app-tauri`.
