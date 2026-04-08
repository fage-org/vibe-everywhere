# AI Execution Batches

## Purpose

This file converts `execution-plan.md` into direct AI dispatch batches.

Use this file when you want to assign work in grouped waves such as:

- one batch per day
- one batch per AI worker queue
- one batch per review cycle

## Batch Rules

- Batches are ordered. Do not skip ahead unless this file explicitly marks a later batch as safe.
- Inside a batch, still dispatch one module plan per implementation task.
- A batch is complete only when its gate is satisfied.
- If a batch discovers a missing contract, update the relevant shared or project plan before
  continuing.
- If two modules in the same batch touch the same write scope, run them serially even if the batch
  says “parallel allowed”.

## Batch Index

| Batch | Focus | Blocking Output |
| --- | --- | --- |
| `[done] B00` | planning freeze | all shared contracts and execution docs frozen |
| `[done] B01` | wire metadata and legacy/session schemas | `vibe-wire` core shape available |
| `[done] B02` | wire containers and voice | complete public `vibe-wire` surface |
| `[done] B03` | server config and storage spine | server can start and persist state |
| `[done] B04` | server auth, sessions, machines | minimum backend path exists |
| `[done] B05` | server presence, router, socket | live backend path exists |
| `[done] B06` | agent auth and HTTP control | agent can authenticate and call server |
| `[done] B07` | agent live control and CLI UX | remote-control path works end-to-end |
| `[done] B08` | server support APIs, files, and images | app/CLI support surface exists |
| `[done] B09` | server router/socket finalization and monitoring | server route/socket surface complete |
| `[done] B10` | CLI foundation | local CLI architecture exists |
| `[done] B11` | CLI daemon and local control plane | daemonized local control works |
| `[done] B12` | CLI first provider vertical slice | first provider runtime slice works end-to-end |
| `[done] B13` | CLI provider expansion | remaining providers and final command wiring land on stable core |
| `[done] B14` | app import baseline | imported app builds in this repo |
| `[done] B15` | app adaptation | app works against Vibe services |
| `[done] B16` | optional sidecar | app-log sidecar parity if still needed |
| `B17` | historical desktop-preview planning freeze and first usable slice | historical Wave 8 baseline for the desktop preview path |
| `B18` | historical desktop promotion planning | historical Wave 8 parity and promotion baseline |
| `B19` | unified runtime bootstrap | `vibe-app-tauri` can host desktop, Android Tauri-mobile, and retained static browser export in one package |
| `B20` | shared core import from Happy | replacement package owns reusable auth/sync/realtime/core modules |
| `B21` | shell surfaces, static browser export, and identity | desktop/Android/static-export `P0` entry flows and create/link/restore flows work |
| `B22` | session runtime and rendering | desktop and mobile reach a real end-to-end session chain |
| `B23` | promotion-critical native capabilities | mobile and cross-platform capability blockers are closed or explicitly waived |
| `B24` | secondary route migration | promotion-critical `P1` routes are wired |
| `B25` | release and store migration | `vibe-app-tauri` can produce the full app artifact set |
| `B26` | promotion and legacy archival | `packages/vibe-app-tauri` is confirmed as the default app path |

## [done] B00: Planning Freeze

### Prerequisites

- repository reset complete

### Module Order

1. `shared/source-crosswalk.md`
2. `shared/naming.md`
3. `shared/data-model.md`
4. `shared/protocol-session.md`
5. `shared/protocol-auth-crypto.md`
6. `shared/protocol-api-rpc.md`
7. `shared/validation.md`
8. `shared/migration-order.md`
9. `execution-plan.md`
10. `execution-batches.md`

### Parallel Allowed

- no; treat this batch as serial

### Gate

- every major Happy source tree has an owner and no critical protocol ambiguity remains

### Validation Focus

- internal document consistency
- no missing module owner for critical-path work

## [done] B01: Wire Core Schemas

### Prerequisites

- `B00` complete

### Module Order

1. `modules/vibe-wire/message-meta.md`
2. `modules/vibe-wire/legacy-protocol.md`
3. `modules/vibe-wire/session-protocol.md`

