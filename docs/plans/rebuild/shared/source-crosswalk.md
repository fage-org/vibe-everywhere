# Happy To Vibe Source Crosswalk

## Package-Level Mapping

| Happy source | Happy responsibility | Vibe target | Target module root | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `packages/happy-wire` | shared message schemas and protocol helpers | `crates/vibe-wire` | `src/` | planned | canonical Rust source for all shared contracts |
| `packages/happy-server` | backend APIs, socket updates, storage, auth | `crates/vibe-server` | `src/` | planned | server rewrite depends on `vibe-wire` contracts |
| `packages/happy-agent` | remote-control CLI client | `crates/vibe-agent` | `src/` | planned | session and machine control client |
| `packages/happy-cli` | local runtime, daemon, provider integrations | `crates/vibe-cli` | `src/` | planned | largest subsystem, start only after wire and server spine |
| `packages/happy-app` | mobile/web/desktop app | `packages/vibe-app` | imported Happy tree | planned | import first, adapt later |
| `packages/happy-app-logs` | log sidecar | `crates/vibe-app-logs` | `src/` | planned | lowest priority subsystem |

## Coverage Conventions

- `*.test.ts`, `*.spec.ts`, and other test-only files inherit the owner of the source module they
  validate; they do not create standalone Vibe module targets.
- Phase-one `packages/vibe-app` keeps the imported Happy tree mostly intact. App paths are mapped to
  dedicated adaptation seams only where a `modules/vibe-app/*.md` plan already exists.
- Helper-only files must stay with the nearest owning subsystem plan. Do not create new top-level
  `utils` modules in Vibe unless a later plan records that boundary change first.
- Critical-path HTTP route entrypoints and socket handler entrypoints must be listed explicitly in
  this table even if a broader wildcard entry already exists elsewhere.

## High-Value Module Mapping

