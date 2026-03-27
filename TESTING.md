# Vibe Everywhere Testing Strategy

## Goals

This repository has four independently changing surfaces:

- `crates/vibe-core`: shared protocol, enums, records, transport state
- `apps/vibe-relay`: relay control plane, persistence, auth, task/shell/port-forward routing
- `apps/vibe-agent`: device runtime, provider adapters, bridge transports, overlay integration
- `apps/vibe-app` and `apps/vibe-app/src-tauri`: Vue control UI and Tauri shell

The test plan must catch:

- protocol drift between relay, agent, and app
- task/shell/port-forward state regression
- provider event mapping regression
- relay-first and overlay transport regression
- frontend contract drift against relay APIs
- Android mobile packaging and runtime wiring regression
- environment/configuration regression before release

## Test Layers

### Layer 0: Formatting, Compile, and Type Gates

Purpose:

- fail fast on broken code generation, missing imports, type drift, or frontend contract mismatches

Commands:

```bash
cargo fmt --all --check
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cd apps/vibe-app && npm run build
```

Pass criteria:

- Rust formatting is clean
- all workspace targets compile
- Vue TypeScript check and production build succeed

Primary modules covered:

- `apps/vibe-relay/src/*.rs`
- `apps/vibe-agent/src/*.rs`
- `apps/vibe-app/src/**/*.ts`
- `apps/vibe-app/src/**/*.vue`
- `apps/vibe-app/src-tauri/src/*.rs`
- `crates/vibe-core/src/lib.rs`

Recommended frequency:

- every local change before handoff
- every PR in CI

### Layer 0.5: Windows Compatibility Gate

Purpose:

- catch Windows-only linker/resource regressions before release
- verify `pnet` can resolve `Packet.lib` under the MSVC toolchain
- verify the Tauri shell still compiles when Windows-specific assets such as `icon.ico` are required

Execution in CI:

```powershell
cargo check --locked -p vibe-relay -p vibe-agent
cd apps/vibe-app
npm run tauri -- build --debug --bundles msi --no-sign --ci
```

Environment notes:

- inject the Npcap SDK `Lib/x64` directory into `%LIB%` before building Rust binaries
- use MSI bundling in CI so Windows-only resource requirements such as `.ico` icons fail before release

Pass criteria:

- `vibe-relay` and `vibe-agent` compile on `x86_64-pc-windows-msvc`
- Tauri desktop MSI bundling succeeds on Windows without missing icon/resource errors
- the frontend build invoked by `tauri build` succeeds

Recommended frequency:

- every PR in CI
- before cutting any release tag intended to ship Windows assets

### Layer 0.75: Android Packaging Gate

Purpose:

- catch Android-only build regressions before release
- verify the generated Tauri Android project still matches the configured package identifier
- verify the Rust mobile library, Gradle packaging, and Tauri Android bridge stay aligned

Execution commands:

```bash
cd apps/vibe-app
npm run android:doctor
npm run android:build:debug:apk
npm run android:build:apk
npm run android:build:aab
```

Environment notes:

- install JDK 17
- install Android SDK cmdline-tools and accept licenses
- install `platform-tools`, `platforms;android-36`, `build-tools;35.0.0`, and `ndk;25.2.9519653`
- export `ANDROID_HOME`, `ANDROID_SDK_ROOT`, `NDK_HOME`, and `ANDROID_NDK_HOME`
- run `npm run android:doctor` before packaging so partial SDK or NDK installs fail fast with a clear diagnosis
- install the Rust target with `rustup target add aarch64-linux-android`

Pass criteria:

- debug APK is produced at `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`
- release APK is produced at `apps/vibe-app/src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`
- release AAB is produced at `apps/vibe-app/src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`
- APK metadata reports package name `org.fageac.vibeeverywhere`

Recommended frequency:

- run the debug APK build on every PR in CI
- run all three commands before any release tag intended to ship Android artifacts

### Layer 1: Rust Unit and Contract Tests

Purpose:

- verify pure logic, protocol mapping, filtering, transport selection, and state transitions without external processes

Existing high-value coverage:

- `apps/vibe-agent/src/main.rs`
  - device registration request construction
  - relay websocket URL rewriting
  - Codex and Claude structured event mapping
  - default advertised capability boundary
- `apps/vibe-agent/src/easytier.rs`
  - listener parsing
  - private network config preservation
- `apps/vibe-agent/src/task_bridge.rs`
  - CLI task bridge auth and event streaming