### Parallel Allowed

- `legacy-protocol.md` and `session-protocol.md` may run in parallel only after
  `message-meta.md` is stable

### Gate

- message metadata, legacy messages, and session envelopes/events are implemented and testable

### Validation Focus

- JSON round-trips
- fixture coverage for legacy and session payloads

## [done] B02: Wire Containers And Voice

### Prerequisites

- `B01` complete

### Module Order

1. `modules/vibe-wire/messages.md`
2. `modules/vibe-wire/voice.md`

### Parallel Allowed

- no; `messages.md` first

### Gate

- `vibe-wire` is publishable as the canonical dependency for downstream crates

### Validation Focus

- update container fixtures
- public type coverage check against `shared/source-crosswalk.md`

## [done] B03: Server Foundation

### Prerequisites

- `B02` complete

### Module Order

1. `modules/vibe-server/versions-and-config.md`
2. `modules/vibe-server/storage-db.md`
3. `modules/vibe-server/storage-redis.md`

### Parallel Allowed

- `versions-and-config.md` may run in parallel with early `storage-db.md` setup if write scopes are
  disjoint

### Gate

- server startup/config exists and primary persistence surfaces are available

### Validation Focus

- startup/config smoke test
- storage schema and migration checks

## [done] B04: Server Minimum Durable Domain

### Prerequisites

- `B03` complete

### Module Order

1. `modules/vibe-server/auth.md`
2. `modules/vibe-server/event-router.md` pass A
3. `modules/vibe-server/session-lifecycle.md`
4. `modules/vibe-server/machine-lifecycle.md`

### Parallel Allowed

- `session-lifecycle.md` and `machine-lifecycle.md` may run in parallel only after auth and
  event-router pass A are stable

### Gate

- authenticated session and machine records can be created, read, and updated

### Validation Focus

- auth flow
- session CRUD
- machine registration and optimistic concurrency

## [done] B05: Server Live Path

### Prerequisites

- `B04` complete

### Module Order

1. `modules/vibe-server/presence.md`
2. `modules/vibe-server/socket-updates.md` pass A
3. `modules/vibe-server/app-api.md` pass A

### Parallel Allowed

- no; presence first

### Gate

- minimum live backend path exists for remote-control clients

### Validation Focus

- socket auth and reconnect behavior
- heartbeat and timeout behavior
- minimum HTTP route registration

## [done] B06: Agent Foundation

### Prerequisites

- `B05` complete

### Module Order

1. `modules/vibe-agent/config.md`
2. `modules/vibe-agent/encryption.md`
3. `modules/vibe-agent/credentials-and-auth.md`
4. `modules/vibe-agent/http-api-client.md`

### Parallel Allowed

- `config.md` and `encryption.md` may run in parallel if they do not touch the same files

### Gate

- `vibe-agent` can authenticate and issue the core HTTP control calls

### Validation Focus

- config resolution
- crypto compatibility vectors
- auth login/logout/status
- HTTP client route coverage

## [done] B07: Agent Live Control

### Prerequisites

- `B06` complete

### Module Order

1. `modules/vibe-agent/session-socket-client.md`
2. `modules/vibe-agent/machine-rpc.md`
3. `modules/vibe-agent/cli-output.md`

### Parallel Allowed

- `session-socket-client.md` and `cli-output.md` may overlap only after the socket client DTOs are
  fixed

### Gate

- one real remote-control flow works end-to-end through `vibe-agent`

### Validation Focus

- live update handling
- machine RPC
- human and JSON output stability

## [done] B08: Server Support APIs

### Prerequisites

- `B07` complete

### Module Order

1. `modules/vibe-server/storage-files.md`
2. `modules/vibe-server/image-processing.md`
3. `modules/vibe-server/account-and-usage.md`
4. `modules/vibe-server/utility-apis.md`
5. `modules/vibe-server/artifacts-and-access-keys.md`
6. `modules/vibe-server/connect-vendors.md`
7. `modules/vibe-server/github.md`
8. `modules/vibe-server/social.md`
9. `modules/vibe-server/feed.md`

