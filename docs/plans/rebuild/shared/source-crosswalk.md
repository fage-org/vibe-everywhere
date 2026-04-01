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

## High-Value Module Mapping

| Happy source path | Happy responsibility | Vibe target crate/package | Vibe target module | Status | Notes |
| --- | --- | --- | --- | --- | --- |
| `packages/happy-wire/src/messages.ts` | wire message containers | `crates/vibe-wire` | `messages.rs` | planned | define encrypted message and update containers |
| `packages/happy-wire/src/sessionProtocol.ts` | session envelope and event union | `crates/vibe-wire` | `session_protocol.rs` | planned | primary protocol source of truth |
| `packages/happy-wire/src/legacyProtocol.ts` | legacy user/agent content wrapper | `crates/vibe-wire` | `legacy_protocol.rs` | planned | required for app compatibility |
| `packages/happy-wire/src/messageMeta.ts` | message metadata | `crates/vibe-wire` | `message_meta.rs` | planned | shared by CLI, agent, app, server |
| `packages/happy-wire/src/voice.ts` | voice token schema | `crates/vibe-wire` | `voice.rs` | planned | add serde + validation |
| `packages/happy-server/sources/app/auth/*` | auth endpoints and guards | `crates/vibe-server` | `auth/` | planned | driven by shared auth spec |
| `packages/happy-server/sources/app/api/*` | API routes and Fastify types | `crates/vibe-server` | `api/` | planned | Axum or equivalent Rust routing |
| `packages/happy-server/sources/app/api/utils/*` | API bootstrap helpers for auth, error handling, and monitoring | `crates/vibe-server` | `api/` | planned | owned by app-api bootstrap rather than standalone feature modules |
| `packages/happy-server/sources/app/events/eventRouter.ts` | update/event routing | `crates/vibe-server` | `events/router.rs` | planned | socket + fanout hub |
| `packages/happy-server/sources/app/api/routes/accountRoutes.ts` | account profile, settings, and usage routes | `crates/vibe-server` | `api/account.rs` | planned | own account-and-usage surface |
| `packages/happy-server/sources/app/api/routes/connectRoutes.ts` | generic vendor connect routes and GitHub route registration | `crates/vibe-server` | `api/connect.rs` | planned | split generic vendor logic from GitHub-specific behavior |
| `packages/happy-server/sources/app/api/routes/devRoutes.ts` | optional AI-debug logging endpoint | `crates/vibe-server` | `api/dev.rs` | deferred | debug-only route; exclude from parity unless a concrete consumer requires it |
| `packages/happy-server/sources/app/api/routes/artifactsRoutes.ts` | artifact CRUD routes | `crates/vibe-server` | `api/artifacts.rs` | planned | pair with socket artifact handlers |
| `packages/happy-server/sources/app/api/routes/accessKeysRoutes.ts` | access-key CRUD routes | `crates/vibe-server` | `api/access_keys.rs` | planned | session/machine access-key surface |
| `packages/happy-server/sources/app/api/routes/kvRoutes.ts` | key-value support routes | `crates/vibe-server` | `api/kv.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/pushRoutes.ts` | push token registration routes | `crates/vibe-server` | `api/push.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/versionRoutes.ts` | version check route | `crates/vibe-server` | `api/version.rs` | planned | support API group |
| `packages/happy-server/sources/app/api/routes/voiceRoutes.ts` | voice token route | `crates/vibe-server` | `api/voice.rs` | planned | support API group |
| `packages/happy-server/sources/app/session/*` | session lifecycle helpers and deletion logic | `crates/vibe-server` | `sessions/` | planned | session CRUD and lifecycle module |
| `packages/happy-server/sources/app/feed/*` | feed domain logic | `crates/vibe-server` | `feed/` | planned | imported app feed behavior |
| `packages/happy-server/sources/app/social/*` | social and friend domain logic | `crates/vibe-server` | `social/` | planned | friend, relationship, username flows |
| `packages/happy-server/sources/app/github/*` | GitHub integration domain logic | `crates/vibe-server` | `github/` | planned | GitHub-specific connect/disconnect/profile logic |
| `packages/happy-server/sources/app/monitoring/*` | metrics and monitoring hooks | `crates/vibe-server` | `monitoring/` | planned | service metrics and health instrumentation |
| `packages/happy-server/sources/app/api/routes/machinesRoutes.ts` | machine create/list/detail routes | `crates/vibe-server` | `machines/http.rs` | planned | machine CRUD and encrypted machine records |
| `packages/happy-server/sources/app/api/socket/machineUpdateHandler.ts` | machine heartbeat and optimistic concurrency updates | `crates/vibe-server` | `machines/socket.rs` | planned | machine-alive and machine-update-* socket handlers |
| `packages/happy-server/sources/app/api/socket/artifactUpdateHandler.ts` | artifact socket read/update/create/delete | `crates/vibe-server` | `api/socket_artifacts.rs` | planned | artifact socket ack/result compatibility |
| `packages/happy-server/sources/app/api/socket/accessKeyHandler.ts` | access-key socket lookup | `crates/vibe-server` | `api/socket_access_keys.rs` | planned | socket access-key retrieval |
| `packages/happy-server/sources/app/presence/sessionCache.ts` | session/machine validation cache and batched activeAt flush | `crates/vibe-server` | `presence/cache.rs` | planned | lock 30s TTL, 30s threshold, 5s flush interval |
| `packages/happy-server/sources/app/presence/timeout.ts` | session/machine inactivity timeout sweeper | `crates/vibe-server` | `presence/timeout.rs` | planned | lock 10 minute timeout and 1 minute sweep |
| `packages/happy-server/sources/app/kv/*` | KV business logic helpers | `crates/vibe-server` | `api/kv.rs` | planned | keep KV ownership explicit outside generic router setup |
| `packages/happy-server/sources/storage/*` | DB/files/redis/image helpers | `crates/vibe-server` | `storage/` | planned | split by backend concern |
| `packages/happy-agent/src/auth.ts` | account auth flow | `crates/vibe-agent` | `auth.rs` | planned | QR auth and token persistence |
| `packages/happy-agent/src/credentials.ts` | local credential storage | `crates/vibe-agent` | `credentials.rs` | planned | stored under `~/.vibe` |
| `packages/happy-agent/src/encryption.ts` | crypto helpers | `crates/vibe-agent` | `encryption.rs` | planned | may share internals with `vibe-wire` |
| `packages/happy-agent/src/api.ts` | REST session and machine client | `crates/vibe-agent` | `api.rs` | planned | direct server control path |
| `packages/happy-agent/src/session.ts` | Socket.IO session client | `crates/vibe-agent` | `session.rs` | planned | live updates and wait logic |
| `packages/happy-agent/src/machineRpc.ts` | machine-scoped RPC | `crates/vibe-agent` | `machine_rpc.rs` | planned | remote runtime control path |
| `packages/happy-cli/src/agent/` | backend registry and abstractions | `crates/vibe-cli` | `agent/` | planned | keep provider/runtime boundaries explicit |
| `packages/happy-cli/src/agent/factories/*` | provider factory wiring | `crates/vibe-cli` | `agent/registry.rs` | planned | provider factory resolution belongs with agent core |
| `packages/happy-cli/src/api/` | API/session client | `crates/vibe-cli` | `api/` | planned | may share transport helpers with agent |
| `packages/happy-cli/src/commands/*` | top-level CLI command handlers | `crates/vibe-cli` | `commands/` | planned | owned by bootstrap-and-commands plan |
| `packages/happy-cli/src/configuration.ts` | global CLI configuration bootstrap | `crates/vibe-cli` | `config.rs` | planned | owned by bootstrap-and-commands plan |
| `packages/happy-cli/src/daemon/` | daemon control plane | `crates/vibe-cli` | `daemon/` | planned | own local process management |
| `packages/happy-cli/src/codex/` | Codex runtime | `crates/vibe-cli` | `providers/codex/` | planned | session protocol mapping required |
| `packages/happy-cli/src/claude/` | Claude runtime | `crates/vibe-cli` | `providers/claude/` | planned | keep adapters local to provider |
| `packages/happy-cli/src/gemini/` | Gemini runtime | `crates/vibe-cli` | `providers/gemini/` | planned | keep parity with Happy behavior |
| `packages/happy-cli/src/openclaw/` | OpenClaw runtime | `crates/vibe-cli` | `providers/openclaw/` | planned | separate provider module |
| `packages/happy-cli/src/index.ts` | top-level CLI entrypoint | `crates/vibe-cli` | `main.rs` | planned | owned by bootstrap-and-commands plan |
| `packages/happy-cli/src/lib.ts` | reusable CLI bootstrap helpers | `crates/vibe-cli` | `bootstrap.rs` | planned | isolate reusable bootstrap from binary main |
| `packages/happy-cli/src/parsers/*` | CLI-local parser helpers | `crates/vibe-cli` | `parsers/` | planned | special commands and similar local parsing |
| `packages/happy-cli/src/projectPath.ts` | project path resolution | `crates/vibe-cli` | `config.rs` | planned | bootstrap path helper |
| `packages/happy-cli/src/sessionProtocol/types.ts` | CLI-local session protocol helper types | `crates/vibe-cli` | `session_protocol_mapper.rs` | planned | keep CLI-local mapper types out of wire crate unless promoted intentionally |
| `packages/happy-cli/src/sandbox/` | sandbox manager | `crates/vibe-cli` | `sandbox/` | planned | implementation may use Rust process isolation primitives |
| `packages/happy-cli/src/persistence.ts` | local persistence | `crates/vibe-cli` | `persistence/` | planned | resume support depends on this |
| `packages/happy-cli/src/resume/` | resume logic | `crates/vibe-cli` | `resume/` | planned | keep separate from persistence core |
| `packages/happy-cli/src/test-setup.ts` | CLI test bootstrap helpers | `crates/vibe-cli` | `tests/` | planned | pair with testing-fixtures plan |
| `packages/happy-cli/src/testing/*` | reusable CLI test scaffolding | `crates/vibe-cli` | `tests/fixtures/` | planned | integration harness and fixtures |
| `packages/happy-cli/src/ui/` | terminal UX | `crates/vibe-cli` | `ui/` | planned | choose Rust TUI only inside locked module plans |
| `packages/happy-cli/src/utils/*` | shared CLI helper utilities | `crates/vibe-cli` | `utils/` | planned | internal helpers and system adapters |
| `packages/happy-app/sources/` | app sources | `packages/vibe-app` | imported tree | planned | phase 1 keeps Happy structure intact |
| `packages/happy-app-logs/src/server.ts` | app log sidecar runtime | `crates/vibe-app-logs` | `server.rs` | planned | minimal sidecar runtime mapped by log-server plan |
| `package.json` | root Yarn workspace and bootstrap scripts | repository root | temporary app bootstrap files | planned | import only the minimum root files documented in `modules/vibe-app/import-and-build.md` |
| `scripts/postinstall.cjs` | root postinstall patching and wire build | repository root | bootstrap install seam | planned | import, then localize so it does not hard-depend on `@slopus/happy-wire` |
| `patches/fix-pglite-prisma-bytes.cjs` | node_modules patch required by app install | `patches/` | imported patch file | planned | keep until app bootstrap no longer needs it |

## Migration Rule

If an implementation task touches a Happy source that is not represented in this table, update this
file first and point the new entry at a concrete Vibe target module.
