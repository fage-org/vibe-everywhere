# Development Guide

Last updated: 2026-03-28

This file is the developer entry point for building, testing, and changing the repository from
source. The top-level `README.md` and `README.en.md` stay focused on users and operators.

## Repository Layout

```text
.
├── apps
│   ├── vibe-relay        # Relay API / control plane
│   ├── vibe-agent        # Device agent / runtimes / providers
│   └── vibe-app          # Vue control app
│       └── src-tauri     # Tauri desktop shell + Android shell
├── crates
│   └── vibe-core         # Shared protocol / models
├── docs
│   ├── plans            # Versioned iteration / remediation plans
│   └── releases         # Versioned release notes and next-release draft
├── scripts               # Installers, smoke tests, release helpers
├── AGENTS.md             # Long-term repository guardrails
└── TESTING.md            # Test strategy and regression checklist
```

## Local Prerequisites

- Rust stable toolchain
- Node.js 24.14.x
- `protobuf-compiler` or another working `protoc`
- WebKitGTK / GTK development packages when building Tauri on Linux
- JDK 17, Android SDK cmdline-tools, `platforms;android-36`, `build-tools;35.0.0`, and
  `ndk;25.2.9519653` for Android builds
- Npcap with WinPcap API-compatible mode enabled on Windows if you need EasyTier / overlay support

## Common Development Commands

Start the relay:

```bash
cargo run -p vibe-relay
```

Start an agent:

```bash
cargo run -p vibe-agent -- --relay-url http://127.0.0.1:8787
```

Start the Web UI:

```bash
cd apps/vibe-app
npm ci
npm run dev
```

Start the desktop shell:

```bash
cd apps/vibe-app
npm ci
npm run tauri dev
```

Build the frontend:

```bash
cd apps/vibe-app
npm ci
npm run build
```

## Validation Commands

Workspace compile and test:

```bash
cargo fmt --all --check
cargo check --locked -p vibe-relay -p vibe-agent -p vibe-app
cargo test --locked --workspace --all-targets -- --nocapture
```

Smoke tests:

```bash
./scripts/dual-process-smoke.sh relay_polling
./scripts/dual-process-smoke.sh overlay
```

Frontend build:

```bash
cd apps/vibe-app
npm run build
```

## Android Builds

Debug APK:

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

Release APK / AAB:

```bash
cd apps/vibe-app
npm run android:build:apk
npm run android:build:aab
```

If you need signed Android release builds, provide the keystore inputs expected by the release
workflow or use `apps/vibe-app/src-tauri/gen/android/app/keystore.properties`.

## Release And Planning References

- planning index: [docs/plans/README.md](./docs/plans/README.md)
- planning process: [docs/plans/process.md](./docs/plans/process.md)
- release notes workflow: [docs/releases/README.md](./docs/releases/README.md)
- repository guardrails: [AGENTS.md](./AGENTS.md)
- testing checklist: [TESTING.md](./TESTING.md)