### Parallel Allowed

- `account-and-usage.md`, `utility-apis.md`, `connect-vendors.md`, `social.md`, and `feed.md` may
  be parallelized after file/image/storage prerequisites are ready

### Gate

- all non-core support domains needed by imported app and local runtime have real owning services

### Wave 4 Feature Inventory

- storage/files:
  - object reference types
  - upload/store/retrieve service seam
  - dev/test local adapter
- image-processing:
  - deterministic image normalize pipeline
  - placeholder/thumbhash metadata seam
- account-and-usage:
  - `/v1/account/profile`
  - `/v1/account/settings` read/write with optimistic concurrency
  - `/v1/usage/query`
- utility-apis:
  - `/v1/kv/:key`
  - `/v1/kv`
  - `/v1/kv/bulk`
  - `/v1/kv`
  - `/v1/push-tokens`
  - `/v1/push-tokens/:token`
  - `/v1/voice/token`
- artifacts-and-access-keys:
  - `/v1/artifacts` CRUD
  - `/v1/access-keys/:sessionId/:machineId` CRUD/rotate
  - auxiliary socket artifact and access-key APIs
- connect-vendors:
  - `/v1/connect/:vendor/register`
  - `/v1/connect/:vendor/token`
  - `/v1/connect/:vendor`
  - `/v1/connect/tokens`
- github:
  - `/v1/connect/github/params`
  - `/v1/connect/github/callback`
  - `/v1/connect/github/webhook`
  - `/v1/connect/github`
- social:
  - `/v1/user/:id`
  - `/v1/user/search`
  - `/v1/friends`
  - `/v1/friends/add`
  - `/v1/friends/remove`
- feed:
  - `/v1/feed`
  - durable feed update ownership for later socket/event-router wiring

### Recommended Delivery Slices

1. shared support storage:
   account settings, usage reports, kv, push tokens, vendor tokens, artifact records, access keys
2. app bootstrap routes:
   account/profile/settings, utility APIs, connect/github bootstrap
3. collaboration routes:
   social and feed read/write surfaces
4. opaque session-adjacent data:
   artifacts and access keys over HTTP first, socket second
5. pass-B completion:
   late route registration, durable update shaping, auxiliary socket hooks, monitoring

### Validation Focus

- route-group completeness against `shared/protocol-api-rpc.md`
- artifact/access-key HTTP and socket compatibility

## [done] B09: Server Finalization

### Prerequisites

- `B08` complete

### Module Order

1. `modules/vibe-server/event-router.md` pass B
2. `modules/vibe-server/socket-updates.md` pass B
3. `modules/vibe-server/app-api.md` pass B
4. `modules/vibe-server/monitoring.md`

### Parallel Allowed

- monitoring can begin late in parallel with app-api pass B once route registration is mostly fixed

### Gate

- server route and socket surfaces are complete enough for app and CLI integration

### Wave 4 Exit Criteria

- every Wave 4 HTTP route group listed in `shared/protocol-api-rpc.md` is mounted in
  `crates/vibe-server`
- late support-domain services own their storage and JSON shaping instead of ad hoc handler logic
- artifact/access-key auxiliary socket APIs exist behind the already-stable `/v1/updates`
  transport
- pass-B event-router and app-api work broadens support-domain coverage without changing the Wave 2
  sequencing spine

### Review Remediation Checklist

- HTTP route mounting alone is not sufficient; each mounted Wave 4 route must also preserve the
  Happy-compatible request/response body shape recorded in `shared/protocol-api-rpc.md`
- account settings, artifacts, feed, social, and related support-domain writes must emit the
  required durable updates before Wave 4 can be called complete
- `/v1/updates` pass-B auxiliary socket APIs and `usage-report` ingestion are required Wave 4
  deliverables, not optional follow-up work
- placeholder GitHub and voice stubs do not satisfy Wave 4 acceptance; these routes must either
  implement the Happy-compatible provider flow or fail with the same compatibility-locked error
  semantics