- `apps/vibe-agent/src/port_forward_bridge.rs`
  - tunnel auth and target traffic proxying
- `apps/vibe-relay/src/main.rs`
  - task create/claim/cancel/state transitions
  - shell create/claim/detail/filter behavior
  - port-forward create/claim/report/close behavior
  - overlay preference and fallback behavior
  - SSE event streaming behavior
- `apps/vibe-relay/src/easytier.rs`
  - relay overlay config defaults and private-mode behavior

Execution command:

```bash
cargo test --workspace --all-targets -- --nocapture
```

Pass criteria:

- all Rust unit and integration-style tests pass
- no ignored failure paths or intermittent transport regressions

Recommended frequency:

- every PR in CI
- before any release tag

### Layer 2: Dual-Process End-to-End Smoke Tests

Purpose:

- validate real relay and agent binaries together, instead of only in-process test helpers
- exercise HTTP registration, task dispatch, streaming, shell, and TCP forwarding across actual OS processes

Existing smoke entrypoint:

- `scripts/dual-process-smoke.sh relay_polling`
- `scripts/dual-process-smoke.sh overlay`

`relay_polling` mode validates:

- relay boot and health endpoint
- agent registration and heartbeat
- provider availability detection
- task creation and completion through relay polling
- relay-tunnel port-forward creation, activation, traffic forwarding, and close

`overlay` mode validates:

- embedded EasyTier bootstrap and agent connectivity
- overlay task dispatch
- overlay shell session activation, input, output, and completion
- overlay port-forward activation, byte forwarding, and close

Execution commands:

```bash
./scripts/dual-process-smoke.sh relay_polling
./scripts/dual-process-smoke.sh overlay
```

Pass criteria:

- both modes exit `0`
- task status reaches `succeeded`
- expected transport is used instead of silent fallback
- shell output contains the smoke marker
- TCP port-forward reply matches the expected payload

Recommended frequency:

- run `relay_polling` on every PR in CI
- run `overlay` on release branches, nightly CI, or before shipping networking changes

### Layer 3: Frontend Manual Regression

Purpose:

- the Vue app currently has no automated unit or browser test harness, so user-visible control flows need a defined regression checklist

Manual checklist:

1. Launch relay and agent locally.
2. Launch `apps/vibe-app` with `npm run dev`.
3. Verify relay URL and optional token can be applied from the dashboard.
4. Verify health counts and device list load correctly.
5. Create a task, observe live event updates, and verify detail rendering.
6. Cancel a running task and verify terminal status rendering.
7. Create a shell session, send input, verify timeline ordering, and close the session.
8. Create a port forward, verify status updates, relay endpoint display, and close flow.
9. Toggle task, shell, and port-forward filters to verify selected-device scoping.
10. Verify mobile-width layout for the dashboard.

Recommended frequency:

- before UI releases
- after changing `apps/vibe-app/src/lib/api.ts`
- after changing `apps/vibe-app/src/stores/control.ts`
- after changing `apps/vibe-app/src/views/DashboardView.vue`

### Layer 4: Desktop Shell and Environment Validation

Purpose:

- catch issues that only appear in the Tauri shell or in environment-variable wiring

Execution:

```bash
cargo check -p vibe-app
cd apps/vibe-app && npm run tauri dev
```

Manual checks:

- app shell boots with default relay config
- `VIBE_PUBLIC_RELAY_BASE_URL` and `VIBE_RELAY_ACCESS_TOKEN` are picked up correctly
- desktop shell can connect to a live relay and render the dashboard

Recommended frequency:

- before desktop releases
- after changing `apps/vibe-app/src-tauri`

### Layer 4.5: Android Device Validation

Purpose:

- catch issues that only appear on a real phone, especially relay URL configuration and mobile viewport behavior

Manual checks:

1. Install the debug APK on a physical Android device.
2. Configure the relay URL with `http://<server-lan-ip>:8787` or a public HTTPS URL, not `http://127.0.0.1:8787`.
3. Verify the dashboard loads and device counts render correctly.
4. Verify task creation, live updates, shell session output, and port-forward flows from the phone.
5. Verify the mobile layout remains usable in portrait orientation.

Recommended frequency:

- before shipping Android artifacts
- after changing `apps/vibe-app/src/views/DashboardView.vue`
- after changing `apps/vibe-app/src/lib/api.ts`
- after changing `apps/vibe-app/src-tauri`

