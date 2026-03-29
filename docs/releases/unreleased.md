## Highlights

- Replace the control-app primary IA with a Poe-style device/project home and a Telegram-style
  project chat flow, including project grouping by working directory and project-scoped topic
  history.
- Prevent release tags from drifting away from the app's internal version metadata by adding an
  explicit version-source verification step to CI and release publishing.
- Prepare the next shipped app version as `0.1.13` so Android/Tauri package metadata matches the
  intended release line again.

## Included Iterations And Remediations

- Iteration v5 / Poe-style device-project navigation and Telegram-like project chat shell
- Remediation v12 / Release version-source synchronization and publish-time validation

## Operator Notes

- The default client flow now starts from a device/project chat home. Relay configuration lives
  under `Menu > Settings > Server` instead of the primary chat surface.
- Future release tags now fail fast if `Cargo.toml`, `apps/vibe-app/package.json`,
  `apps/vibe-app/package-lock.json`, and `apps/vibe-app/src-tauri/tauri.conf.json` do not agree
  with each other or with the pushed tag version.

## Validation

- `cargo check -p vibe-relay -p vibe-agent -p vibe-app`
- `cd apps/vibe-app && npm run build`
- `./scripts/verify-release-version.sh v0.1.13`
- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`