- `storage-files`, `image-processing`, and `monitoring` remain part of the Wave 4 definition and
  must not be omitted from the final completion claim
- Wave 4 remediation also requires fixing correctness defects found during review:
  - artifact optimistic-concurrency writes must remain atomic across header/body updates
  - KV batch mutation must remain all-or-nothing when any mutation in the batch conflicts
  - auxiliary socket APIs must enforce the same ownership and account scoping guarantees as their
    HTTP counterparts
  - displaced GitHub-account owners must receive the required `update-account` disconnect update
    when another account takes over the same GitHub linkage
  - auxiliary socket artifact create and usage-report flows must preserve the same idempotency and
    timestamp semantics as Happy

### Validation Focus

- late support-domain update routing audit
- full route registration audit
- auxiliary socket API audit
- monitoring hooks on stable surfaces only

## [done] B10: CLI Foundation

### Prerequisites

- `B09` complete

### Module Order

1. `modules/vibe-cli/utils-and-parsers.md`
2. `modules/vibe-cli/ui-terminal.md`
3. `modules/vibe-cli/bootstrap-and-commands.md` pass A
4. `modules/vibe-cli/agent-core.md`
5. `modules/vibe-cli/agent-adapters.md`
6. `modules/vibe-cli/session-protocol-mapper.md`
7. `modules/vibe-cli/transport.md`
8. `modules/vibe-cli/auth.md`
9. `modules/vibe-cli/api-client.md`

### Parallel Allowed

- `utils-and-parsers.md` and `ui-terminal.md` may run in parallel
- `agent-core.md` and early `bootstrap-and-commands.md` may overlap once config ownership is fixed

### Gate

- local CLI architecture exists and can authenticate plus route provider output toward the server

### Validation Focus

- parser/bootstrap tests
- mapper/transport tests
- API client and auth tests

## [done] B11: CLI Control Plane

### Prerequisites

- `B10` complete

### Module Order

1. `modules/vibe-cli/daemon.md`
2. `modules/vibe-cli/sandbox.md`
3. `modules/vibe-cli/persistence-resume.md`
4. `modules/vibe-cli/builtin-modules.md`

### Parallel Allowed

- `sandbox.md` and `builtin-modules.md` may run in parallel after daemon interfaces are fixed

### Gate

- daemonized local runtime control and persistence/resume infrastructure are real

### Validation Focus

- daemon lifecycle
- sandbox policy
- persistence/resume correctness

## [done] B12: CLI First Provider Vertical Slice

### Prerequisites

- `B11` complete

### Module Order

1. `modules/vibe-cli/claude-runtime.md`
2. `modules/vibe-cli/testing-fixtures.md` pass A

### Parallel Allowed

- no; keep this vertical slice serial

### Gate

- first provider runtime slice works end-to-end through the shared CLI core and `vibe-server`

### Validation Focus

- provider event mapping
- runtime/daemon integration
- first end-to-end CLI integration harness

## [done] B13: CLI Provider Expansion

### Prerequisites

- `B12` complete

### Module Order

1. `modules/vibe-cli/codex-runtime.md`
2. `modules/vibe-cli/gemini-runtime.md`
3. `modules/vibe-cli/openclaw-runtime.md`
4. `modules/vibe-cli/agent-acp.md`
5. `modules/vibe-cli/bootstrap-and-commands.md` pass B
6. `modules/vibe-cli/testing-fixtures.md` pass B

### Parallel Allowed

- provider modules may run in parallel if they keep disjoint write scopes
- keep `bootstrap-and-commands.md` pass B and `testing-fixtures.md` pass B serial after the
  provider set stabilizes

### Gate

- remaining Happy-represented provider paths and final CLI command wiring land on the stable core

### Validation Focus

- per-provider runtime tests
- top-level command wiring regression checks
- broader fixture matrix

## [done] B14: App Import Baseline

### Prerequisites

- `B13` complete

### Module Order

1. `modules/vibe-app/import-and-build.md`

### Parallel Allowed

- no; import/build first

### Gate

