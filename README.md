# vibe-remote

Happy-aligned remote coding stack rebuilt around Rust services and clients.

`vibe-remote` keeps the Happy product shape, but moves the server, CLI, agent, wire contracts, and
log sidecar into Rust while preserving the imported `packages/vibe-app` client. The repository is
usable as a self-hosted baseline today and remains plan-governed through [PLAN.md](/root/vibe-remote/PLAN.md).

## What ships here

- `crates/vibe-wire`: canonical shared protocol and compatibility fixtures
- `crates/vibe-server`: HTTP + Socket.IO backend for sessions, machines, auth, and updates
- `crates/vibe-agent`: remote-control CLI for machines and sessions
- `crates/vibe-cli`: local runtime wrapper for `claude`, `codex`, `gemini`, `openclaw`, and `acp`
- `crates/vibe-app-logs`: optional remote app log receiver
- `packages/vibe-app`: imported Happy app adapted to Vibe naming, endpoints, and release envs

## Quick Start

### 1. Install toolchains

- Rust stable with `cargo fmt`
- Node.js `24.14.0`
- Yarn `1.22.22`

```bash
git clone <your-fork-or-origin>
cd vibe-remote
yarn install --frozen-lockfile
cargo check --workspace
```

### 2. Start the backend

The current `vibe-server` baseline is intentionally simple:

- single process
- process-local in-memory database/cache state
- local file storage by default
- no Postgres or Redis required for local bring-up

Minimum local boot:

```bash
export VIBE_MASTER_SECRET='replace-with-a-long-random-secret'
cargo run -p vibe-server -- --host 0.0.0.0 --port 3005
```

Default API base URL is `http://127.0.0.1:3005`.

### 3. Point the clients at your server

```bash
export VIBE_SERVER_URL='http://127.0.0.1:3005'
export VIBE_WEBAPP_URL='http://127.0.0.1:3000'
```

For local CLI state, credentials, daemon metadata, and optional app logs, clients use
`~/.vibe` by default. Override it with:

```bash
export VIBE_HOME_DIR="$HOME/.vibe-dev"
```

### 4. Use the CLI

Run the local runtime:

```bash
# Start an auth flow
cargo run -p vibe-cli -- auth login

# Start the Codex wrapper
cargo run -p vibe-cli -- codex run --tag demo --prompt "Summarize this repository"

# Or the Claude wrapper
cargo run -p vibe-cli -- claude run --tag demo --prompt "Summarize this repository"
```

If you prefer a background machine endpoint for remote spawning and resume:

```bash
cargo run -p vibe-cli -- daemon install
cargo run -p vibe-cli -- daemon start
cargo run -p vibe-cli -- daemon status
```

### 5. Use the remote-control agent

`vibe-agent` is the control-side CLI for listing machines, spawning sessions, sending prompts, and
waiting for turns to finish.

```bash
# Authenticate
cargo run -p vibe-agent -- auth login

# List registered machines
cargo run -p vibe-agent -- machines --active

# Spawn a session on a machine
cargo run -p vibe-agent -- spawn --machine <machine-id> --path ~/project --agent codex

# Send a message and wait for idle
cargo run -p vibe-agent -- send <session-id> "Run the test suite" --wait
```

## Deployment

### Server

`vibe-server` only requires `VIBE_MASTER_SECRET` to start. The current parity baseline uses
process-local typed stores for account/session/machine state, so it should be deployed as a
single-instance service unless you replace the storage seams with durable adapters.

Core runtime variables:

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `VIBE_MASTER_SECRET` | yes | none | bearer-token and auth crypto root secret |
| `VIBE_SERVER_HOST` | no | `0.0.0.0` | bind host |
| `VIBE_SERVER_PORT` | no | `3005` | bind port |
| `VIBE_WEBAPP_URL` | no | `https://app.vibe.engineering` | web redirect base for auth/connect flows |
| `VIBE_IOS_STORE_URL` | no | `https://app.vibe.engineering/ios` | app version/update metadata |
| `VIBE_ANDROID_STORE_URL` | no | `https://app.vibe.engineering/android` | app version/update metadata |
| `VIBE_IOS_UP_TO_DATE` | no | `>=1.4.1` | minimum accepted iOS version string |
| `VIBE_ANDROID_UP_TO_DATE` | no | `>=1.4.1` | minimum accepted Android version string |

File storage defaults to `~/.vibe/server` and can be redirected with:

