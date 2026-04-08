# Happy To Vibe Source Crosswalk

## Package-Level Mapping

| Happy source | Happy responsibility | Vibe target | Target module root | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `packages/happy-wire` | shared message schemas and protocol helpers | `crates/vibe-wire` | `src/` | done | canonical Rust source for all shared contracts |
| `packages/happy-server` | backend APIs, socket updates, storage, auth | `crates/vibe-server` | `src/` | done | Rust rewrite landed; route, storage, auth, and socket ownership now live under the current server tree |
| `packages/happy-agent` | remote-control CLI client | `crates/vibe-agent` | `src/` | done | remote session and machine control client is implemented and validated |
| `packages/happy-cli` | local runtime, daemon, provider integrations | `crates/vibe-cli` | `src/` | done | local runtime, daemon, and provider ownership now live under the current CLI tree |
| `packages/happy-app` | imported Wave 7 Vibe app baseline | `packages/vibe-app` | deprecated legacy reference tree | deprecated | keep only as a Vibe-specific historical reference when `/root/happy/packages/happy-app` cannot answer a continuity question; do not use as an active CI or release owner |
| `packages/happy-app` | active Wave 9 mobile/web/desktop replacement target | `packages/vibe-app-tauri` | active replacement tree | in-progress | direct Happy reference now flows to `packages/vibe-app-tauri`; new planning and active pipelines should target this package first |
| `packages/happy-app-logs` | log sidecar | `crates/vibe-app-logs` | `src/` | done | minimal Wave-7 sidecar runtime now satisfies the imported app log receiver flow |

## Coverage Conventions

- `*.test.ts`, `*.spec.ts`, and other test-only files inherit the owner of the source module they
  validate; they do not create standalone Vibe module targets.
- `packages/vibe-app` is a deprecated legacy reference. Use it only when `/root/happy/packages/happy-app`
  cannot answer a Vibe-specific continuity question; do not use it to drive new planning, CI, or release ownership.
- The detailed `packages/vibe-app` rows below remain as historical Wave 7 mapping records rather than active ownership guidance.
- Helper-only files must stay with the nearest owning subsystem plan. Do not create new top-level
  `utils` modules in Vibe unless a later plan records that boundary change first.
- Root bootstrap files, lockfiles, environment manifests, and migration trees that a planned import
  or rewrite depends on must be listed explicitly in this table. Package-level wildcard ownership is
  not enough for those shared build inputs.
- Critical-path HTTP route entrypoints and socket handler entrypoints must be listed explicitly in
  this table even if a broader wildcard entry already exists elsewhere.

## High-Value Module Mapping