- imported app builds inside this repo with explicit root bootstrap files

### Wave 6 Feature Inventory

- import `packages/happy-app/**` into `packages/vibe-app` with the Happy layout preserved first
- localize the required root bootstrap files (`package.json`, `yarn.lock`, `scripts/postinstall.cjs`,
  `patches/fix-pglite-prisma-bytes.cjs`) so the app can install in this repo
- remove or replace stale root-relative assumptions such as the `hello-world` alias, local
  `CHANGELOG.md` dependency, and Tauri schema path drift
- replace the imported `@slopus/happy-wire` dependency with a Vibe-owned compatibility seam
- record any remaining compatibility-locked `happy` identifiers and validate install/build

### Validation Focus

- install/build reproducibility
- root-relative path cleanup

## [done] B15: App Adaptation

### Prerequisites

- `B14` complete

### Module Order

1. `modules/vibe-app/protocol-parser-compat.md`
2. `modules/vibe-app/api-endpoint-adaptation.md`
3. `modules/vibe-app/branding-and-naming-adaptation.md`
4. `modules/vibe-app/desktop-tauri-adaptation.md`
5. `modules/vibe-app/release-and-env.md`

### Parallel Allowed

- branding may run in parallel with endpoint adaptation only after protocol parser work is stable

### Gate

- app works against Vibe backend path without public Happy leakage

### Wave 6 Feature Inventory

- validate and normalize parser/reducer behavior against `vibe-wire` fixtures
- centralize Vibe server URL, socket endpoint, and app runtime env resolution
- replace public Happy naming with Vibe naming across titles, package metadata, deep links, and
  user-facing strings
- adapt desktop/Tauri bundle identifiers, config, and script paths to the Vibe package layout
- finalize release profiles and env variables under `EXPO_PUBLIC_VIBE_*` and `VIBE_*`
- validate the integrated app surface against Vibe services without introducing protocol forks

### Validation Focus

- protocol parsing
- endpoint/path adaptation
- user-visible naming
- Tauri shell checks

## [done] B16: Optional Sidecar

### Prerequisites

- `B15` complete
- imported app proves the sidecar is still required

### Module Order

1. `modules/vibe-app-logs/log-server.md`

### Parallel Allowed

- no

### Gate

- sidecar behavior required by app tooling is satisfied or explicitly retired

### Validation Focus

- sidecar startup and ingestion smoke tests

## B17: `vibe-app-tauri` Next Desktop Iteration

### Status

- historical and closed; do not dispatch new work here

### Prerequisites

- `B16` complete
- `projects/vibe-app-tauri.md` exists and is treated as the owning project plan
- `vibe-app-tauri-extraction-inventory.md` exists
- `vibe-app-tauri-route-inventory.md` exists
- `vibe-app-tauri-capability-matrix.md` exists
- `vibe-app-tauri-coexistence-matrix.md` exists
- `shared/source-crosswalk.md`, `shared/validation.md`, `shared/naming.md`, and
  `master-details.md` reflect the new parallel desktop project boundary

### Module Order

1. `[done]` `modules/vibe-app-tauri/bootstrap-and-package.md`
2. `modules/vibe-app-tauri/desktop-shell-and-routing.md`
3. `modules/vibe-app-tauri/core-logic-extraction.md`
4. `modules/vibe-app-tauri/desktop-platform-adapters.md`
5. `modules/vibe-app-tauri/auth-and-session-state.md`
6. `modules/vibe-app-tauri/session-ui-parity.md`

### Parallel Allowed

- `core-logic-extraction.md` may overlap lightly with late shell work only after the new package
  and route/layout ownership are fixed
- `desktop-platform-adapters.md` may continue late hardening in parallel with `auth-and-session-state.md`
  or `session-ui-parity.md` only after the auth-critical adapter layer is stable
- keep the rest serial to avoid broad concurrent rewrites in a greenfield desktop package

### Gate

- a separate `packages/vibe-app-tauri` desktop app exists, reaches a first usable desktop session
  slice against the real Vibe backend, and preserves the current `packages/vibe-app` as the
  production baseline

