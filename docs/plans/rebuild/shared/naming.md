# Naming And Surface Rules

## Product Naming

- product name: `Vibe`
- repository root: `vibe-remote`
- external package/crate prefix: `vibe`
- internal Rust module prefix: `vibe_`

## Project Mapping

| Happy name | Vibe name | Notes |
| --- | --- | --- |
| `happy-wire` | `vibe-wire` | Rust library crate |
| `happy-server` | `vibe-server` | Rust server binary crate |
| `happy-agent` | `vibe-agent` | Rust agent-control binary crate |
| `happy-cli` | `vibe-cli` | Rust runtime crate, binary name `vibe` |
| `happy-app` | `vibe-app` | deprecated legacy TS reference package |
| `happy-app` | `vibe-app-tauri` | active TS app package and replacement target |
| `happy-app-logs` | `vibe-app-logs` | Rust sidecar binary crate |

## Package And Import Naming

- repository root package/workspace name: `vibe-remote`
- TypeScript package names created in this repository must prefer `vibe-*` names
- `packages/vibe-app-tauri` is the reserved active app package name in this repository
- Do not introduce new Vibe-owned package names or import paths under `@slopus/happy-*`
- Temporary compatibility imports of `@slopus/happy-wire` are allowed only inside imported
  app/CLI transition seams and must be isolated or removed by the owning module plan before the
  corresponding parity milestone closes

## Binary Names

- `vibe-server`
- `vibe-agent`
- `vibe`
- `vibe-app-logs`

Do not ship `happy-*` binaries from this repository.

## Public Commands And Scripts

- public CLI, helper, and release script names must use `vibe` naming
- `vibe-app-tauri` remains the repository package path for the active app package; any public naming cleanup must still be documented before a user-facing rename
- if a dev-only variant command is needed, prefer `vibe-dev`
- if an MCP/stdin bridge helper is needed, prefer `vibe-mcp`
- do not document or ship new primary interfaces named `happy`, `happy-dev`, `happy-mcp`, or
  similar Happy-branded commands
- compatibility-only aliases may exist temporarily inside adapter or migration seams, but they must
  stay undocumented as the primary Vibe interface

## Config And Home Paths

- default home dir: `~/.vibe`
- development-only variant home dir: `~/.vibe-dev`
- agent credentials: `~/.vibe/agent.key`
- CLI credentials: `~/.vibe/access.key`
- daemon files: `~/.vibe/daemon/`
- app log sidecar files: `~/.vibe/app-logs/`

Do not introduce new Vibe-owned flows that default to `~/.happy` or `~/.happy-dev`.

## Environment Variable Prefix

- primary prefix: `VIBE_`
- preferred direct ports of Happy environment variables should use:
  - `VIBE_SERVER_URL`
  - `VIBE_WEBAPP_URL`
  - `VIBE_HOME_DIR`
  - `VIBE_PROJECT_DIR`
  - `VIBE_PROJECT_ROOT`
  - `VIBE_EXPERIMENTAL`
  - `VIBE_DISABLE_CAFFEINATE`
  - `VIBE_HTTP_MCP_URL`
- app runtime variables exposed to Expo/web/JS code must use the `EXPO_PUBLIC_VIBE_*` prefix
- non-public app build, native, or release-only variables must use the `VIBE_` prefix unless an
  owning module plan records a compatibility exception
- `modules/vibe-app/release-and-env.md` may define the concrete variable list and profile wiring,
  but it must not redefine these public/private prefix rules

Do not introduce new public `HAPPY_*` environment variables. Temporary compatibility aliases are
allowed only inside adapter modules and must be called out there.

## Flags, RPC Names, And Compatibility Aliases

- public flags and subcommand names must prefer `vibe-*` or neutral names, never new `happy-*`
  names
- internal compatibility-only identifiers such as legacy daemon RPC names, hidden flags, or
  imported script knobs may temporarily retain `happy*` strings only when an owning module plan
  explicitly needs them
- examples that must remain adapter-only if they survive during migration:
  - `--happy-starting-mode`
  - `spawn-happy-session`
  - `resume-happy-session`
- these compatibility aliases must not be documented as the primary Vibe UX

## Deep Links And URLs

- primary app deep link scheme: `vibe:///`
- the old `packages/vibe-app` and `packages/vibe-app-tauri` coexistence rule for `vibe:///` is now historical reference only; active ownership decisions belong to the Wave 9 plan set
- web/documentation branding: `vibe`
- server base URL variables and docs must use `vibe` naming
- public issue/help/documentation links and user-visible prompts must not send users to
  Happy-branded command names or repositories

## Compatibility-Locked Exceptions

- some serialized field names, crypto literals, and imported-app compatibility shims intentionally
  retain `happy*` strings during phase one
- these exceptions are allowed only when a shared spec or owning module plan records them
- current examples live in:
  - `shared/data-model.md`
  - `shared/protocol-auth-crypto.md`
  - `modules/vibe-app/branding-and-naming-adaptation.md`
- naming compatibility exceptions must be explicit; this file does not authorize silent drift back
  to Happy-branded public surfaces

## Source-Level Transition Rule

During the imported-app phase, some source identifiers contained `happy` strings.
That historical allowance applied inside `packages/vibe-app` during Wave 7 and is no longer an active naming rule for new work.

External behavior must prefer Vibe naming from day one:

- docs
- binaries
- env vars
- home directories
- package and crate names
- user-visible command output