| Happy source path | Happy responsibility | Vibe target crate/package | Vibe target module | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `packages/happy-wire/src/index.ts` | package export surface and type barrel | `crates/vibe-wire` | `lib.rs` | done | re-export only; public contracts stay canonical in `vibe-wire` |
| `packages/happy-wire/src/messages.ts` | wire message containers | `crates/vibe-wire` | `messages.rs` | done | define encrypted message and update containers |
| `packages/happy-wire/src/sessionProtocol.ts` | session envelope and event union | `crates/vibe-wire` | `session_protocol.rs` | done | primary protocol source of truth |
| `packages/happy-wire/src/legacyProtocol.ts` | legacy user/agent content wrapper | `crates/vibe-wire` | `legacy_protocol.rs` | done | required for app compatibility |
| `packages/happy-wire/src/messageMeta.ts` | message metadata | `crates/vibe-wire` | `message_meta.rs` | done | shared by CLI, agent, app, server |
| `packages/happy-wire/src/voice.ts` | voice token schema | `crates/vibe-wire` | `voice.rs` | done | add serde + validation |
| `packages/happy-server/sources/main.ts` | primary server binary bootstrap | `crates/vibe-server` | `main.rs` | done | owned by `versions-and-config`; no separate bootstrap layer outside typed config/context |
| `packages/happy-server/sources/standalone.ts` | alternate standalone startup path | `crates/vibe-server` | `main.rs` | done | folded into the same startup/bootstrap module as `main.ts` |
| `packages/happy-server/sources/versions.ts` | version/build metadata helpers | `crates/vibe-server` | `version.rs` | done | owned by `versions-and-config` |
| `packages/happy-server/sources/context.ts` | shared server context construction | `crates/vibe-server` | `context.rs` | done | typed app context for router and domain services |
| `packages/happy-server/sources/types.ts` | shared server DTOs used across API/domain layers | `crates/vibe-server` | `api/types.rs` | done | shared account/artifact/GitHub-adjacent API shapes live with `app-api` |
| `packages/happy-server/sources/app/api/api.ts` | top-level HTTP router bootstrap | `crates/vibe-server` | `api/mod.rs` | done | owned by `app-api` pass A/B |
| `packages/happy-server/sources/app/api/types.ts` | HTTP-layer DTOs and shared API type helpers | `crates/vibe-server` | `api/types.rs` | done | router-facing types owned by `app-api` and reused by GitHub/account routes |
| `packages/happy-server/sources/app/api/socket.ts` | Socket.IO transport bootstrap and connection scope handling | `crates/vibe-server` | `api/socket.rs` | done | transport entrypoint owned by `socket-updates`; domain writes still live in their owning modules |
| `packages/happy-server/sources/app/api/utils/*` | API bootstrap helpers for auth, error handling, and monitoring | `crates/vibe-server` | `api/mod.rs` | done | owned by `app-api` bootstrap rather than standalone feature modules |
| `packages/happy-server/sources/app/auth/*` | auth endpoints and guards | `crates/vibe-server` | `auth/` | done | implemented under the shared auth spec |
| `packages/happy-server/sources/app/events/eventRouter.ts` | update/event routing | `crates/vibe-server` | `events/router.rs` | done | socket + fanout hub |
| `packages/happy-server/sources/app/api/routes/authRoutes.ts` | auth and account-link HTTP route registration | `crates/vibe-server` | `auth/` | done | HTTP auth route glue stays with the auth module; `app-api` only mounts the route group |
| `packages/happy-server/sources/app/api/routes/accountRoutes.ts` | account profile, settings, and usage routes | `crates/vibe-server` | `api/account.rs` | done | own account-and-usage surface |
| `packages/happy-server/sources/app/api/routes/connectRoutes.ts` | generic vendor connect routes and GitHub route registration | `crates/vibe-server` | `api/connect.rs` | done | split generic vendor logic from GitHub-specific behavior |
| `packages/happy-server/sources/app/api/routes/devRoutes.ts` | optional AI-debug logging endpoint | `crates/vibe-server` | `api/dev.rs` | deferred | debug-only route; exclude from parity unless a concrete consumer requires it |
| `packages/happy-server/sources/app/api/routes/artifactsRoutes.ts` | artifact CRUD routes | `crates/vibe-server` | `api/artifacts.rs` | done | pair with socket artifact handlers |
| `packages/happy-server/sources/app/api/routes/accessKeysRoutes.ts` | access-key CRUD routes | `crates/vibe-server` | `api/artifacts.rs` | done | access-key HTTP handlers stay grouped with artifact/access-key support APIs; shared DTOs remain in `api/types.rs` |
| `packages/happy-server/sources/app/api/routes/sessionRoutes.ts` | legacy `/v1` and `/v2` session CRUD, list, and history routes | `crates/vibe-server` | `sessions/http.rs` | done | session module owns create-or-load-by-tag, list, delete, and legacy history HTTP behavior; `app-api` only mounts it |
| `packages/happy-server/sources/app/api/routes/v3SessionRoutes.ts` | paged and idempotent v3 session message routes | `crates/vibe-server` | `sessions/http.rs` | done | keep v1/v2 and v3 session HTTP semantics in the same session module so message/history contracts do not drift |
| `packages/happy-server/sources/app/api/routes/kvRoutes.ts` | key-value support routes | `crates/vibe-server` | `api/utility.rs` | done | grouped support API handlers; shared DTOs stay in `api/types.rs` |
| `packages/happy-server/sources/app/api/routes/pushRoutes.ts` | push token registration routes | `crates/vibe-server` | `api/utility.rs` | done | grouped support API handlers; shared DTOs stay in `api/types.rs` |
| `packages/happy-server/sources/app/api/routes/versionRoutes.ts` | version check route | `crates/vibe-server` | `api/utility.rs` | done | grouped support API handlers; shared DTOs stay in `api/types.rs` |
| `packages/happy-server/sources/app/api/routes/voiceRoutes.ts` | voice token route | `crates/vibe-server` | `api/utility.rs` | done | grouped support API handlers; voice wire DTOs continue to come from `vibe-wire` |
| `packages/happy-server/sources/app/api/routes/feedRoutes.ts` | feed route registration and response DTO glue | `crates/vibe-server` | `api/feed.rs` | done | feed HTTP surface is grouped under the shared API tree rather than a standalone `feed/http.rs` module |
| `packages/happy-server/sources/app/api/routes/userRoutes.ts` | user profile/search and friend route registration | `crates/vibe-server` | `api/social.rs` | done | social HTTP surface is grouped under the shared API tree rather than a standalone `social/http.rs` module |
| `packages/happy-server/sources/app/api/socket/pingHandler.ts` | transport health/ping socket handler | `crates/vibe-server` | `api/socket.rs` | done | transport-local helper; keep with socket bootstrap instead of creating a separate ping module |
| `packages/happy-server/sources/app/api/socket/sessionUpdateHandler.ts` | inbound session socket events and acks | `crates/vibe-server` | `api/socket.rs` | done | transport entrypoint delegates durable state changes to `session-lifecycle` |
| `packages/happy-server/sources/app/api/socket/rpcHandler.ts` | machine RPC socket forwarding | `crates/vibe-server` | `api/socket.rs` | done | socket transport owns forwarding and ack framing; machine behavior stays in `machine-lifecycle` |
| `packages/happy-server/sources/app/session/*` | session lifecycle helpers and deletion logic | `crates/vibe-server` | `sessions/` | done | session CRUD and lifecycle module |
| `packages/happy-server/sources/app/feed/*` | feed domain logic | `crates/vibe-server` | `api/feed.rs` | done | phase-one implementation keeps feed HTTP/service behavior inside the shared API tree |
| `packages/happy-server/sources/app/social/*` | social and friend domain logic | `crates/vibe-server` | `api/social.rs` | done | phase-one implementation keeps social HTTP/service behavior inside the shared API tree |
| `packages/happy-server/sources/app/github/*` | GitHub integration domain logic | `crates/vibe-server` | `api/connect.rs` | done | phase-one implementation keeps GitHub-specific connect/disconnect/profile logic in the shared connect module |
| `packages/happy-server/sources/app/monitoring/*` | metrics and monitoring hooks | `crates/vibe-server` | `monitoring/` | done | service metrics and health instrumentation |
| `packages/happy-server/sources/app/api/routes/machinesRoutes.ts` | machine create/list/detail routes | `crates/vibe-server` | `machines/http.rs` | done | machine CRUD and encrypted machine records |
| `packages/happy-server/sources/app/api/socket/machineUpdateHandler.ts` | machine heartbeat and optimistic concurrency updates | `crates/vibe-server` | `machines/socket.rs` | done | machine-alive and machine-update-* socket handlers |
| `packages/happy-server/sources/app/api/socket/artifactUpdateHandler.ts` | artifact socket read/update/create/delete | `crates/vibe-server` | `api/socket.rs` | done | auxiliary artifact socket APIs stay in the unified socket transport module; shared DTOs remain in `api/types.rs` |
| `packages/happy-server/sources/app/api/socket/accessKeyHandler.ts` | access-key socket lookup | `crates/vibe-server` | `api/socket.rs` | done | auxiliary access-key socket APIs stay in the unified socket transport module; shared DTOs remain in `api/types.rs` |
| `packages/happy-server/sources/app/api/socket/usageHandler.ts` | usage-report socket helper surface | `crates/vibe-server` | `api/socket.rs` | done | pass-B auxiliary socket API owned jointly by `socket-updates` transport and `account-and-usage` service |
| `packages/happy-server/sources/app/presence/sessionCache.ts` | session/machine validation cache and batched activeAt flush | `crates/vibe-server` | `presence/cache.rs` | done | lock 30s TTL, 30s threshold, 5s flush interval |
| `packages/happy-server/sources/app/presence/timeout.ts` | session/machine inactivity timeout sweeper | `crates/vibe-server` | `presence/timeout.rs` | done | lock 10 minute timeout and 1 minute sweep |
| `packages/happy-server/sources/app/kv/*` | KV business logic helpers | `crates/vibe-server` | `api/utility.rs` | done | keep KV ownership explicit, even though the phase-one implementation groups support APIs into one module |
| `packages/happy-server/sources/storage/db.ts` | primary relational storage bootstrap | `crates/vibe-server` | `storage/db.rs` | done | relational persistence owner |
| `packages/happy-server/sources/storage/inTx.ts` | transaction wrapper helpers | `crates/vibe-server` | `storage/tx.rs` | done | stays with relational storage, not a generic server helper layer |
| `packages/happy-server/sources/storage/seq.ts` | monotonic sequence allocation helpers | `crates/vibe-server` | `storage/seq.rs` | done | used by sessions/events through `storage-db` |
| `packages/happy-server/prisma/*` | relational schema and migration history | `crates/vibe-server` | `migrations/` | deferred | owned by `storage-db`; current Wave-2/Wave-4 server uses the documented process-local bootstrap seam, and checked SQL migrations have not landed in-repo yet |
| `packages/happy-server/sources/storage/redis.ts` | Redis-backed cache/queue integration | `crates/vibe-server` | `storage/redis.rs` | done | cache and fanout support owned by `storage-redis` |
| `packages/happy-server/sources/storage/files.ts` | object storage accessors and file refs | `crates/vibe-server` | `storage/files.rs` | done | primary file/blob storage module |
| `packages/happy-server/sources/storage/types.ts` | file/image reference types | `crates/vibe-server` | `storage/files.rs` | done | phase-one file/image reference types live with the storage implementation module |
| `packages/happy-server/sources/storage/uploadImage.ts` | storage-layer upload composition for images | `crates/vibe-server` | `storage/files.rs` | done | keep upload orchestration with file storage; image transforms are layered separately |
| `packages/happy-server/sources/storage/repeatKey.ts` | repeated-upload dedupe helper | `crates/vibe-server` | `storage/files.rs` | done | file-storage helper, not its own subsystem |
| `packages/happy-server/sources/storage/simpleCache.ts` | storage-local cache helper | `crates/vibe-server` | `storage/redis.rs` | done | keep local cache helpers inside the cache/storage boundary |
| `packages/happy-server/sources/storage/pgliteLoader.ts` | dev/test local DB bootstrap helper | `crates/vibe-server` | `storage/db.rs` | done | stays with relational storage bootstrap |
| `packages/happy-server/sources/storage/processImage.ts` | image normalization pipeline | `crates/vibe-server` | `storage/process_image.rs` | done | owned by `image-processing` |
| `packages/happy-server/sources/storage/thumbhash.ts` | thumbhash placeholder helper | `crates/vibe-server` | `storage/thumbhash.rs` | done | owned by `image-processing` |
| `packages/happy-server/sources/modules/encrypt.ts` | server-local symmetric encryption bootstrap | `crates/vibe-server` | `auth/` | done | keep server-owned secret handling with auth/config bootstrap; do not fork shared crypto rules elsewhere |
| `packages/happy-server/sources/modules/github.ts` | GitHub app/webhook bootstrap helpers | `crates/vibe-server` | `api/connect.rs` | done | provider bootstrap stays with the GitHub/connect integration module during phase one |
| `packages/happy-server/sources/utils/*` | crate-local helper functions | `crates/vibe-server` | caller-owned helper files | done | helpers stay with the nearest subsystem under `config`, `api`, `events`, `storage`, or domain modules; no standalone `utils` target |
| `packages/happy-agent/src/config.ts` | env and home-path resolution | `crates/vibe-agent` | `config.rs` | done | foundational config/bootstrap module |
| `packages/happy-agent/src/auth.ts` | account auth flow | `crates/vibe-agent` | `auth.rs` | done | QR auth and token persistence |
| `packages/happy-agent/src/credentials.ts` | local credential storage | `crates/vibe-agent` | `credentials.rs` | done | stored under `~/.vibe` |
| `packages/happy-agent/src/encryption.ts` | crypto helpers | `crates/vibe-agent` | `encryption.rs` | done | may share internals with `vibe-wire` |
| `packages/happy-agent/src/api.ts` | REST session and machine client | `crates/vibe-agent` | `api.rs` | done | direct server control path |
| `packages/happy-agent/src/session.ts` | Socket.IO session client | `crates/vibe-agent` | `session.rs` | done | live updates and wait logic |
| `packages/happy-agent/src/machineRpc.ts` | machine-scoped RPC | `crates/vibe-agent` | `machine_rpc.rs` | done | remote runtime control path |
| `packages/happy-agent/src/output.ts` | human-readable and JSON CLI formatting | `crates/vibe-agent` | `output.rs` | done | owned by `cli-output` |
| `packages/happy-agent/src/index.ts` | binary entrypoint and command wiring | `crates/vibe-agent` | `main.rs` | done | top-level CLI wiring stays with `cli-output`, not with transport or API modules |
| `packages/happy-agent/bin/happy-agent.mjs` | packaged agent binary wrapper | `crates/vibe-agent` | `main.rs` | done | CLI packaging wrapper maps to the Rust binary entrypoint |
| `packages/happy-cli/src/index.ts` | top-level CLI entrypoint | `crates/vibe-cli` | `main.rs` | done | owned by `bootstrap-and-commands` plan |
| `packages/happy-cli/bin/happy.mjs` | packaged primary CLI wrapper | `crates/vibe-cli` | `main.rs` | done | owned by `bootstrap-and-commands`; public Vibe binary remains `vibe` |
| `packages/happy-cli/bin/happy-dev.mjs` | packaged dev-only CLI wrapper | `crates/vibe-cli` | `main.rs` | done | compatibility-only dev entrypoint maps to the same command bootstrap surface |
| `packages/happy-cli/bin/happy-mcp.mjs` | packaged MCP/stdin bridge wrapper | `crates/vibe-cli` | `main.rs` | done | compatibility-only wrapper stays owned by CLI bootstrap until the MCP path is ported |
| `packages/happy-cli/src/lib.ts` | reusable CLI bootstrap helpers | `crates/vibe-cli` | `bootstrap.rs` | done | isolate reusable bootstrap from binary main |
| `packages/happy-cli/src/configuration.ts` | global CLI configuration bootstrap | `crates/vibe-cli` | `config.rs` | done | owned by `bootstrap-and-commands` plan |
| `packages/happy-cli/src/projectPath.ts` | project path resolution | `crates/vibe-cli` | `config.rs` | done | bootstrap path helper |
| `packages/happy-cli/src/commands/*` | top-level CLI command handlers | `crates/vibe-cli` | `commands/` | done | owned by `bootstrap-and-commands` plan |
| `packages/happy-cli/src/commands/connect/*` | CLI provider-connect command entrypoints | `crates/vibe-cli` | `auth.rs` | done | command parsing is rooted in bootstrap, but connect semantics belong to CLI auth |
| `packages/happy-cli/src/agent/index.ts` | provider registry entrypoint | `crates/vibe-cli` | `agent/registry.rs` | done | re-export and initialization surface for provider registration |
| `packages/happy-cli/src/agent/core/*` | provider-agnostic runtime abstractions | `crates/vibe-cli` | `agent/core.rs` | done | stable backend trait and runtime event model |
| `packages/happy-cli/src/agent/factories/*` | provider factory wiring | `crates/vibe-cli` | `agent/registry.rs` | done | provider factory resolution belongs with agent core |
| `packages/happy-cli/src/agent/adapters/*` | provider-native normalization helpers | `crates/vibe-cli` | `agent/adapters/` | done | normalize provider events before wire mapping |
| `packages/happy-cli/src/agent/transport/*` | runtime-to-server transport layer | `crates/vibe-cli` | `transport/` | done | async buffered transport boundary |
| `packages/happy-cli/src/agent/acp/*` | ACP-specific runtime/session support | `crates/vibe-cli` | `agent/acp/` | done | first-class provider path, not an afterthought in generic transport |
| `packages/happy-cli/src/api/api.ts` | API client root exports | `crates/vibe-cli` | `api.rs` | done | centralized server communication entrypoint |
| `packages/happy-cli/src/api/apiSession.ts` | session HTTP helpers | `crates/vibe-cli` | `api.rs` | done | session create/send/list/history flows live in the unified CLI API client |
| `packages/happy-cli/src/api/apiMachine.ts` | machine HTTP helpers | `crates/vibe-cli` | `api.rs` | done | machine registration/detail helpers live in the unified CLI API client |
| `packages/happy-cli/src/api/rpc/*` | RPC request/response helpers | `crates/vibe-cli` | `api.rs` | done | runtime RPC transport surface lives in the unified CLI API client |
| `packages/happy-cli/src/api/types.ts` | shared CLI API DTO helpers | `crates/vibe-cli` | `api.rs` | done | keep API DTO helpers under the centralized client tree |
| `packages/happy-cli/src/api/pushNotifications.ts` | push-token API helpers | `crates/vibe-cli` | `api.rs` | done | supporting API surface stays inside the central client module |
| `packages/happy-cli/src/api/auth.ts` | CLI auth API flows | `crates/vibe-cli` | `auth.rs` | done | local auth/connect semantics live outside the generic API client |
| `packages/happy-cli/src/api/webAuth.ts` | web auth callback/login helpers | `crates/vibe-cli` | `auth.rs` | done | auth-specific browser flow stays under CLI auth |
| `packages/happy-cli/src/daemon/*` | daemon control plane | `crates/vibe-cli` | `daemon.rs` | done | own local process management |
| `packages/happy-cli/src/modules/*` | bundled local helper binaries/modules | `crates/vibe-cli` | `modules/` | done | owned by `builtin-modules`, not by generic utils |
| `packages/happy-cli/src/codex/utils/sessionProtocolMapper.ts` | Codex-specific wire mapping rules | `crates/vibe-cli` | `session_protocol_mapper.rs` | done | explicit exception to provider-local ownership; this file defines canonical mapping behavior |
| `packages/happy-cli/src/claude/utils/sessionProtocolMapper.ts` | Claude-specific wire mapping rules | `crates/vibe-cli` | `session_protocol_mapper.rs` | done | shared mapper owns wire-shape decisions across providers |
| `packages/happy-cli/src/codex/*` | Codex runtime | `crates/vibe-cli` | `providers/codex/` | done | provider-local runtime except the dedicated session-protocol mapper files above |
| `packages/happy-cli/src/claude/*` | Claude runtime | `crates/vibe-cli` | `providers/claude.rs` | done | keep adapters local to provider except shared mapping seams |
| `packages/happy-cli/src/gemini/*` | Gemini runtime | `crates/vibe-cli` | `providers/gemini/` | done | keep parity with Happy behavior |
| `packages/happy-cli/src/openclaw/*` | OpenClaw runtime | `crates/vibe-cli` | `providers/openclaw/` | done | separate provider module |
| `packages/happy-cli/src/parsers/*` | CLI-local parser helpers | `crates/vibe-cli` | `utils/parsing.rs` | done | owned by `utils-and-parsers` |
| `packages/happy-cli/src/sessionProtocol/types.ts` | CLI-local session protocol helper types | `crates/vibe-cli` | `session_protocol_mapper.rs` | done | keep CLI-local mapper types out of wire crate unless promoted intentionally |
| `packages/happy-cli/src/sandbox/*` | sandbox manager | `crates/vibe-cli` | `sandbox.rs` | done | implementation uses the single sandbox module in the current CLI layout |
| `packages/happy-cli/src/persistence.ts` | local persistence | `crates/vibe-cli` | `persistence.rs` | done | resume support depends on this |
| `packages/happy-cli/src/resume/*` | resume logic | `crates/vibe-cli` | `resume.rs` | done | keep separate from persistence core in the current flat-file CLI layout |
| `packages/happy-cli/src/test-setup.ts` | CLI test bootstrap helpers | `crates/vibe-cli` | `tests/` | done | pair with `testing-fixtures` plan |
| `packages/happy-cli/src/testing/*` | reusable CLI test scaffolding | `crates/vibe-cli` | `tests/fixtures/` | done | integration harness and fixtures |
| `packages/happy-cli/src/ui/*` | terminal UX | `crates/vibe-cli` | `ui/` | done | choose Rust TUI only inside locked module plans |
| `packages/happy-cli/src/utils/*` | shared CLI helper utilities | `crates/vibe-cli` | `utils/` | done | internal helpers and system adapters |
| `packages/happy-cli/scripts/*` | CLI helper scripts for launch, local development, and tool bootstrap | `crates/vibe-cli` | `bootstrap.rs` | done | explicit owner existed before Wave 5; helper bootstrap behavior now lives in the CLI bootstrap module |
| `packages/happy-cli/tools/*` | packaged helper tool archives and license payloads | `crates/vibe-cli` | `modules/` | done | owned by `builtin-modules`; download/unpack behavior stays aligned with the same module plan |
| `packages/happy-app/package.json` | app package metadata and scripts | `packages/vibe-app` | package metadata | done | imported by `import-and-build`; script/env renames landed through `release-and-env` |
| `packages/happy-app/index.ts` | app package entrypoint | `packages/vibe-app` | imported app tree | done | baseline import owner is `import-and-build` |
| `packages/happy-app/app.config.js` | Expo app config and identifiers | `packages/vibe-app` | app config | done | owned by `release-and-env` and `branding-and-naming-adaptation` |
| `packages/happy-app/babel.config.js` | app build-tool bootstrap | `packages/vibe-app` | imported app build config | done | imported and kept package-local |
| `packages/happy-app/metro.config.js` | Metro bundler bootstrap | `packages/vibe-app` | imported app build config | done | imported and kept package-local |
| `packages/happy-app/release.cjs` | release script entrypoint | `packages/vibe-app` | release/env scripts | done | owned by `release-and-env` |
| `packages/happy-app/release-*.sh` | release shell helpers | `packages/vibe-app` | release/env scripts | done | owned by `release-and-env` |
| `packages/happy-app/src-tauri/**` | desktop shell, config, and bundle metadata | `packages/vibe-app` | `src-tauri/` | done | owned by `desktop-tauri-adaptation` |
| `packages/happy-app/src-tauri/**` | desktop shell behavior reference for the active replacement package | `packages/vibe-app-tauri` | `universal-bootstrap-and-runtime` / `desktop-shell-and-platform-parity` | planned | reuse behavior as reference only; do not mutate the existing `packages/vibe-app/src-tauri` path while bootstrapping the active Wave 9 package |
| `packages/happy-app/index.ts` | Happy app runtime entrypoint and bootstrap wiring reference | `packages/vibe-app-tauri` | `universal-bootstrap-and-runtime` | planned | continuity input only; Wave 9 canonical runtime entrypoints live in package-local Tauri desktop, Android, and static-export bootstrap files |
| `packages/happy-app/app.config.js` | Happy Expo/mobile release config reference | `packages/vibe-app-tauri` | `universal-bootstrap-and-runtime` / `release-ota-and-store-migration` | planned | continuity input only; do not treat Expo config as the canonical Wave 9 runtime or release boundary |
| `packages/happy-app/eas.json` | Happy EAS build profile reference | `packages/vibe-app-tauri` | `release-ota-and-store-migration` | planned | continuity input only; Wave 9 canonical release inputs are package-local Android, desktop, and static-export configs |
| `packages/happy-app/release.cjs` | Happy release automation reference | `packages/vibe-app-tauri` | `release-ota-and-store-migration` | planned | continuity input only; active release automation must target GitHub Releases and APK-first Android distribution |
| `packages/happy-app/release-*.sh` | Happy release helper script reference | `packages/vibe-app-tauri` | `release-ota-and-store-migration` | planned | continuity input only; Wave 9 release helpers are package-local and Tauri-aligned |
| `packages/happy-app/sources/` | app sources | `packages/vibe-app` | imported tree | done | imported baseline plus Vibe adaptations landed |
| `packages/happy-app/sources/` | app route, state, and component reference for the active replacement package | `packages/vibe-app-tauri` | `modules/vibe-app-tauri/*` | planned | extraction during `vibe-app-tauri` lands package-local first; broad cross-package sharing is deferred until the unified replacement proves parity |
| `packages/happy-app/sources/app/**` | router, entry screens, and top-level app flows | `packages/vibe-app` | imported app tree | done | imported first; public naming cleanup landed through `branding-and-naming-adaptation` |
| `packages/happy-app/sources/app/**` | route and screen reference for the active replacement package | `packages/vibe-app-tauri` | `mobile-shell-and-navigation` / `desktop-shell-and-platform-parity` / `session-rendering-and-composer` / `secondary-routes-and-social` | planned | use the Wave 9 route matrix as the active parity baseline; historical desktop inventories are continuity-only references |
| `packages/happy-app/sources/auth/**` | app-side auth UX and deep-link entrypoints | `packages/vibe-app` | imported auth tree | done | endpoint and naming changes route through `api-endpoint-adaptation` and `branding-and-naming-adaptation` |
| `packages/happy-app/sources/auth/**` | auth/account restore reference and extraction source for the active replacement package | `packages/vibe-app-tauri` | `auth-and-identity-flows` | planned | logic lands package-local first; secure storage and callback handling are reworked behind explicit platform adapters |
| `packages/happy-app/sources/config.ts` | app config/env resolution helpers | `packages/vibe-app` | config/env seam | done | owned by `api-endpoint-adaptation` and finalized by `release-and-env` |
| `packages/happy-app/sources/config.ts` | config/env reference for the active replacement package | `packages/vibe-app-tauri` | `universal-bootstrap-and-runtime` | planned | replacement bootstrap may reuse concepts but should not inherit deprecated package assumptions directly |
| `packages/happy-app/sources/realtime/**` | live update subscription and realtime glue | `packages/vibe-app` | realtime adapter seam | done | endpoint/socket changes landed through `api-endpoint-adaptation` |
| `packages/happy-app/sources/realtime/**` | realtime/session subscription reference for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `session-runtime-and-storage` | planned | realtime behavior must support desktop, Android, and retained static export ownership without forking protocol semantics |
| `packages/happy-app/sources/sync/**` | sync engine, reducers, storage, and parser entrypoints | `packages/vibe-app` | sync/protocol seam | done | protocol parsing belongs to `protocol-parser-compat`; API wiring belongs to `api-endpoint-adaptation` |
| `packages/happy-app/sources/sync/**` | sync/domain extraction source for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `session-runtime-and-storage` | planned | sync/domain logic lands package-local first and must be split from old mobile-host assumptions explicitly |
| `packages/happy-app/sources/encryption/**` | app-side encryption helpers | `packages/vibe-app` | imported encryption tree | done | keep imported first; do not fork protocol rules outside the shared crypto spec |
| `packages/happy-app/sources/encryption/**` | encryption helper reference for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` | planned | may be copied/adapted into the new package if it can be detached from mobile runtime assumptions |
| `packages/happy-app/sources/components/**` | reusable UI components | `packages/vibe-app` | imported component tree | done | imported baseline; branding-visible changes landed through dedicated app adaptation plans |
| `packages/happy-app/sources/components/**` | UI parity reference and rewrite source for the active replacement package | `packages/vibe-app-tauri` | `mobile-shell-and-navigation` / `desktop-shell-and-platform-parity` / `session-rendering-and-composer` / `secondary-routes-and-social` | planned | component behavior is a parity reference; new implementations may differ by platform shell while preserving Happy semantics |
| `packages/happy-app/sources/modal/**` | modal UI flows | `packages/vibe-app` | imported component tree | done | imported baseline |
| `packages/happy-app/sources/modal/**` | overlay/modal reference for the active replacement package | `packages/vibe-app-tauri` | `desktop-shell-and-platform-parity` / `mobile-shell-and-navigation` / `web-export-and-browser-runtime` | planned | modal semantics should be preserved while adapting focus and browser/mobile behavior explicitly |
| `packages/happy-app/sources/hooks/**` | app hooks and local state helpers | `packages/vibe-app` | imported hook tree | done | imported baseline |
| `packages/happy-app/sources/hooks/**` | state and interaction helper reference for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `session-rendering-and-composer` | planned | only hooks without hard UI-host coupling should be ported directly |
| `packages/happy-app/sources/utils/**` | app-local utilities and platform helpers | `packages/vibe-app` | imported utility tree | done | keep local to the app package; avoid promoting these into shared Rust contracts |
| `packages/happy-app/sources/utils/**` | utility and platform-adapter reference for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `desktop-shell-and-platform-parity` / `mobile-native-capabilities` / `release-ota-and-store-migration` | planned | pure TS utilities may be copied; platform helpers must be rewritten behind explicit Wave 9 adapters |
| `packages/happy-app/sources/text/**` | translations and public strings | `packages/vibe-app` | text/localization tree | done | public-surface changes landed through `branding-and-naming-adaptation` |
| `packages/happy-app/sources/text/**` | text/localization source for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `secondary-routes-and-social` | planned | text assets may be mirrored package-local to keep the old app untouched during early phases |
| `packages/happy-app/sources/constants/**` | app constants and static config values | `packages/vibe-app` | imported constant tree | done | import baseline first; env/public constant cleanup landed later |
| `packages/happy-app/sources/constants/**` | constant/token reference for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `mobile-shell-and-navigation` | planned | copy only the constants that support parity and avoid pulling deprecated mobile-host assumptions forward |
| `packages/happy-app/sources/assets/**` | fonts, images, animations, and bundled visual assets | `packages/vibe-app` | imported asset tree | done | imported baseline keeps the current packaged asset set |
| `packages/happy-app/sources/assets/**` | asset parity source for the active replacement package | `packages/vibe-app-tauri` | `universal-bootstrap-and-runtime` / `mobile-shell-and-navigation` / `secondary-routes-and-social` | planned | copy or mirror only the assets required for parity; keep package-local ownership and build outputs distinct from `packages/vibe-app` |
| `packages/happy-app/sources/track/**` | analytics and screen tracking hooks | `packages/vibe-app` | imported tracking tree | done | imported baseline; renamed only where public Vibe branding required it |
| `packages/happy-app/sources/track/**` | tracking/analytics decision point for the active replacement package | `packages/vibe-app-tauri` | `release-ota-and-store-migration` | planned | track ownership stays explicit even if telemetry is deferred during early parity work |
| `packages/happy-app/sources/types/**` | app-local TypeScript type helpers and shims | `packages/vibe-app` | imported type tree | done | app-local type ownership stays inside the imported package unless promoted deliberately |
| `packages/happy-app/sources/types/**` | TS shim/type-helper source for the active replacement package | `packages/vibe-app-tauri` | `shared-core-from-happy` / `universal-bootstrap-and-runtime` | planned | package-local declaration files may be copied as needed without creating a shared package |
| `packages/happy-app-logs/src/server.ts` | app log sidecar runtime | `crates/vibe-app-logs` | `server.rs` | done | `config.rs`, `server.rs`, and `main.rs` now back the app-facing `/logs` runtime plus root `yarn app-logs` launch flow |
| `package.json` | root Yarn workspace and bootstrap scripts | repository root | temporary app bootstrap files | done | imported minimum root workspace metadata now also carries the public `yarn app-logs` helper |
| `yarn.lock` | root workspace lockfile for imported app bootstrap | repository root | temporary app bootstrap files | done | explicit root lockfile remains required for the imported app workspace bootstrap |
| `scripts/postinstall.cjs` | root postinstall patching and wire build | repository root | bootstrap install seam | done | imported and localized so it no longer assumes a missing Happy workspace dependency tree |
| `scripts/release.cjs` | root release/bootstrap helper used by imported app scripts | `packages/vibe-app` | release/env scripts | done | import audit closed this conditional path without adopting the root helper; release ownership is localized to the package-local `release.cjs` entrypoint |
| `environments/environments.ts` | shared environment manifest referenced by imported app/release flows | `packages/vibe-app` | config/env seam | done | import audit confirmed no standalone root environment manifest was required; env tiers are defined by package-local app config and scripts |
| `patches/fix-pglite-prisma-bytes.cjs` | node_modules patch required by app install | `patches/` | imported patch file | done | retained because the app bootstrap still depends on the patch during install |

## Migration Rule

If an implementation task touches a Happy source that is not represented in this table, update this
file first and point the new entry at a concrete Vibe target module.