### Validation Focus

- package bootstrap and Tauri shell smoke tests
- auth/session desktop chain against a real backend
- route-level desktop navigation checks
- historical parity-checklist progress review against current desktop behavior

## B18: `vibe-app-tauri` Promotion Readiness

### Status

- historical and closed; do not dispatch new work here

### Prerequisites

- `B17` complete
- first-usable-slice parity gaps are recorded in `vibe-app-tauri-parity-checklist.md`

### Module Order

1. `modules/vibe-app-tauri/secondary-surfaces.md`
2. `modules/vibe-app-tauri/release-and-promotion.md`

### Parallel Allowed

- no; complete secondary surfaces before finalizing release and promotion rules

### Gate

- `vibe-app-tauri` closes required promotion-scope parity items, keeps historical coexistence rules documented, and does not reopen `packages/vibe-app` as an active lane before sign-off

### Validation Focus

- secondary-surface route and integration checks
- release artifact and startup validation on Linux, macOS, and Windows
- historical parity-checklist sign-off review where continuity notes still matter


## B19: `vibe-app-tauri` Unified Runtime Bootstrap

### Prerequisites

- Wave 8 is closed as historical desktop-preview baseline material
- `projects/vibe-app-tauri.md` records the Wave 9 project boundary
- `vibe-app-tauri-wave9-unified-replacement-plan.md` exists
- `vibe-app-tauri-wave9-route-and-capability-matrix.md` exists
- `vibe-app-tauri-wave9-migration-and-release-plan.md` exists

### Module Order

1. `modules/vibe-app-tauri/universal-bootstrap-and-runtime.md`

### Implementation Tasks

1. Create or normalize the package-root bootstrap files so `packages/vibe-app-tauri` owns the
   package-root desktop/mobile/browser bootstrap surface directly.
2. Define the package-internal directory layout for `sources/app`, `sources/shared`,
   `sources/mobile`, `sources/desktop`, and `src-tauri` without importing from `packages/vibe-app`
   at runtime.
3. Port root theme, font, splash, and provider bootstrap dependencies from Happy into package-local
   ownership.
4. Add or normalize scripts for `tauri:dev`, desktop build/smoke flows, mobile build/dev flows,
   and retained static browser export validation.
5. Make env/config resolution explicit for preview versus production modes and keep outputs
   package-local.
6. Validate that desktop boot, Android mobile runtime ownership, and retained static browser export
   all work without mutating `packages/vibe-app`.

### Parallel Allowed

- no; stabilize the package structure first

### Gate

- `packages/vibe-app-tauri` can host the desktop shell, Android Tauri-mobile ownership, and
  retained static browser export path in one package

### Validation Focus

- Tauri boot
- mobile runtime/bootstrap validation
- Android mobile-path validation
- retained static browser export validation

## B20: Shared Core Import From Happy

### Prerequisites

- `B19` complete

### Module Order

1. `modules/vibe-app-tauri/shared-core-from-happy.md`

### Implementation Tasks

1. Inventory Happy auth/sync/realtime/encryption/text/changelog/constants/utils/hooks modules into
   reusable, adapter-needed, and UI-only categories.
2. Port reusable auth and encryption helpers into `sources/shared` with Vibe naming and current
   endpoint assumptions.
3. Port sync, reducer, parser, and realtime state logic into `sources/shared`, keeping protocol
   behavior aligned to `vibe-wire`.
4. Port text, changelog, constants, and UI-independent helpers into package-local shared modules.
5. Add import-boundary enforcement so shared core does not pull in screen-level React Native code.
6. Add shared-core unit and compatibility tests for auth/sync/realtime/parser seams before route work
   broadens.

### Parallel Allowed

- no; keep shared-core ownership tight before screen work broadens

### Gate

- replacement package owns reusable auth/sync/realtime/core logic directly ported from Happy

### Validation Focus

- shared-core unit tests
- parser/reducer/auth/realtime checks
- import-boundary checks

## B21: Shell Surfaces, Static Browser Export, And Identity

### Prerequisites

- `B20` complete

