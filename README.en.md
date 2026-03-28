# Vibe Everywhere

[![CI](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/ci.yml)
[![Release](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml/badge.svg)](https://github.com/fage-ac-org/vibe-everywhere/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

[中文](./README.md) | [English](./README.en.md)

Rust-first remote AI control plane: `Rust relay + Rust agent + Vue 3.5 + Tauri 2 app`.

This is not a traditional remote desktop product. It is an AI-session-first remote development control system. The relay provides the control-plane API and shared state, the agent runs on the target machine to execute AI sessions plus advanced diagnostics, and the control UI connects through Web, a Tauri desktop shell, or the Android shell.

## Status

- Positioning: personal-edition MVP / open source experimental project
- Primary flow: device selection, AI session launch, event-stream supervision, plus advanced terminal and preview tools
- Technical direction: Rust for protocol, backend, and agent; Vue + Tauri for the control client
- Mobile status: Android arm64 APK / AAB packaging is working, iOS is still pending
- Best-fit use cases: self-hosted personal AI operations console, multi-device control plane, cross-platform experimentation

## Features

- Rust workspace with shared protocol, backend, agent, and desktop app
- `vibe-relay` for Axum APIs, device state, AI-session scheduling, and terminal/preview control-plane flows
- `vibe-agent` for registration, polling, provider adapters, workspace-root execution, and advanced terminal/tunnel runtime
- `vibe-app` for the Vue 3.5 control UI, now centered on an AI session workspace, with `src-tauri` as the desktop and Android shell
- Provider integration for `Codex`, `Claude Code`, and `OpenCode`
- Relay-first AI session, terminal, and TCP preview/forwarding paths
- EasyTier-based overlay-assisted transport
- Tauri Android arm64 debug APK, release APK, and AAB builds
- SSE / WebSocket / tunnel based real-time updates

## Architecture

```text
┌──────────────────────────────────────────────────────────┐
│                     Control App                          │
│           Vue 3.5 Web UI / Tauri Desktop Shell          │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP / SSE / WebSocket
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-relay                          │
│   device registry · task control · shell · port proxy   │
│   auth · persistence · overlay-aware transport choice    │
└───────────────────────────┬──────────────────────────────┘
                            │ HTTP polling / bridge / tunnel
┌───────────────────────────▼──────────────────────────────┐
│                      vibe-agent                          │
│ provider adapters · task runtime · shell runtime         │
│ port-forward runtime · embedded overlay node             │
└───────────────────────────┬──────────────────────────────┘
                            │ local process / local TCP
                    ┌───────▼────────┐
                    │ target machine │
                    └────────────────┘
```

## Repository Layout

```text
.
├── apps
│   ├── vibe-relay        # Relay API / control plane
│   ├── vibe-agent        # Device agent / runtimes / providers
│   └── vibe-app          # Vue control app
│       └── src-tauri     # Tauri desktop shell
│           └── gen/android  # Generated Tauri Android project
├── crates
│   └── vibe-core         # Shared protocol / models
├── scripts               # Smoke tests and helper scripts
└── TESTING.md            # Testing strategy and validation matrix
```

## Quick Start

### Prerequisites

- Rust stable toolchain
- Node.js 20+
- `protobuf-compiler` or another working `protoc`
- WebKitGTK / GTK development packages when building Tauri on Linux
- Android builds require JDK 17, Android SDK cmdline-tools, plus `platforms;android-36`, `build-tools;35.0.0`, and `ndk;25.2.9519653`
- On Windows, install Npcap with WinPcap API-compatible mode enabled if you want EasyTier / overlay networking features
- At least one provider CLI installed locally if you want to execute AI tasks
  - `codex`
  - `claude`
  - `opencode`

### 1. Clone the repository

```bash
git clone https://github.com/fage-ac-org/vibe-everywhere.git
cd vibe-everywhere
```

### 2. Start the relay

```bash
cargo run -p vibe-relay
```

The default address is `http://127.0.0.1:8787`.

To enable single-user access control:

```bash
export VIBE_RELAY_ACCESS_TOKEN=change-me
```

### 3. Start the agent

```bash
cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787
```

If no provider CLI is installed, the device will still register successfully, but AI task execution will be unavailable.

### 4. Start the Web control UI

```bash
cd apps/vibe-app
npm ci
npm run dev
```

Default UI address:

- `http://127.0.0.1:1420`

If the relay requires an access token, you can enter it in the UI or set:

```bash
export VITE_RELAY_BASE_URL=http://127.0.0.1:8787
export VITE_RELAY_ACCESS_TOKEN=change-me
```

### 5. Start the desktop shell

```bash
cd apps/vibe-app
npm ci
npm run tauri dev
```

The Tauri shell reads:

- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`

### 6. Build an Android test package

If you want to control your server from an Android phone, you can build the Tauri Android app directly:

```bash
rustup target add aarch64-linux-android

export JAVA_HOME=/path/to/jdk-17
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_SDK_ROOT=$ANDROID_HOME
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
export ANDROID_NDK_HOME=$NDK_HOME

cd apps/vibe-app
npm ci
npm run android:doctor
npm run android:build:debug:apk
```

Default debug APK output:

- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`

For release artifacts:

```bash
cd apps/vibe-app
npm run android:build:apk
npm run android:build:aab
```

Output paths:

- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`
- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`

To sign the release APK / AAB during the build, add
`apps/vibe-app/src-tauri/gen/android/app/keystore.properties`
or export the signing values as environment variables:

```properties
storeFile=/absolute/path/to/vibe-everywhere-release.jks
storePassword=your-store-password
keyAlias=vibe-everywhere
keyPassword=your-key-password
```

The following environment variables are supported and override
`keystore.properties`:

- `VIBE_ANDROID_KEYSTORE_PATH`
- `VIBE_ANDROID_KEYSTORE_PASSWORD`
- `VIBE_ANDROID_KEY_ALIAS`
- `VIBE_ANDROID_KEY_PASSWORD`

With signing configured, run:

```bash
cd apps/vibe-app
npm run android:build:apk
npm run android:build:aab
```

The signed release APK will be written to:

- `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release.apk`

If you want GitHub Actions to sign Android release artifacts automatically,
configure these four repository or organization Actions secrets:

- `VIBE_ANDROID_KEYSTORE_BASE64`
- `VIBE_ANDROID_KEYSTORE_PASSWORD`
- `VIBE_ANDROID_KEY_ALIAS`
- `VIBE_ANDROID_KEY_PASSWORD`

`VIBE_ANDROID_KEYSTORE_BASE64` must contain the Base64-encoded keystore file.
For example:

```bash
base64 -w 0 /absolute/path/to/vibe-everywhere-release.jks
```

On macOS, if `base64` does not support `-w`, use:

```bash
base64 < /absolute/path/to/vibe-everywhere-release.jks | tr -d '\n'
```

Once these secrets are present, the release workflow decodes the keystore into
the runner temp directory and injects the `VIBE_ANDROID_*` signing variables for
the Android build. If the secrets are absent, the workflow still succeeds, but
the release APK remains unsigned.

Notes:

- Android and iOS control clients no longer prefill `127.0.0.1:8787`; on first launch, enter the relay machine's LAN IP or an HTTPS public URL unless you explicitly set `VIBE_PUBLIC_RELAY_BASE_URL`
- On the phone, point the relay URL to `http://<server-lan-ip>:8787` or a public HTTPS relay URL, not `http://127.0.0.1:8787`
- The Android app currently allows cleartext HTTP traffic for self-hosted LAN relays; use HTTPS for public deployments
- If `tauri android build` fails because `source.properties` is missing from an NDK directory, the SDK contains a partial NDK install. Run `npm run android:doctor`, then reinstall that NDK or explicitly export `NDK_HOME`
- Never commit `apps/vibe-app/src-tauri/gen/android/app/keystore.properties` or any `.jks` / `.keystore` files

### 7. Verify the stack

After the steps above, you should be able to:

- connect the UI to the relay
- see the agent in the device list
- create and execute tasks if a provider CLI is available
- open shell sessions
- create TCP port forwards

## Development

```bash
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm ci && npm run build
```

Common local entrypoints:

```bash
cargo run -p vibe-relay
cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787
cd apps/vibe-app && npm run dev
cd apps/vibe-app && npm run tauri dev
cd apps/vibe-app && npm run android:build:debug:apk
cd apps/vibe-app && npm run android:build:apk
cd apps/vibe-app && npm run android:build:aab
```

## Testing

See [TESTING.md](./TESTING.md) for the full test strategy.

Recommended local baseline:

```bash
cargo fmt --all --check
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm ci && npm run build
./scripts/dual-process-smoke.sh relay_polling
```

For Android changes, also run:

```bash
cd apps/vibe-app && npm run android:build:debug:apk
cd apps/vibe-app && npm run android:build:apk
cd apps/vibe-app && npm run android:build:aab
```

For overlay, EasyTier, shell, and forwarding transport changes, also run:

```bash
./scripts/dual-process-smoke.sh overlay
```

## GitHub Actions

The repository includes two workflows:

- `CI`
  - Triggers on `push` to `main`, `pull_request`, and manual dispatch
  - Runs formatting checks, workspace builds, workspace tests, frontend build, `relay_polling` smoke tests, Windows Rust/Tauri MSI bundling validation, and Android debug APK builds with artifact upload
- `Release`
  - Triggers on `v*` tags
  - Runs full verification, best-effort `overlay` smoke tests, Linux and Windows CLI packaging, Linux and Windows Tauri desktop packaging, Android debug APK / release APK / AAB packaging, and GitHub Release asset publishing

Release example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

Expected release assets include:

- `vibe-everywhere-cli-x86_64-unknown-linux-gnu.tar.gz`
- `vibe-everywhere-desktop-x86_64-unknown-linux-gnu.tar.gz`
- `vibe-everywhere-cli-x86_64-pc-windows-msvc.zip`
- `vibe-everywhere-desktop-x86_64-pc-windows-msvc.zip`
- `vibe-everywhere-android-arm64-debug.apk`
- `vibe-everywhere-android-arm64-release-unsigned.apk`
- `vibe-everywhere-android-arm64-release.aab`
- `SHA256SUMS.txt`

Notes:

- The repository does not yet ship Android release signing keys, so the release APK is currently `unsigned`
- Use the debug APK when you need an immediately installable test build

## Common Environment Variables

### relay

- `VIBE_RELAY_HOST`
- `VIBE_RELAY_PORT`
- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`
- `VIBE_RELAY_STATE_FILE`
- `VIBE_RELAY_FORWARD_HOST`
- `VIBE_RELAY_FORWARD_BIND_HOST`

### agent

- `VIBE_RELAY_URL`
- `VIBE_RELAY_ACCESS_TOKEN`
- `VIBE_DEVICE_NAME`
- `VIBE_DEVICE_ID`
- `VIBE_WORKING_ROOT`
- `VIBE_CODEX_COMMAND`
- `VIBE_CLAUDE_COMMAND`
- `VIBE_OPENCODE_COMMAND`

### overlay

- `VIBE_EASYTIER_RELAY_ENABLED`
- `VIBE_EASYTIER_NETWORK_NAME`
- `VIBE_EASYTIER_NETWORK_SECRET`
- `VIBE_EASYTIER_BOOTSTRAP_URL`
- `VIBE_EASYTIER_LISTENERS`

### frontend / desktop

- `VITE_RELAY_BASE_URL`
- `VITE_RELAY_ACCESS_TOKEN`
- `VIBE_PUBLIC_RELAY_BASE_URL`
- `VIBE_RELAY_ACCESS_TOKEN`

### android signing

- `VIBE_ANDROID_KEYSTORE_PATH`
- `VIBE_ANDROID_KEYSTORE_PASSWORD`
- `VIBE_ANDROID_KEY_ALIAS`
- `VIBE_ANDROID_KEY_PASSWORD`
- `VIBE_ANDROID_KEYSTORE_BASE64` (GitHub Actions secret only)

## Roadmap

- stronger authentication, auditing, and production deployment support
- frontend automated tests and protocol round-trip tests
- iOS packaging and broader mobile release automation
- continued extraction of large `main.rs` responsibilities into stable modules
- richer file sync, workspace browsing, and notification capabilities
- better desktop and mobile UX

## Contributing

Issues and pull requests are welcome.

Please include:

- the problem statement and change goal
- the affected crate / app
- the validation commands you ran
- screenshots for UI changes
- any new environment variables or system dependencies

Conventional Commits are recommended, for example:

```text
feat(agent): add claude stream-json mapping
fix(relay): keep overlay transport fallback stable
docs(readme): rewrite project overview and quick start
```

## License

This project is licensed under the [MIT License](./LICENSE).
