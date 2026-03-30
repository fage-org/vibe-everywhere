# Vibe Everywhere Testing Strategy

## Goals

This repository has four independently changing surfaces:

- `crates/vibe-core`: shared protocol, enums, records, transport state
- `apps/vibe-relay`: relay control plane, persistence, auth, task/shell/port-forward routing
- `apps/vibe-agent`: device runtime, provider adapters, bridge transports, overlay integration
- `apps/vibe-app` and `apps/vibe-app/src-tauri`: Vue control UI and Tauri shell

The test plan must catch:

- protocol drift between relay, agent, and app
- conversation/task/shell/port-forward state regression
- provider event mapping regression
- relay-first and overlay transport regression
- frontend contract drift against relay APIs
- navigation or visibility regression against the shipped
  `首页 / 项目 / 通知 / 我的` and `会话 / 变更 / 文件 / 日志` model
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
./scripts/verify-release-version.sh v0.0.0
bash -n scripts/install-relay.sh
cargo build --locked --release --target x86_64-unknown-linux-musl -p vibe-relay -p vibe-agent
file target/x86_64-unknown-linux-musl/release/vibe-relay
file target/x86_64-unknown-linux-musl/release/vibe-agent
./scripts/render-release-notes.sh v0.0.0 >/dev/null
```

```powershell
pwsh -NoProfile -Command "[void][System.Management.Automation.Language.Parser]::ParseFile('scripts/install-relay.ps1',[ref]`$null,[ref]`$null)"
```

Manual review:

- confirm `./scripts/verify-release-version.sh vX.Y.Z` succeeds for the intended release tag and
  that the resolved version matches `Cargo.toml`, `apps/vibe-app/package.json`,
  `apps/vibe-app/package-lock.json`, and `apps/vibe-app/src-tauri/tauri.conf.json`
- confirm release asset names in `.github/workflows/release.yml` include the tag/version
- confirm the default Linux CLI release asset is `vibe-everywhere-cli-<tag>-x86_64-unknown-linux-musl.tar.gz`
  instead of a hosted-runner-coupled `x86_64-unknown-linux-gnu` archive
- confirm Linux CLI packaging copies binaries from `target/x86_64-unknown-linux-musl/release/`
  and that `file` reports the published Linux CLI binaries as `static-pie linked` or otherwise
  static without `NEEDED` dynamic-library entries
- confirm release packaging no longer copies repository README files into published artifacts
- confirm Windows CLI packaging keeps `vibe-relay(.exe)` / `vibe-agent(.exe)` and the EasyTier
  runtime files (`Packet.dll`, `wintun.dll`, `WinDivert64.sys`, optional `WinDivert.dll`)
  side-by-side in the staged archive layout
- confirm `scripts/install-relay.sh` and `scripts/install-relay.ps1` expose `install`, `update`,
  `uninstall`, and `help`, and remain limited to binary lifecycle operations rather than startup
  automation
- confirm both install scripts can manage `vibe-relay` and `vibe-agent`, and document the
  component-selection flags consistently
- confirm both install scripts still document and implement GitHub acceleration controls, including
  the default proxy prefix and the direct-access override flags
- confirm `scripts/install-relay.ps1` installs the Windows runtime files beside `vibe-relay.exe`
  and `vibe-agent.exe` instead of extracting only one executable
- confirm `uninstall` on Windows removes `vibe-relay.exe` and the packaged side-by-side runtime
  files together when no CLI binaries remain in the install directory
- confirm `docs/releases/unreleased.md` or the target `docs/releases/vX.Y.Z.md` exists and matches
  the shipped work
- confirm `README.md` and `README.en.md` stay user/operator-facing and do not absorb developer or
  governance-only content
- confirm both top-level README files still include a simple deployment path, a clear auth-token
  explanation, and a detailed usage flow that matches the shipped product behavior
- confirm user-facing runtime configuration tables and examples use the actual supported `VIBE_*`
  environment-variable names instead of obsolete relay-script aliases
- confirm the README deployment sections clearly distinguish relay bind host/port from
  `VIBE_PUBLIC_RELAY_BASE_URL` and document the overlay-related fixed listener ports that appear
  only when EasyTier mode is enabled
- confirm deployment docs describe CLI binary installation separately from relay startup and
  point to the dedicated startup guides
- confirm the top-level README files use a technical-document style suitable for end users and
  operators, avoiding promotional or overly conversational copy in deployment and usage sections
- confirm the top-level README files do not present `DEVELOPMENT.md`, `TESTING.md`, `AGENTS.md`,
  `PLAN.md`, or versioned planning docs as primary documentation entry points
- confirm the README download sections distinguish published artifacts from any source-built-only
  client surfaces instead of implying unsupported release packaging
- confirm `DEVELOPMENT.md` contains the current developer entry path when build or contributor
  instructions change

Pass criteria:

- release version sources stay synchronized and match the intended release tag
- installer scripts parse successfully
- the Linux CLI release target builds successfully for `x86_64-unknown-linux-musl`
- the Linux CLI binaries report a static executable format instead of dynamic `glibc` linkage
- the release notes renderer succeeds against the repository note source
- release asset naming and note-source rules remain aligned with the workflow

Recommended frequency:

- before cutting any release tag
- whenever release workflow, README onboarding, `DEVELOPMENT.md`, or deployment scripts change

## Manual Product Regression Checklist

Run this checklist whenever UI semantics, navigation, visibility gating, relay configuration
behavior, or project-workspace layout changes.

### Home / Host / Project Entry

- configure relay URL and access token from the in-app settings page
- confirm the default mobile navigation shows `首页 / 项目 / 通知 / 我的`
- confirm 首页 shows current host and project attention items instead of a legacy dashboard hero
- confirm 项目页 lists hosts and projects and can open a project workspace
- confirm a host can surface projects even when they have no prior conversation history, as long as
  the project is discoverable from the configured agent working root
- confirm previously discovered projects remain visible with a degraded state when the host goes
  offline or project refresh fails
- confirm project cards and the desktop tree show inventory availability state instead of silently
  dropping stale entries
- confirm offline and empty-state messaging is clear when no host or project is available

### Project Workspace

- open a project and confirm the header keeps host, project, branch, and AI state visible
- confirm the default secondary tab is `会话`
- confirm the project also exposes `变更 / 文件 / 日志`
- open an inventory-only project with no prior history and confirm the empty conversation state
  still allows sending the first prompt
- create a new conversation and continue an existing conversation in the same project
- switch execution mode between `只读 / 可改文件 / 可改并测试` and confirm the selected mode is
  visible in the conversation transcript metadata
- verify the composer shows an `有效约束 / Effective enforcement` summary that changes with the
  selected execution mode
- send a clearly risky writable prompt and confirm the extra confirmation step appears before send
- verify provider pending-input prompts can be answered inline with option chips and custom text
- verify latest task status changes appear in the project workspace after refresh or live updates
- verify conversation turns show a per-task summary, recent execution events, and expandable raw
  event output instead of only flat chat bubbles
- verify pending or running task cards can request stop directly from the conversation surface
- verify completed or failed task cards expose quick follow-up actions for retry and explanation
- verify completed tasks can jump directly to `变更`, and failed or completed tasks can jump
  directly to `日志`

### Review And Inspection

- verify `变更` shows Git summary information before raw detail
- verify `变更` shows review summary cards before the raw file diff panel
- verify selecting a changed file loads its staged and/or unstaged diff output
- verify `文件` can browse the project tree and preview a text file
- verify `日志` surfaces error summaries first and supports `全部 / 错误 / 工具 / Provider` filtering
- verify `日志` also shows audit records related to the active conversation tasks before the raw
  event stream
- when an ACP-backed task runs in `只读`, verify write attempts or terminal command attempts are
  rejected; when it runs in `可改文件`, verify test-style terminal commands are rejected until
  `可改并测试` is selected
- verify CLI-backed Codex tasks derive sandbox/approval flags from execution mode, and CLI-backed
  Claude read-only tasks enter native `plan` permission mode plus the default write/shell
  disallowed-tools set
- verify CLI-backed Claude workspace-write tasks include the default disallowed-tools blacklist for
  common test-style Bash commands
- verify CLI-backed Codex write-and-test tasks use the workspace-write sandbox plus `never`
  approval, and Claude write-and-test tasks do not inherit the write-mode test blacklist by
  default
- verify task cards show an `有效约束 / Effective enforcement` summary that matches ACP, Codex,
  Claude, or generic provider behavior for that task
- verify a failed task can be reopened from the relevant project context
- on desktop width, create a sibling worktree from the project workspace and confirm the new branch
  and path appear in the refreshed host/project tree, even when the worktree came from Git
  inventory expansion instead of only the original top-level scan; then reopen that worktree from
  the sidebar
- verify non-current worktrees can be removed from the desktop sidebar, the current worktree cannot
  be removed, and the refreshed tree drops the removed entry
- verify desktop worktree items surface lifecycle states such as current, detached, inventory
  missing, offline/unreachable, and remove-failed when those conditions occur

### Notifications And Settings

- confirm 通知页 lists failures, waiting-input items, or recent completions when present
- switch the default notification preference between `全部活动` and `仅失败与等待输入`, then confirm
  projects without overrides inherit that default
- switch a project between `全部活动` and `仅失败与等待输入`, then confirm completed-task
  notifications are hidden or shown accordingly
- reset a project override and confirm it falls back to the current default preference
- confirm 通知页 separates unread and recent items, and opening an unread item moves it out of the
  unread section on the next render
- confirm notification status filters (`全部 / 未读 / 失败 / 等待 / 已完成`) narrow the list correctly
- confirm notification entries navigate back to the intended project workspace and, when a
  conversation id is provided, restore that exact conversation instead of only opening the project
- confirm notification quick actions can directly open `会话 / 变更 / 日志`, and the target
  workspace tab matches the chosen action
- confirm 我的 contains relay settings, locale, theme, and other secondary preferences instead of
  primary project workflow content
- confirm 我的 lets the user change default execution mode, default notification preference, and the
  high-risk confirmation toggle
- confirm a newly opened conversation composer inherits the configured default execution mode
- confirm 我的 also includes a global audit trail with `全部 / 任务 / Shell 与预览` filtering
- confirm desktop-width layouts remain usable and do not degrade into one long mobile column when a
  workbench layout is intended

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
3. Verify the default route opens the chat home, the bottom navigation exposes `Chat`, `Devices`, and `Menu`, and the old `#/connections` or `#/sessions` deep links no longer define the primary workflow.
4. Verify relay URL and the control-plane access token can be applied from `Menu > Settings > Server`, then return to the chat home without losing the configured connection.
5. Verify language switching between English and Simplified Chinese updates visible section copy.
6. Verify light, dark, and system theme switching updates all primary sections without layout regressions.
7. Verify the chat home groups prior work by device and project folder, and that each device card can open its default workspace even when no history exists yet.
8. Verify governance / audit UI stays hidden by default; if the relevant feature flag is enabled, verify it appears intentionally rather than by default leakage.
9. Open an existing project, verify the chat page shows the expected project path, and use the top-left history control to switch between prior topics in the same project.
10. Verify the main transcript keeps only user / assistant dialogue and inline input requests; raw task lifecycle, tool output, and stderr events must stay out of the primary message flow.
11. Verify turns that only produced runtime activity render a lightweight Trace entry instead of dumping raw `Task queued` / `cwd=` / `Task finished` style events into the transcript.
12. Create a new topic inside a project, verify it inherits the same device and project folder, and send a follow-up prompt in the same topic to confirm provider-native continuation.
13. Trigger or simulate a provider input request, then verify option chips render inline and that the custom-text path can also be submitted.
14. Verify `Devices` shows inventory, runtime metadata, deployment metadata, current-client-only platform information, provider availability, and selected-device workload counts.
15. Open `Menu`, then `Server Settings`, and verify language/theme changes work without leaving stale visual state in the chat shell.
16. Create a shell session in the secondary tools surface, send input, verify timeline ordering, and close the session.
17. Create a preview / port forward in the secondary tools surface, verify status updates, relay endpoint display, and close flow.
18. Start the agent with `VIBE_RELAY_ENROLLMENT_TOKEN`, restart it once, and verify the same device reconnects successfully without placing the control-plane token on the agent host.
19. Confirm the agent working root now contains `.vibe-agent/identity.json` after first registration and that deleting it forces re-enrollment on the next start.
20. Verify narrow-width layout keeps the project chat primary, the history drawer remains usable, and the bottom navigation does not regress into a long all-in-one dashboard.

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
- desktop shell opens on the device/project chat home, project chats behave like a messaging view, and server settings live under the menu path instead of the primary transcript
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
4. Verify the chat home identifies the current client as `Android`, keeps device/project entry as the main surface, and lets the user configure relay access from the menu settings path without exposing other platforms as switchable choices.
5. Verify bottom navigation remains usable in portrait orientation.
6. Verify project selection, topic-history drawer access, conversation creation, follow-up replies, inline choice prompts, transcript noise suppression for raw runtime events, shell session output, and preview / port-forward flows from the phone.

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
  - expand the initial `vitest` coverage beyond `src/lib/policy.ts` and `src/lib/projectInventory.ts`
  - add coverage for `src/lib/api.ts`, `src/lib/runtime.ts`, `src/lib/i18n.ts`, and `src/lib/theme.ts`
  - mock `fetch`, `EventSource`, and `WebSocket` to test reconnect behavior, locale / theme persistence, and workspace / Git loading states
  - add focused coverage for platform/current-client detection semantics and project-workspace visibility rules across Web, Tauri Desktop, and Android
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