### Module Order

1. `modules/vibe-app-tauri/mobile-shell-and-navigation.md`
2. `modules/vibe-app-tauri/web-export-and-browser-runtime.md`
3. `modules/vibe-app-tauri/desktop-shell-and-platform-parity.md`
4. `modules/vibe-app-tauri/auth-and-identity-flows.md`

### Implementation Tasks

1. Recreate the Happy root provider chain and top-level route naming inside the new package for
   Android and retained static browser export ownership.
2. Port Android `P0` entry routes: home, restore, inbox, settings hub, and top-level authenticated
   shell behavior.
3. Stand up retained static browser export wiring, favicon/metadata hooks, and browser-only provider
   affordances.
4. Recreate the active desktop shell: route chrome, header/sidebar behavior, keyboard/focus rules,
   modal/overlay semantics, clipboard flows, and required file-dialog/notification seams.
5. Port create-account, QR/device-link, secret-key restore, and credential persistence flows for
   both mobile and desktop.
6. Harden the desktop localhost loopback callback contract with `127.0.0.1` binding, per-attempt
   `state`, one-shot lifecycle, timeout handling, and per-process ownership.
7. Validate desktop/Android/static-export `P0` entry flows and auth bootstrap behavior before
   session work starts.

### Parallel Allowed

- limited overlap is allowed only after the provider, static-export path, and route shell are stable

### Gate

- desktop/Android `P0` entry routes plus retained static browser export and create/link/restore
  flows work on the new package

### Validation Focus

- provider boot
- desktop shell, keyboard/focus, and adapter checks
- Android phone/tablet route checks
- retained static browser export checks
- create-account, device-link, and secret-key restore checks

## B22: Session Runtime And Rendering

### Prerequisites

- `B21` complete

### Module Order

1. `modules/vibe-app-tauri/session-runtime-and-storage.md`
2. `modules/vibe-app-tauri/session-rendering-and-composer.md`

### Implementation Tasks

1. Port session bootstrap, profile/bootstrap fetch chains, realtime subscriptions, and local state
   persistence into shared runtime modules.
2. Recreate session inventory selectors and hooks required by home, inbox, recent-session, and
   session-detail routes.
3. Port session-detail shell behavior and message timeline rendering by message kind.
4. Port composer send/abort/autocomplete/mode-selection behavior for both desktop and mobile hosts.
5. Port markdown, diff, tool, and file rendering semantics with parity-focused validation.
6. Validate one real end-to-end session chain covering bootstrap, realtime receipt, message send,
   and tool/file rendering on desktop and mobile.

### Parallel Allowed

- rendering work may overlap lightly only after runtime selectors and realtime state are stable

### Gate

- desktop and mobile both reach a real end-to-end session chain

### Validation Focus

- session inventory and bootstrap checks
- realtime update checks
- message/composer/tool rendering checks

## B23: Promotion-Critical Native Capabilities

### Prerequisites

- `B22` complete

### Module Order

1. `modules/vibe-app-tauri/mobile-native-capabilities.md`

### Implementation Tasks

1. Turn the Wave 9 capability matrix into an implementation checklist covering each `C1`
   promotion-critical capability and every Happy mobile/native integration seam.
2. Implement or explicitly defer notification routing and push registration behavior with real-device
   validation.
3. Implement or explicitly defer purchases and entitlement refresh behavior with continuity notes.
4. Implement or explicitly defer QR/camera and voice/microphone flows, including permission behavior
   on real devices.
5. Implement or explicitly defer file import/export/share flows where route parity depends on them.
6. Record desktop-specific capability rules that still matter for promotion, especially clipboard,
   file dialog, and shell interaction semantics that are owned outside the mobile-native module.
7. Write down release-impacting keep/defer decisions in the migration plan before promotion work
   continues.

### Parallel Allowed

- no; keep capability ownership explicit and serial

### Gate

- every promotion-critical capability is implemented or explicitly waived in writing

### Validation Focus

- notification routing
- purchases and entitlement refresh
- QR/camera
- voice/microphone
- file/share flows where required

