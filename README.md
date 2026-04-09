# vibe-remote

Happy-aligned remote coding stack rebuilt around Rust services and clients.

`vibe-remote` keeps the Happy product shape, but moves the server, CLI, agent, wire contracts, and
log sidecar into Rust while treating `packages/vibe-app` as a deprecated legacy import/reference.
`packages/vibe-app-tauri` is now the default app path and active release owner, and the repository
remains plan-governed through [PLAN.md](./PLAN.md).

## What ships here

- `crates/vibe-wire`: canonical shared protocol and compatibility fixtures
- `crates/vibe-server`: HTTP + Socket.IO backend for sessions, machines, auth, and updates
- `crates/vibe-agent`: remote-control CLI for machines and sessions
- `crates/vibe-cli`: local runtime wrapper for `claude`, `codex`, `gemini`, `openclaw`, and `acp`
- `crates/vibe-app-logs`: optional remote app log receiver
- `packages/vibe-app-tauri`: default app path and active release owner for desktop, Android APK, and retained browser export
- `packages/vibe-app`: deprecated legacy import kept only as a Vibe-specific reference when Happy is insufficient

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

### App package status

Active app ownership now lives in `packages/vibe-app-tauri`. The legacy `packages/vibe-app`
package is deprecated from active CI and release lanes and should only be consulted when
`/root/happy/packages/happy-app` cannot answer a Vibe-specific continuity question.

Current active local app validation commands:

```bash
yarn workspace vibe-app-tauri typecheck
yarn workspace vibe-app-tauri test
yarn workspace vibe-app-tauri tauri:test
yarn workspace vibe-app-tauri tauri:smoke
```

Primary active app env vars now revolve around the default package and its active release
ownership:

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

Legacy `packages/vibe-app` deploy manifests and release files remain in-repo only as historical
reference material.

## Validation

Repository PR-ready baseline:

```bash
cargo fmt --all --check
cargo check --workspace --locked
cargo test --workspace --locked
yarn workspace vibe-app-tauri typecheck
yarn workspace vibe-app-tauri test
yarn workspace vibe-app-tauri tauri:test
yarn workspace vibe-app-tauri tauri:smoke
yarn --cwd scripts validate:vibe-app-tauri-promotion
```

Legacy `packages/vibe-app` validation is intentionally out of the active baseline. Use it only as a
manual reference path when Happy cannot answer a Vibe-specific question.

Additional required checks depend on the touched scope:

- if `crates/vibe-wire` changes, also run `cargo run --example export-fixtures -p vibe-wire` and
  `yarn --cwd scripts validate:vibe-wire-fixtures`
- if app release, promotion, or rollout docs change, also run `yarn app:promotion-ready`
- if a module plan defines extra tests, smoke checks, or release evidence, run them or report the
  block explicitly in the change summary

Repository quality gates:

- no hardcoded secrets in docs, examples, workflows, or committed config
- plan status, execution status, and active workflow ownership must stay consistent across `PLAN.md`,
  `docs/plans/rebuild/`, and root entry docs
- coverage expectations are enforced per project and module plan; keep or improve automated test
  coverage on touched surfaces and do not skip required acceptance checks

## GitHub Actions

Three workflows are provided:

- `.github/workflows/ci.yml`
  - runs on push, pull request, and manual dispatch
  - validates the Rust workspace plus the active `packages/vibe-app-tauri` test/build set
- `.github/workflows/release.yml`
  - runs on `v*` tags or manual dispatch with a tag
  - verifies the workspace version matches the release tag
  - builds `vibe`, `vibe-agent`, `vibe-server`, and `vibe-app-logs`
  - publishes tarballs and `sha256` files to a GitHub Release
- `.github/workflows/app-release.yml`
  - runs on `app-v*` tags or manual dispatch
  - validates and packages the active `packages/vibe-app-tauri` desktop, browser-export, and
    Android APK lanes
  - the deprecated `packages/vibe-app` web/desktop/android lanes remain intentionally disabled
  - publishes active app artifacts to a GitHub Release for `app-v*` tags

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

- the active workflow currently packages `packages/vibe-app-tauri` desktop bundles on Linux, macOS,
  and Windows as direct installer assets, plus a retained browser-export archive and Android APK
  artifacts
- the deprecated `packages/vibe-app` web/desktop/android lanes are not built in CI anymore
- Android APK, retained browser export, and release-oriented desktop ownership now sit on `packages/vibe-app-tauri`
- if `packages/vibe-app` must be inspected, treat it as a legacy Vibe-specific reference only when Happy is insufficient

## Notes

- `vibe-server` currently targets the completed rebuild baseline, not a horizontally scaled
  production cluster.
- `packages/vibe-app` is deprecated from active CI/release use and remains in-repo only as a legacy
  Vibe-specific reference when Happy cannot answer a continuity question.
- `/root/happy` remains the behavioral source of truth for product concepts and parity checks.

## License

MIT, with Happy-origin attribution preserved in [LICENSE](./LICENSE).
