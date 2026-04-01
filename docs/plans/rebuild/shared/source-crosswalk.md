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
| `packages/happy-server/sources/app/events/eventRouter.ts` | update/event routing | `crates/vibe-server` | `events/router.rs` | planned | socket + fanout hub |
| `packages/happy-server/sources/app/presence/*` | presence and timeouts | `crates/vibe-server` | `presence/` | planned | active session and machine presence |
| `packages/happy-server/sources/storage/*` | DB/files/redis/image helpers | `crates/vibe-server` | `storage/` | planned | split by backend concern |
| `packages/happy-agent/src/auth.ts` | account auth flow | `crates/vibe-agent` | `auth.rs` | planned | QR auth and token persistence |
| `packages/happy-agent/src/credentials.ts` | local credential storage | `crates/vibe-agent` | `credentials.rs` | planned | stored under `~/.vibe` |
| `packages/happy-agent/src/encryption.ts` | crypto helpers | `crates/vibe-agent` | `encryption.rs` | planned | may share internals with `vibe-wire` |
| `packages/happy-agent/src/api.ts` | REST session and machine client | `crates/vibe-agent` | `api.rs` | planned | direct server control path |
| `packages/happy-agent/src/session.ts` | Socket.IO session client | `crates/vibe-agent` | `session.rs` | planned | live updates and wait logic |
| `packages/happy-agent/src/machineRpc.ts` | machine-scoped RPC | `crates/vibe-agent` | `machine_rpc.rs` | planned | remote runtime control path |
| `packages/happy-cli/src/agent/` | backend registry and abstractions | `crates/vibe-cli` | `agent/` | planned | keep provider/runtime boundaries explicit |
| `packages/happy-cli/src/api/` | API/session client | `crates/vibe-cli` | `api/` | planned | may share transport helpers with agent |
| `packages/happy-cli/src/daemon/` | daemon control plane | `crates/vibe-cli` | `daemon/` | planned | own local process management |
| `packages/happy-cli/src/codex/` | Codex runtime | `crates/vibe-cli` | `providers/codex/` | planned | session protocol mapping required |
| `packages/happy-cli/src/claude/` | Claude runtime | `crates/vibe-cli` | `providers/claude/` | planned | keep adapters local to provider |
| `packages/happy-cli/src/gemini/` | Gemini runtime | `crates/vibe-cli` | `providers/gemini/` | planned | keep parity with Happy behavior |
| `packages/happy-cli/src/openclaw/` | OpenClaw runtime | `crates/vibe-cli` | `providers/openclaw/` | planned | separate provider module |
| `packages/happy-cli/src/sandbox/` | sandbox manager | `crates/vibe-cli` | `sandbox/` | planned | implementation may use Rust process isolation primitives |
| `packages/happy-cli/src/persistence.ts` | local persistence | `crates/vibe-cli` | `persistence/` | planned | resume support depends on this |
| `packages/happy-cli/src/resume/` | resume logic | `crates/vibe-cli` | `resume/` | planned | keep separate from persistence core |
| `packages/happy-cli/src/ui/` | terminal UX | `crates/vibe-cli` | `ui/` | planned | choose Rust TUI only inside locked module plans |
| `packages/happy-app/sources/` | app sources | `packages/vibe-app` | imported tree | planned | phase 1 keeps Happy structure intact |

## Migration Rule

If an implementation task touches a Happy source that is not represented in this table, update this
file first and point the new entry at a concrete Vibe target module.
