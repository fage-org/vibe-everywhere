# vibe-app

Imported app baseline adapted to Vibe naming, endpoint wiring, and desktop metadata.

Key validation commands:

- `yarn workspace vibe-app typecheck`
- `yarn workspace vibe-app test`
- `yarn workspace vibe-app expo export --platform web --output-dir dist`
- `yarn workspace vibe-app tauri:check`
- `yarn workspace vibe-app tauri:smoke`

Primary app runtime environment variables:

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

Notes:

- Android release builds should point `VIBE_GOOGLE_SERVICES_FILE` at a real Firebase config; the
  checked-in template lives at `packages/vibe-app/google-services.example.json`.
- OTA and EAS ownership metadata are intentionally env-driven so the app does not default back to
  legacy release infrastructure.
- EAS workflow OTA jobs pin both `environment` and inline `VIBE_APP_ENV` / `APP_ENV` so preview
  and production updates do not accidentally fall back to the default development variant.
- App version-check flows still depend on the server-side `VIBE_IOS_STORE_URL` and
  `VIBE_ANDROID_STORE_URL`; production deployments should set those to the real Vibe store pages.
