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
- release packaging drift, missing release notes, or broken operator bootstrap scripts

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

### Layer 0.5: Windows Compatibility And Smoke Gate

Purpose:

- catch Windows-only linker/resource regressions before release
- verify `pnet` can resolve `Packet.lib` under the MSVC toolchain
- verify the Tauri shell still compiles when Windows-specific assets such as `icon.ico` are required
- verify a real Windows relay-plus-agent relay-polling path still works outside in-process unit
  tests

Execution in CI:

```powershell
cargo check --locked -p vibe-relay -p vibe-agent
./scripts/dual-process-smoke.ps1 relay_polling
cd apps/vibe-app
npm run tauri -- build --debug --bundles msi --no-sign --ci
```

Environment notes:

- inject the Npcap SDK `Lib/x64` directory into `%LIB%` before building Rust binaries
- use MSI bundling in CI so Windows-only resource requirements such as `.ico` icons fail before release

Pass criteria:

- `vibe-relay` and `vibe-agent` compile on `x86_64-pc-windows-msvc`
- a Windows runner can start real relay and agent processes, register the device, complete a task,
  and complete a relay-polling shell smoke path
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

### Layer 0.9: Release Packaging And Operator Onboarding Gate

Purpose:

- catch release-asset naming regressions before publishing
- ensure repository-owned release notes exist for the next or current release
- verify bootstrap installers remain syntactically valid after deployment-doc changes
- ensure top-level README files remain user-facing while developer entry points stay in
  `DEVELOPMENT.md`

Execution checks:

```bash
bash -n scripts/install-relay.sh
./scripts/render-release-notes.sh v0.0.0 >/dev/null
```

```powershell
pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/install-relay.ps1',[ref]`$null,[ref]`$null)"
```

Manual review:

- confirm release asset names in `.github/workflows/release.yml` include the tag/version
- confirm release packaging no longer copies repository README files into published artifacts
- confirm Windows CLI packaging keeps `vibe-relay(.exe)` / `vibe-agent(.exe)` and the EasyTier
  runtime files (`Packet.dll`, `wintun.dll`, `WinDivert64.sys`, optional `WinDivert.dll`)
  side-by-side in the staged archive layout
- confirm `scripts/install-relay.ps1` installs the Windows runtime files beside `vibe-relay.exe`
  instead of extracting only the executable
- confirm `docs/releases/unreleased.md` or the target `docs/releases/vX.Y.Z.md` exists and matches
  the shipped work
- confirm `README.md` and `README.en.md` stay user/operator-facing and do not absorb developer or
  governance-only content
- confirm the top-level README files do not present `DEVELOPMENT.md`, `TESTING.md`, `AGENTS.md`,
  `PLAN.md`, or versioned planning docs as primary documentation entry points
- confirm `DEVELOPMENT.md` contains the current developer entry path when build or contributor
  instructions change

Pass criteria:

- installer scripts parse successfully
- the release notes renderer succeeds against the repository note source
- release asset naming and note-source rules remain aligned with the workflow

Recommended frequency:

- before cutting any release tag
- whenever release workflow, README onboarding, `DEVELOPMENT.md`, or deployment scripts change

### Layer 1: Rust Unit and Contract Tests

Purpose:

- verify pure logic, protocol mapping, filtering, transport selection, and state transitions without external processes

Existing high-value coverage:

- `apps/vibe-agent/src/main.rs`
  - device registration request construction
  - agent identity persistence and device-credential reuse
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
- `apps/vibe-agent/src/workspace_runtime.rs`
  - workspace path boundary enforcement
  - browse and preview behavior within the configured working root
- `apps/vibe-agent/src/git_runtime.rs`
  - Git status parsing
  - non-repository handling and changed-file / recent-commit collection
- `apps/vibe-relay/src/main.rs`
  - task create/claim/cancel/state transitions
  - control-plane versus device-credential route boundaries
  - workspace browse / preview request claim-complete orchestration
  - Git inspect request claim-complete orchestration
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
- `scripts/dual-process-smoke.ps1 relay_polling` on Windows CI