## Release Validation Matrix

### Minimum PR Gate

```bash
cargo fmt --all --check
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm run build
./scripts/dual-process-smoke.sh relay_polling
```

For PRs that touch Android packaging or mobile shell code, add:

```bash
cd apps/vibe-app && npm run android:build:debug:apk
```

### Networking or Transport Change Gate

Run the minimum PR gate, then add:

```bash
./scripts/dual-process-smoke.sh overlay
```

Apply this extended gate when changing:

- `apps/vibe-relay/src/tasks.rs`
- `apps/vibe-relay/src/shell.rs`
- `apps/vibe-relay/src/port_forwards.rs`
- `apps/vibe-relay/src/easytier.rs`
- `apps/vibe-agent/src/task_runtime.rs`
- `apps/vibe-agent/src/shell_runtime.rs`
- `apps/vibe-agent/src/port_forward_runtime.rs`
- `apps/vibe-agent/src/task_bridge.rs`
- `apps/vibe-agent/src/shell_bridge.rs`
- `apps/vibe-agent/src/port_forward_bridge.rs`
- `apps/vibe-agent/src/easytier.rs`

### Android Release Gate

Run the minimum PR gate, then add:

```bash
cd apps/vibe-app && npm run android:build:debug:apk
cd apps/vibe-app && npm run android:build:apk
cd apps/vibe-app && npm run android:build:aab
```

Complete these with the Android device manual checks when shipping mobile artifacts.

Apply this gate when changing:

- `apps/vibe-app/src-tauri/tauri.conf.json`
- `apps/vibe-app/src-tauri/gen/android/**`
- `apps/vibe-app/src-tauri/src/lib.rs`
- `apps/vibe-app/package.json`

### Release Gate

Run the networking or transport change gate, then complete:

- frontend manual regression checklist
- Tauri shell manual validation
- Android device manual validation when mobile artifacts are included
- environment-variable sanity checks for tokenized relay access
- GitHub Actions release packaging for Linux, Windows, and Android

## Current Coverage Gaps

These areas should be added next if the goal is a more complete automated test suite:

1. `crates/vibe-core/src/lib.rs`
   - add serde round-trip tests for protocol enums and records
   - add behavior tests for timestamp helpers and terminal-state helpers
2. `apps/vibe-agent/src/providers.rs`
   - add direct tests for provider command construction
   - add fallback tests for malformed JSON lines and provider stderr/stdout mapping
3. `apps/vibe-agent/src/config.rs`
   - add env parsing tests for polling, heartbeat, working root, and command overrides
4. `apps/vibe-relay/src/auth.rs`, `apps/vibe-relay/src/config.rs`, `apps/vibe-relay/src/store.rs`
   - add focused unit tests for token extraction, config defaults, and state-file path handling
5. `apps/vibe-app`
   - introduce `vitest` and cover `src/lib/api.ts`, `src/lib/runtime.ts`, and `src/stores/control.ts`
   - mock `fetch`, `EventSource`, and `WebSocket` to test reconnect and filter behavior
6. cross-platform runtime validation
   - Linux remains the most complete smoke-test baseline
   - Windows now has dedicated compile/package validation in CI and Release, but still lacks runtime smoke coverage
   - Android now has dedicated APK/AAB packaging validation, but still lacks emulator or device-level automated smoke coverage
   - macOS shell behavior and packaging still need dedicated validation

## Execution Record Template

When validating a change, record:

- commit or workspace state
- commands executed
- pass/fail result
- whether smoke ran in `relay_polling`, `overlay`, or both modes
- whether frontend manual regression was completed
- whether Tauri manual validation was completed

## Latest Baseline

Date:

- `2026-03-27`

Workspace state:

- Android mobile support changes applied in the workspace before execution

Executed commands:

```bash
cargo fmt --all --check
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm run build
./scripts/dual-process-smoke.sh relay_polling
./scripts/dual-process-smoke.sh overlay
cd apps/vibe-app && npm run android:build:debug:apk
cd apps/vibe-app && npm run android:build:apk
cd apps/vibe-app && npm run android:build:aab
```

Result:

- all commands passed
- `relay_polling` smoke passed with successful task execution and relay-tunnel port forwarding
- `overlay` smoke passed with successful task execution, shell session I/O, and overlay port forwarding
- Android debug APK, release APK, and release AAB builds passed
- frontend manual regression not executed in this run
- Tauri GUI manual validation not executed in this run