| Happy source path | Happy responsibility | Vibe target crate/package | Vibe target module | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `packages/happy-wire/src/index.ts` | package export surface and type barrel | `crates/vibe-wire` | `lib.rs` | planned | re-export only; public contracts stay canonical in `vibe-wire` |
| `packages/happy-wire/src/messages.ts` | wire message containers | `crates/vibe-wire` | `messages.rs` | planned | define encrypted message and update containers |
| `packages/happy-wire/src/sessionProtocol.ts` | session envelope and event union | `crates/vibe-wire` | `session_protocol.rs` | planned | primary protocol source of truth |
| `packages/happy-wire/src/legacyProtocol.ts` | legacy user/agent content wrapper | `crates/vibe-wire` | `legacy_protocol.rs` | planned | required for app compatibility |
| `packages/happy-wire/src/messageMeta.ts` | message metadata | `crates/vibe-wire` | `message_meta.rs` | planned | shared by CLI, agent, app, server |
| `packages/happy-wire/src/voice.ts` | voice token schema | `crates/vibe-wire` | `voice.rs` | planned | add serde + validation |
| `packages/happy-server/sources/main.ts` | primary server binary bootstrap | `crates/vibe-server` | `main.rs` | planned | owned by `versions-and-config`; no separate bootstrap layer outside typed config/context |
| `packages/happy-server/sources/standalone.ts` | alternate standalone startup path | `crates/vibe-server` | `main.rs` | planned | folds into the same startup/bootstrap module as `main.ts` |
| `packages/happy-server/sources/versions.ts` | version/build metadata helpers | `crates/vibe-server` | `version.rs` | planned | owned by `versions-and-config` |
| `packages/happy-server/sources/context.ts` | shared server context construction | `crates/vibe-server` | `context.rs` | planned | typed app context for router and domain services |
| `packages/happy-server/sources/types.ts` | shared server DTOs used across API/domain layers | `crates/vibe-server` | `api/types.rs` | planned | shared account/artifact/GitHub-adjacent API shapes live with `app-api` |
| `packages/happy-server/sources/app/api/api.ts` | top-level HTTP router bootstrap | `crates/vibe-server` | `api/mod.rs` | planned | owned by `app-api` pass A/B |
| `packages/happy-server/sources/app/api/types.ts` | HTTP-layer DTOs and shared API type helpers | `crates/vibe-server` | `api/types.rs` | planned | router-facing types owned by `app-api` and reused by GitHub/account routes |
| `packages/happy-server/sources/app/api/socket.ts` | Socket.IO transport bootstrap and connection scope handling | `crates/vibe-server` | `api/socket.rs` | planned | transport entrypoint owned by `socket-updates`; domain writes still live in their owning modules |
| `packages/happy-server/sources/app/api/utils/*` | API bootstrap helpers for auth, error handling, and monitoring | `crates/vibe-server` | `api/mod.rs` | planned | owned by `app-api` bootstrap rather than standalone feature modules |
| `packages/happy-server/sources/app/auth/*` | auth endpoints and guards | `crates/vibe-server` | `auth/` | planned | driven by shared auth spec |
| `packages/happy-server/sources/app/events/eventRouter.ts` | update/event routing | `crates/vibe-server` | `events/router.rs` | planned | socket + fanout hub |
| `packages/happy-server/sources/app/api/routes/authRoutes.ts` | auth and account-link HTTP route registration | `crates/vibe-server` | `auth/` | planned | HTTP auth route glue stays with the auth module; `app-api` only mounts the route group |
| `packages/happy-server/sources/app/api/routes/accountRoutes.ts` | account profile, settings, and usage routes | `crates/vibe-server` | `api/account.rs` | planned | own account-and-usage surface |
| `packages/happy-server/sources/app/api/routes/connectRoutes.ts` | generic vendor connect routes and GitHub route registration | `crates/vibe-server` | `api/connect.rs` | planned | split generic vendor logic from GitHub-specific behavior |
| `packages/happy-server/sources/app/api/routes/devRoutes.ts` | optional AI-debug logging endpoint | `crates/vibe-server` | `api/dev.rs` | deferred | debug-only route; exclude from parity unless a concrete consumer requires it |
| `packages/happy-server/sources/app/api/routes/artifactsRoutes.ts` | artifact CRUD routes | `crates/vibe-server` | `api/artifacts.rs` | planned | pair with socket artifact handlers |
| `packages/happy-server/sources/app/api/routes/accessKeysRoutes.ts` | access-key CRUD routes | `crates/vibe-server` | `api/access_keys.rs` | planned | session/machine access-key surface |
| `packages/happy-server/sources/app/api/routes/sessionRoutes.ts` | legacy `/v1` and `/v2` session CRUD, list, and history routes | `crates/vibe-server` | `sessions/http.rs` | planned | session module owns create-or-load-by-tag, list, delete, and legacy history HTTP behavior; `app-api` only mounts it |
| `packages/happy-server/sources/app/api/routes/v3SessionRoutes.ts` | paged and idempotent v3 session message routes | `crates/vibe-server` | `sessions/http.rs` | planned | keep v1/v2 and v3 session HTTP semantics in the same session module so message/history contracts do not drift |
| `packages/happy-server/sources/app/api/routes/kvRoutes.ts` | key-value support routes | `crates/vibe-server` | `api/kv.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/pushRoutes.ts` | push token registration routes | `crates/vibe-server` | `api/push.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/versionRoutes.ts` | version check route | `crates/vibe-server` | `api/version.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/voiceRoutes.ts` | voice token route | `crates/vibe-server` | `api/voice.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/feedRoutes.ts` | feed route registration and response DTO glue | `crates/vibe-server` | `feed/http.rs` | planned | feed module owns the route surface and DTO shaping; `app-api` only mounts it |
| `packages/happy-server/sources/app/api/routes/userRoutes.ts` | user profile/search and friend route registration | `crates/vibe-server` | `social/http.rs` | planned | social module owns user and friend HTTP surfaces plus relationship-aware DTO shaping |
| `packages/happy-server/sources/app/api/socket/pingHandler.ts` | transport health/ping socket handler | `crates/vibe-server` | `api/socket.rs` | planned | transport-local helper; keep with socket bootstrap instead of creating a separate ping module |
| `packages/happy-server/sources/app/api/socket/sessionUpdateHandler.ts` | inbound session socket events and acks | `crates/vibe-server` | `api/socket.rs` | planned | transport entrypoint delegates durable state changes to `session-lifecycle` |
| `packages/happy-server/sources/app/api/socket/rpcHandler.ts` | machine RPC socket forwarding | `crates/vibe-server` | `api/socket.rs` | planned | socket transport owns forwarding and ack framing; machine behavior stays in `machine-lifecycle` |
| `packages/happy-server/sources/app/session/*` | session lifecycle helpers and deletion logic | `crates/vibe-server` | `sessions/` | planned | session CRUD and lifecycle module |
| `packages/happy-server/sources/app/feed/*` | feed domain logic | `crates/vibe-server` | `feed/` | planned | imported app feed behavior |
| `packages/happy-server/sources/app/social/*` | social and friend domain logic | `crates/vibe-server` | `social/` | planned | friend, relationship, username flows |
| `packages/happy-server/sources/app/github/*` | GitHub integration domain logic | `crates/vibe-server` | `github/` | planned | GitHub-specific connect/disconnect/profile logic |
| `packages/happy-server/sources/app/monitoring/*` | metrics and monitoring hooks | `crates/vibe-server` | `monitoring/` | planned | service metrics and health instrumentation |
| `packages/happy-server/sources/app/api/routes/machinesRoutes.ts` | machine create/list/detail routes | `crates/vibe-server` | `machines/http.rs` | planned | machine CRUD and encrypted machine records |
| `packages/happy-server/sources/app/api/socket/machineUpdateHandler.ts` | machine heartbeat and optimistic concurrency updates | `crates/vibe-server` | `machines/socket.rs` | planned | machine-alive and machine-update-* socket handlers |
| `packages/happy-server/sources/app/api/socket/artifactUpdateHandler.ts` | artifact socket read/update/create/delete | `crates/vibe-server` | `api/socket_artifacts.rs` | planned | artifact socket ack/result compatibility |
| `packages/happy-server/sources/app/api/socket/accessKeyHandler.ts` | access-key socket lookup | `crates/vibe-server` | `api/socket_access_keys.rs` | planned | socket access-key retrieval |
| `packages/happy-server/sources/app/api/socket/usageHandler.ts` | usage-report socket helper surface | `crates/vibe-server` | `api/socket.rs` | planned | pass-B auxiliary socket API owned jointly by `socket-updates` transport and `account-and-usage` service |
| `packages/happy-server/sources/app/presence/sessionCache.ts` | session/machine validation cache and batched activeAt flush | `crates/vibe-server` | `presence/cache.rs` | planned | lock 30s TTL, 30s threshold, 5s flush interval |
| `packages/happy-server/sources/app/presence/timeout.ts` | session/machine inactivity timeout sweeper | `crates/vibe-server` | `presence/timeout.rs` | planned | lock 10 minute timeout and 1 minute sweep |
| `packages/happy-server/sources/app/kv/*` | KV business logic helpers | `crates/vibe-server` | `api/kv.rs` | planned | keep KV ownership explicit outside generic router setup |
| `packages/happy-server/sources/storage/db.ts` | primary relational storage bootstrap | `crates/vibe-server` | `storage/db.rs` | planned | relational persistence owner |
| `packages/happy-server/sources/storage/inTx.ts` | transaction wrapper helpers | `crates/vibe-server` | `storage/tx.rs` | planned | stays with relational storage, not a generic server helper layer |
| `packages/happy-server/sources/storage/seq.ts` | monotonic sequence allocation helpers | `crates/vibe-server` | `storage/seq.rs` | planned | used by sessions/events through `storage-db` |
| `packages/happy-server/sources/storage/redis.ts` | Redis-backed cache/queue integration | `crates/vibe-server` | `storage/redis.rs` | planned | cache and fanout support owned by `storage-redis` |
| `packages/happy-server/sources/storage/files.ts` | object storage accessors and file refs | `crates/vibe-server` | `storage/files.rs` | planned | primary file/blob storage module |
| `packages/happy-server/sources/storage/types.ts` | file/image reference types | `crates/vibe-server` | `storage/types.rs` | planned | shared file-storage DTOs stay with `storage-files` |
| `packages/happy-server/sources/storage/uploadImage.ts` | storage-layer upload composition for images | `crates/vibe-server` | `storage/files.rs` | planned | keep upload orchestration with file storage; image transforms are layered separately |
| `packages/happy-server/sources/storage/repeatKey.ts` | repeated-upload dedupe helper | `crates/vibe-server` | `storage/files.rs` | planned | file-storage helper, not its own subsystem |
| `packages/happy-server/sources/storage/simpleCache.ts` | storage-local cache helper | `crates/vibe-server` | `storage/redis.rs` | planned | keep local cache helpers inside the cache/storage boundary |
| `packages/happy-server/sources/storage/pgliteLoader.ts` | dev/test local DB bootstrap helper | `crates/vibe-server` | `storage/db.rs` | planned | stays with relational storage bootstrap |
| `packages/happy-server/sources/storage/processImage.ts` | image normalization pipeline | `crates/vibe-server` | `storage/process_image.rs` | planned | owned by `image-processing` |
| `packages/happy-server/sources/storage/thumbhash.ts` | thumbhash placeholder helper | `crates/vibe-server` | `storage/thumbhash.rs` | planned | owned by `image-processing` |
| `packages/happy-server/sources/modules/encrypt.ts` | server-local symmetric encryption bootstrap | `crates/vibe-server` | `auth/` | planned | keep server-owned secret handling with auth/config bootstrap; do not fork shared crypto rules elsewhere |
| `packages/happy-server/sources/modules/github.ts` | GitHub app/webhook bootstrap helpers | `crates/vibe-server` | `github/` | planned | provider bootstrap stays with the GitHub integration module |
| `packages/happy-server/sources/utils/*` | crate-local helper functions | `crates/vibe-server` | caller-owned helper files | planned | helpers stay with the nearest subsystem under `config`, `api`, `events`, `storage`, or domain modules; no standalone `utils` target |
| `packages/happy-agent/src/config.ts` | env and home-path resolution | `crates/vibe-agent` | `config.rs` | planned | foundational config/bootstrap module |
| `packages/happy-agent/src/auth.ts` | account auth flow | `crates/vibe-agent` | `auth.rs` | planned | QR auth and token persistence |
| `packages/happy-agent/src/credentials.ts` | local credential storage | `crates/vibe-agent` | `credentials.rs` | planned | stored under `~/.vibe` |
| `packages/happy-agent/src/encryption.ts` | crypto helpers | `crates/vibe-agent` | `encryption.rs` | planned | may share internals with `vibe-wire` |
| `packages/happy-agent/src/api.ts` | REST session and machine client | `crates/vibe-agent` | `api.rs` | planned | direct server control path |
| `packages/happy-agent/src/session.ts` | Socket.IO session client | `crates/vibe-agent` | `session.rs` | planned | live updates and wait logic |
| `packages/happy-agent/src/machineRpc.ts` | machine-scoped RPC | `crates/vibe-agent` | `machine_rpc.rs` | planned | remote runtime control path |
| `packages/happy-agent/src/output.ts` | human-readable and JSON CLI formatting | `crates/vibe-agent` | `output.rs` | planned | owned by `cli-output` |
| `packages/happy-agent/src/index.ts` | binary entrypoint and command wiring | `crates/vibe-agent` | `main.rs` | planned | top-level CLI wiring stays with `cli-output`, not with transport or API modules |
| `packages/happy-cli/src/index.ts` | top-level CLI entrypoint | `crates/vibe-cli` | `main.rs` | planned | owned by `bootstrap-and-commands` plan |
| `packages/happy-cli/src/lib.ts` | reusable CLI bootstrap helpers | `crates/vibe-cli` | `bootstrap.rs` | planned | isolate reusable bootstrap from binary main |
| `packages/happy-cli/src/configuration.ts` | global CLI configuration bootstrap | `crates/vibe-cli` | `config.rs` | planned | owned by `bootstrap-and-commands` plan |
| `packages/happy-cli/src/projectPath.ts` | project path resolution | `crates/vibe-cli` | `config.rs` | planned | bootstrap path helper |
| `packages/happy-cli/src/commands/*` | top-level CLI command handlers | `crates/vibe-cli` | `commands/` | planned | owned by `bootstrap-and-commands` plan |
| `packages/happy-cli/src/commands/connect/*` | CLI provider-connect command entrypoints | `crates/vibe-cli` | `auth/` | planned | command parsing is rooted in bootstrap, but connect semantics belong to CLI auth |
| `packages/happy-cli/src/agent/index.ts` | provider registry entrypoint | `crates/vibe-cli` | `agent/registry.rs` | planned | re-export and initialization surface for provider registration |
| `packages/happy-cli/src/agent/core/*` | provider-agnostic runtime abstractions | `crates/vibe-cli` | `agent/core.rs` | planned | stable backend trait and runtime event model |
| `packages/happy-cli/src/agent/factories/*` | provider factory wiring | `crates/vibe-cli` | `agent/registry.rs` | planned | provider factory resolution belongs with agent core |
| `packages/happy-cli/src/agent/adapters/*` | provider-native normalization helpers | `crates/vibe-cli` | `agent/adapters/` | planned | normalize provider events before wire mapping |
| `packages/happy-cli/src/agent/transport/*` | runtime-to-server transport layer | `crates/vibe-cli` | `transport/` | planned | async buffered transport boundary |
| `packages/happy-cli/src/agent/acp/*` | ACP-specific runtime/session support | `crates/vibe-cli` | `agent/acp/` | planned | first-class provider path, not an afterthought in generic transport |
| `packages/happy-cli/src/api/api.ts` | API client root exports | `crates/vibe-cli` | `api/mod.rs` | planned | centralized server communication entrypoint |
| `packages/happy-cli/src/api/apiSession.ts` | session HTTP helpers | `crates/vibe-cli` | `api/session.rs` | planned | session create/send/list/history flows |
| `packages/happy-cli/src/api/apiMachine.ts` | machine HTTP helpers | `crates/vibe-cli` | `api/machine.rs` | planned | machine registration/detail helpers |
| `packages/happy-cli/src/api/rpc/*` | RPC request/response helpers | `crates/vibe-cli` | `api/rpc.rs` | planned | runtime RPC transport surface |
| `packages/happy-cli/src/api/types.ts` | shared CLI API DTO helpers | `crates/vibe-cli` | `api/mod.rs` | planned | keep API DTO helpers under the centralized client tree |
| `packages/happy-cli/src/api/pushNotifications.ts` | push-token API helpers | `crates/vibe-cli` | `api/mod.rs` | planned | supporting API surface stays inside the central client module |
| `packages/happy-cli/src/api/auth.ts` | CLI auth API flows | `crates/vibe-cli` | `auth/` | planned | local auth/connect semantics live outside the generic API client |
| `packages/happy-cli/src/api/webAuth.ts` | web auth callback/login helpers | `crates/vibe-cli` | `auth/` | planned | auth-specific browser flow stays under CLI auth |
| `packages/happy-cli/src/daemon/*` | daemon control plane | `crates/vibe-cli` | `daemon/` | planned | own local process management |
| `packages/happy-cli/src/modules/*` | bundled local helper binaries/modules | `crates/vibe-cli` | `modules/` | planned | owned by `builtin-modules`, not by generic utils |
| `packages/happy-cli/src/codex/utils/sessionProtocolMapper.ts` | Codex-specific wire mapping rules | `crates/vibe-cli` | `session_protocol_mapper.rs` | planned | explicit exception to provider-local ownership; this file defines canonical mapping behavior |
| `packages/happy-cli/src/claude/utils/sessionProtocolMapper.ts` | Claude-specific wire mapping rules | `crates/vibe-cli` | `session_protocol_mapper.rs` | planned | shared mapper owns wire-shape decisions across providers |
| `packages/happy-cli/src/codex/*` | Codex runtime | `crates/vibe-cli` | `providers/codex/` | planned | provider-local runtime except the dedicated session-protocol mapper files above |
| `packages/happy-cli/src/claude/*` | Claude runtime | `crates/vibe-cli` | `providers/claude/` | planned | keep adapters local to provider except shared mapping seams |
| `packages/happy-cli/src/gemini/*` | Gemini runtime | `crates/vibe-cli` | `providers/gemini/` | planned | keep parity with Happy behavior |
| `packages/happy-cli/src/openclaw/*` | OpenClaw runtime | `crates/vibe-cli` | `providers/openclaw/` | planned | separate provider module |
| `packages/happy-cli/src/parsers/*` | CLI-local parser helpers | `crates/vibe-cli` | `parsers/` | planned | owned by `utils-and-parsers` |
| `packages/happy-cli/src/sessionProtocol/types.ts` | CLI-local session protocol helper types | `crates/vibe-cli` | `session_protocol_mapper.rs` | planned | keep CLI-local mapper types out of wire crate unless promoted intentionally |
| `packages/happy-cli/src/sandbox/*` | sandbox manager | `crates/vibe-cli` | `sandbox/` | planned | implementation may use Rust process isolation primitives |
| `packages/happy-cli/src/persistence.ts` | local persistence | `crates/vibe-cli` | `persistence/` | planned | resume support depends on this |
| `packages/happy-cli/src/resume/*` | resume logic | `crates/vibe-cli` | `resume/` | planned | keep separate from persistence core |
| `packages/happy-cli/src/test-setup.ts` | CLI test bootstrap helpers | `crates/vibe-cli` | `tests/` | planned | pair with `testing-fixtures` plan |
| `packages/happy-cli/src/testing/*` | reusable CLI test scaffolding | `crates/vibe-cli` | `tests/fixtures/` | planned | integration harness and fixtures |
| `packages/happy-cli/src/ui/*` | terminal UX | `crates/vibe-cli` | `ui/` | planned | choose Rust TUI only inside locked module plans |
| `packages/happy-cli/src/utils/*` | shared CLI helper utilities | `crates/vibe-cli` | `utils/` | planned | internal helpers and system adapters |
| `packages/happy-app/package.json` | app package metadata and scripts | `packages/vibe-app` | package metadata | planned | imported by `import-and-build`; script/env renames later flow through `release-and-env` |
| `packages/happy-app/index.ts` | app package entrypoint | `packages/vibe-app` | imported app tree | planned | baseline import owner is `import-and-build` |
| `packages/happy-app/app.config.js` | Expo app config and identifiers | `packages/vibe-app` | app config | planned | owned by `release-and-env` and `branding-and-naming-adaptation` |
| `packages/happy-app/babel.config.js` | app build-tool bootstrap | `packages/vibe-app` | imported app build config | planned | imported intact first by `import-and-build` |
| `packages/happy-app/metro.config.js` | Metro bundler bootstrap | `packages/vibe-app` | imported app build config | planned | imported intact first by `import-and-build` |
| `packages/happy-app/release.cjs` | release script entrypoint | `packages/vibe-app` | release/env scripts | planned | owned by `release-and-env` |
| `packages/happy-app/release-*.sh` | release shell helpers | `packages/vibe-app` | release/env scripts | planned | owned by `release-and-env` |
| `packages/happy-app/src-tauri/**` | desktop shell, config, and bundle metadata | `packages/vibe-app` | `src-tauri/` | planned | owned by `desktop-tauri-adaptation` |
| `packages/happy-app/sources/` | app sources | `packages/vibe-app` | imported tree | planned | phase 1 keeps Happy structure intact |
| `packages/happy-app/sources/app/**` | router, entry screens, and top-level app flows | `packages/vibe-app` | imported app tree | planned | imported first; public naming cleanup later flows through `branding-and-naming-adaptation` |
| `packages/happy-app/sources/auth/**` | app-side auth UX and deep-link entrypoints | `packages/vibe-app` | imported auth tree | planned | endpoint and naming changes must route through `api-endpoint-adaptation` and `branding-and-naming-adaptation` |
| `packages/happy-app/sources/config.ts` | app config/env resolution helpers | `packages/vibe-app` | config/env seam | planned | owned by `api-endpoint-adaptation` and finalized by `release-and-env` |
| `packages/happy-app/sources/realtime/**` | live update subscription and realtime glue | `packages/vibe-app` | realtime adapter seam | planned | endpoint/socket changes flow through `api-endpoint-adaptation` |
| `packages/happy-app/sources/sync/**` | sync engine, reducers, storage, and parser entrypoints | `packages/vibe-app` | sync/protocol seam | planned | protocol parsing belongs to `protocol-parser-compat`; API wiring belongs to `api-endpoint-adaptation` |
| `packages/happy-app/sources/encryption/**` | app-side encryption helpers | `packages/vibe-app` | imported encryption tree | planned | keep imported first; do not fork protocol rules outside the shared crypto spec |
| `packages/happy-app/sources/components/**` | reusable UI components | `packages/vibe-app` | imported component tree | planned | imported baseline; branding-visible changes later flow through dedicated app adaptation plans |
| `packages/happy-app/sources/modal/**` | modal UI flows | `packages/vibe-app` | imported component tree | planned | imported baseline |
| `packages/happy-app/sources/hooks/**` | app hooks and local state helpers | `packages/vibe-app` | imported hook tree | planned | imported baseline |
| `packages/happy-app/sources/utils/**` | app-local utilities and platform helpers | `packages/vibe-app` | imported utility tree | planned | keep local to the app package; avoid promoting these into shared Rust contracts |
| `packages/happy-app/sources/text/**` | translations and public strings | `packages/vibe-app` | text/localization tree | planned | public-surface changes flow through `branding-and-naming-adaptation` |
| `packages/happy-app/sources/constants/**` | app constants and static config values | `packages/vibe-app` | imported constant tree | planned | import baseline first; env/public constant cleanup later |
| `packages/happy-app/sources/track/**` | analytics and screen tracking hooks | `packages/vibe-app` | imported tracking tree | planned | imported baseline; rename only where public Vibe branding requires it |
| `packages/happy-app/sources/types/**` | app-local TypeScript type helpers | `packages/vibe-app` | imported type tree | planned | app-local type ownership stays inside the imported package unless promoted deliberately |
| `packages/happy-app-logs/src/server.ts` | app log sidecar runtime | `crates/vibe-app-logs` | `server.rs` | planned | minimal sidecar runtime mapped by log-server plan |
| `package.json` | root Yarn workspace and bootstrap scripts | repository root | temporary app bootstrap files | planned | import only the minimum root files documented in `modules/vibe-app/import-and-build.md` |
| `scripts/postinstall.cjs` | root postinstall patching and wire build | repository root | bootstrap install seam | planned | import, then localize so it does not hard-depend on `@slopus/happy-wire` |
| `patches/fix-pglite-prisma-bytes.cjs` | node_modules patch required by app install | `patches/` | imported patch file | planned | keep until app bootstrap no longer needs it |

## Migration Rule

If an implementation task touches a Happy source that is not represented in this table, update this
file first and point the new entry at a concrete Vibe target module.