`relay_polling` mode validates:

- relay boot and health endpoint
- agent registration and heartbeat
- provider availability detection
- task creation and completion through relay polling
- relay-tunnel port-forward creation, activation, traffic forwarding, and close
- on Windows CI, task creation/completion and relay-polling shell execution through real Windows
  processes

`overlay` mode validates:

- embedded EasyTier bootstrap and agent connectivity
- overlay bridge listener reachability from the relay side before functional validation starts
- overlay task dispatch, including graceful fallback to relay polling when the task bridge is not
  ready yet
- overlay shell session activation, input, output, and completion, including graceful fallback to
  relay polling when the shell bridge is not ready yet
- overlay port-forward activation, byte forwarding, and close
- same-host CI harness stability through a test-only bootstrap host and explicit overlay node IP,
  a dedicated harness-only agent listener, faster harness-only EasyTier restart/poll cadence, and
  harness-only bridge recovery timers, without changing product/runtime defaults

GitHub-hosted runner note:

- local or self-controlled environments should still treat `./scripts/dual-process-smoke.sh
  overlay` as the meaningful full overlay smoke path
- GitHub-hosted Linux now runs `overlay` smoke as a blocking `Linux Overlay Smoke` job in both
  `CI` and `Release`
- that hosted job uses harness-only `VIBE_TEST_EASYTIER_NO_TUN=1`, so it verifies truthful
  overlay control-plane and fallback behavior on hosted runners without changing product defaults

Execution commands:

```bash
./scripts/dual-process-smoke.sh relay_polling
./scripts/dual-process-smoke.sh overlay
```

Pass criteria:

- both modes exit `0`
- task status reaches `succeeded`
- `relay_polling` mode keeps task traffic on relay polling
- full `overlay` mode allows task and shell traffic to fall back to relay polling, but the
  port-forward check must still exercise real overlay transport when TUN-backed overlay is
  available
- the hosted Linux no_tun overlay gate allows task and shell traffic to fall back to relay
  polling, requires preview/port-forward selection to fall back truthfully to relay tunnel, and
  does not claim hosted preview byte-path coverage that the environment cannot provide
- shell output contains the smoke marker
- TCP port-forward reply matches the expected payload

Task-bridge recovery preference remains covered by relay tests, for example
`overlay_task_bridge_probe_restores_overlay_preference_after_recovery` in
`apps/vibe-relay/src/main.rs`.

Recommended frequency:

- run both `relay_polling` and `overlay` on every PR in CI
- treat hosted Linux `overlay` as a blocking gate in both `CI` and `Release`, not a best-effort
  signal

### Layer 3: Frontend Manual Regression

Purpose:

- the Vue app currently has no automated unit or browser test harness, so user-visible control flows need a defined regression checklist

Manual checklist:

1. Launch relay and agent locally.
2. Launch `apps/vibe-app` with `npm run dev`.
3. Verify desktop uses sidebar navigation and mobile-width uses bottom navigation for `Sessions`, `Devices`, and `Advanced`, and that the legacy `#/connections` deep link redirects into `Sessions`.
4. Verify relay URL and the control-plane access token can be applied from the top of the `Sessions` primary workflow.
5. Verify language switching between English and Simplified Chinese updates visible section copy.
6. Verify light, dark, and system theme switching updates all primary sections without layout regressions.
7. Verify `Sessions` keeps the everyday workflow on one page: relay connection, device selection, session creation, and result review.
8. Verify governance / audit UI stays hidden by default; if the relevant feature flag is enabled, verify it appears intentionally rather than by default leakage.
9. Create a task in `Sessions`, observe live event updates, and verify the current-session review card renders prompt, status, and summary data.
10. Verify the result-review panel loads Git metadata, changed files, recent commits, and diff counters before terminal usage is required.
11. Verify the workspace browser loads, path navigation works, and file preview renders text content from the same `Sessions` workflow.
12. Verify `Devices` shows inventory, runtime metadata, deployment metadata, current-client-only platform information, provider availability, and selected-device workload counts.
13. Create a shell session in `Advanced`, send input, verify timeline ordering, and close the session.
14. Create a preview / port forward in `Advanced`, verify status updates, relay endpoint display, and close flow.
15. Start the agent with `VIBE_RELAY_ENROLLMENT_TOKEN`, restart it once, and verify the same device reconnects successfully without placing the control-plane token on the agent host.
16. Confirm the agent working root now contains `.vibe-agent/identity.json` after first registration and that deleting it forces re-enrollment on the next start.
17. Toggle task, shell, and port-forward filters to verify selected-device scoping.
18. Verify narrow-width layout no longer collapses back into one long all-in-one page.

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
- desktop shell can connect to a live relay and render the route-backed primary sections
- desktop shell `Sessions` view keeps relay config, device selection, session launch, and result review on the same primary surface
- desktop shell `Devices` view shows the current client as `Desktop` without listing other platforms as in-page choices