## B24: Secondary Route Migration

### Prerequisites

- `B23` complete

### Module Order

1. `modules/vibe-app-tauri/secondary-routes-and-social.md`

### Implementation Tasks

1. Port `P1` settings detail routes in priority order: account, appearance, features, language,
   voice, voice/language, usage, and connect/claude.
2. Port `P1` session-adjacent detail routes: message permalink, info, files, and single-file view.
3. Port `P1` artifacts flows: list, create, detail, and edit.
4. Port `P1` terminal/connect, server, changelog, and text-selection surfaces.
5. Port `P1` friends, user, and machine surfaces while keeping social/feed scope explicit.
6. Add route smoke checks and targeted integration checks for artifacts, settings detail, and friend
   flows.
7. Record any surviving `P2` or low-value dev-route deferrals explicitly in planning before moving
   on to promotion.

### Parallel Allowed

- no; keep route-parity closure ordered and reviewable

### Gate

- promotion-critical `P1` routes are present and wired

### Validation Focus

- route smoke tests for `P1` surfaces
- targeted artifacts/settings/friends checks

## B25: Release And Store Migration

### Prerequisites

- `B24` complete

### Module Order

1. `modules/vibe-app-tauri/release-ota-and-store-migration.md`

### Implementation Tasks

1. Recreate package-local release inputs: `release.cjs`, browser/native build config,
   `release-dev.sh`, and `release-production.sh`.
2. Port preview and production identifier rules for desktop, Android, deep links, updater channels,
   APK naming, and GitHub Release metadata without colliding with historical shipping ownership.
3. Update repo workflows so preview and production-candidate lanes package from
   `packages/vibe-app-tauri`.
4. Validate preview and production-candidate artifact generation for desktop, Android APK, and
   retained static browser export outputs.
5. Record the analytics/tracking continuity keep/defer decision, including provider bootstrap,
   screen tracking, opt-in/out state, and review-prompt telemetry implications.
6. Complete the data-migration review table with concrete validation artifacts for each continuity
   area.
7. Document exact rollback mechanics and confirm no legacy `packages/vibe-app` upgrade-validation
   lane remains in scope.

### Parallel Allowed

- no; release-owner switch prep should be serialized

### Gate

- `packages/vibe-app-tauri` can produce the app artifacts required for the default app path

### Validation Focus

- preview and production-candidate artifact generation
- retained static browser export artifact generation
- workflow packaging checks
- analytics/tracking continuity review
- rollback-path review

## B26: Promotion And Legacy Deprecation

### Prerequisites

- `B25` complete

### Module Order

1. `modules/vibe-app-tauri/promotion-and-vibe-app-deprecation.md`

### Implementation Tasks

1. Confirm every `P0`/`P1` route and `C0`/`C1` capability is either satisfied or explicitly waived
   in writing.
2. Confirm default release ownership is runnable across desktop, Android, and retained static
   browser export outputs.
3. Run and record the final hold/rollback drill against the active Wave 9 replacement package.
4. Update docs, helper scripts, workflow defaults, and release-owner notes so they point to the
   default app path.
5. Record the fallback retention window and eventual retirement policy for `packages/vibe-app`.
6. Produce the final promotion decision record showing that `packages/vibe-app-tauri` is now the
   default app path and `packages/vibe-app` remains reference-only.

### Parallel Allowed

- no; promotion is a single explicit decision point

### Gate

- `packages/vibe-app-tauri` can be confirmed as the default app path with hold/rollback and archival
  rules documented

### Validation Focus

- promotion checklist review
- release-owner switch review
- rollback drill review
- docs/workflow default-owner review

## Direct Prompt Template

Use this template when dispatching a batch item:

> Implement `<module-plan-path>`.
> Follow `<project-plan-path>` and the referenced shared specs.
> Assume all earlier modules in batch `<batch-id>` are complete.
> Treat `<previous-module-plan-path>` as the immediately preceding implementation dependency.
> Preserve the batch gate: `<gate-text>`.
> If code reality differs from the plan, update the plan first before continuing.