- `VIBE_DATA_DIR`
- `VIBE_PUBLIC_URL`
- `VIBE_S3_ENDPOINT`
- `VIBE_S3_REGION`
- `VIBE_S3_ACCESS_KEY`
- `VIBE_S3_SECRET_KEY`
- `VIBE_S3_BUCKET`
- `VIBE_S3_PUBLIC_URL`

If `VIBE_S3_ENDPOINT` is not set, uploads stay on the local filesystem.

### Optional app log receiver

The imported app can send remote console logs to `vibe-app-logs`.

```bash
export VIBE_HOME_DIR="$HOME/.vibe-dev"
cargo run -p vibe-app-logs -- --host 0.0.0.0 --port 8787
```

Relevant env vars:

- `VIBE_APP_LOGS_HOST`
- `VIBE_APP_LOGS_PORT`
- `VIBE_HOME_DIR`

Logs are stored under `${VIBE_HOME_DIR:-~/.vibe}/app-logs`.

### Web and app package

The app package stays in `packages/vibe-app` and uses Expo/Tauri release flows. For local web work:

```bash
yarn workspace vibe-app typecheck
yarn workspace vibe-app test --exclude 'sources/**/*.integration.spec.ts'
yarn workspace vibe-app expo export --platform web --output-dir dist
```

Primary app env vars:

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

An example Kubernetes manifest for the web export lives at
[packages/vibe-app/deploy/vibe-app.yaml](/root/vibe-remote/packages/vibe-app/deploy/vibe-app.yaml).

## Validation

Repository baseline checks:

```bash
cargo fmt --all --check
cargo check --workspace --locked
cargo test --workspace --locked
yarn workspace vibe-app typecheck
yarn workspace vibe-app test --exclude 'sources/**/*.integration.spec.ts'
```

The excluded app specs are long-running real-chain integration tests and are better suited to
targeted validation against a prepared local server environment.

## GitHub Actions

Three workflows are provided:

- `.github/workflows/ci.yml`
  - runs on push, pull request, and manual dispatch
  - validates the Rust workspace plus the stable `packages/vibe-app` test set
- `.github/workflows/release.yml`
  - runs on `v*` tags or manual dispatch with a tag
  - verifies the workspace version matches the release tag
  - builds `vibe`, `vibe-agent`, `vibe-server`, and `vibe-app-logs`
  - publishes tarballs and `sha256` files to a GitHub Release
- `.github/workflows/app-release.yml`
  - runs on `app-v*` tags or manual dispatch
  - validates `packages/vibe-app` once before packaging
  - exports the web bundle, builds Tauri desktop bundles on Linux/macOS/Windows, and builds
    Android locally on the GitHub runner with `expo prebuild --platform android` plus
    `./gradlew app:bundleRelease`
  - publishes app assets to a GitHub Release for `app-v*` tags

Release flow:

```bash
# 1. bump [workspace.package].version in Cargo.toml
# 2. commit the version change
# 3. create and push the matching tag
git tag vX.Y.Z
git push origin vX.Y.Z
```

The release workflow will fail if the tag does not match the root workspace version.

App release flow:

```bash
# build and publish app assets under a dedicated app tag
git tag app-vX.Y.Z
git push origin app-vX.Y.Z
```

App workflow notes:

- `web` is exported locally from `packages/vibe-app`
- `desktop` packages are built with Tauri on Linux, macOS, and Windows
- `android` packaging uses `expo prebuild --platform android` and `./gradlew app:bundleRelease`
  on the GitHub runner
- `android` packaging also requires `VIBE_EAS_PROJECT_ID` as a GitHub variable or secret so the
  app config resolves the correct Expo project metadata
- `VIBE_EAS_OWNER` is optional if it matches the GitHub repository owner; the workflow now falls
  back to `github.repository_owner`
- if Firebase-backed Android configuration is required, set `VIBE_GOOGLE_SERVICES_JSON` as a
  GitHub secret containing the JSON file contents

## Notes

- `vibe-server` currently targets the completed rebuild baseline, not a horizontally scaled
  production cluster.
- `packages/vibe-app` keeps package-local release scripts for EAS, OTA, and Tauri because app-store
  automation remains separate from the Rust binary release flow.
- `/root/happy` remains the behavioral source of truth for product concepts and parity checks.

## License

MIT, with Happy-origin attribution preserved in [LICENSE](/root/vibe-remote/LICENSE).