Recommended frequency:

- before desktop releases
- after changing `apps/vibe-app/src-tauri`

### Layer 4.5: Android Device Validation

Purpose:

- catch issues that only appear on a real phone, especially relay URL configuration and mobile viewport behavior

Manual checks:

1. Install the debug APK on a physical Android device.
2. Configure the relay URL with `http://<server-lan-ip>:8787` or a public HTTPS URL, not `http://127.0.0.1:8787`.
3. Verify the app does not prefill a loopback relay URL by default.
4. Verify the `Sessions` primary workflow identifies the current client as `Android` and lets the user configure relay access without exposing other platforms as switchable choices.
5. Verify bottom navigation remains usable in portrait orientation.
6. Verify task creation, live updates, result review, shell session output, and preview / port-forward flows from the phone.

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
VIBE_TEST_EASYTIER_NO_TUN=1 ./scripts/dual-process-smoke.sh overlay
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
  - confirm control clients use `VIBE_RELAY_ACCESS_TOKEN` while agents use `VIBE_RELAY_ENROLLMENT_TOKEN`
  - confirm deleting the persisted agent identity forces a fresh enrollment instead of silently reusing the human control token
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
   - add focused unit tests for control-token, enrollment-token, and device-credential extraction,
     config defaults, and state-file path handling
5. `apps/vibe-relay/src/workspace.rs` and `apps/vibe-relay/src/git.rs`
   - add dual-process smoke coverage for request timeout, agent completion, and API wiring through real relay / agent binaries
6. `apps/vibe-app`
  - introduce `vitest` and cover `src/lib/api.ts`, `src/lib/runtime.ts`, `src/lib/i18n.ts`, `src/lib/theme.ts`, and `src/stores/control.ts`
  - mock `fetch`, `EventSource`, and `WebSocket` to test reconnect behavior, locale / theme persistence, and workspace / Git loading states
   - add focused coverage for `src/lib/platform.ts`, feature-flag visibility rules, relay placeholder behavior, and current-client detection semantics across Web, Tauri Desktop, and Android
7. cross-platform runtime validation
   - Linux remains the most complete smoke-test baseline, while GitHub-hosted Linux gating uses the
     harness-only `no_tun` path instead of claiming direct overlay bridge or preview byte-path
     coverage that the runner does not provide
   - Windows now has dedicated relay-polling runtime smoke coverage in CI and Release, but still
     lacks hosted overlay smoke coverage
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

- `2026-03-29`

Workspace state:

- relay and agent auth-boundary hardening plus deployment-doc/test-plan updates applied in the
  workspace before execution

Executed commands:

```bash
cargo fmt --all
cargo check -p vibe-relay -p vibe-agent -p vibe-app
cargo test --workspace --all-targets -- --nocapture
cd apps/vibe-app && npm run build
bash -n scripts/install-relay.sh
./scripts/dual-process-smoke.sh relay_polling
./scripts/dual-process-smoke.sh overlay
```

Result:

- all commands passed
- `relay_polling` and `overlay` smoke both passed with split control/enrollment token coverage
- docs, installers, and manual verification now reflect the control-token plus enrollment-token auth model and persisted agent identity file
- frontend manual regression not executed in this run
- Tauri GUI manual validation not executed in this run
