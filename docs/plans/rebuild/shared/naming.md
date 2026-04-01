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
| `happy-app` | `vibe-app` | imported TS app package |
| `happy-app-logs` | `vibe-app-logs` | Rust sidecar binary crate |

## Binary Names

- `vibe-server`
- `vibe-agent`
- `vibe`
- `vibe-app-logs`

Do not ship `happy-*` binaries from this repository.

## Config And Home Paths

- default home dir: `~/.vibe`
- agent credentials: `~/.vibe/agent.key`
- CLI credentials: `~/.vibe/access.key`
- daemon files: `~/.vibe/daemon/`
- logs: `~/.vibe/logs/`

## Environment Variable Prefix

- primary prefix: `VIBE_`
- app runtime variables should follow `EXPO_PUBLIC_VIBE_*` or the environment model chosen inside
  `modules/vibe-app/release-and-env.md`

Do not introduce new public `HAPPY_*` environment variables. Temporary compatibility aliases are
allowed only inside adapter modules and must be called out there.

## Deep Links And URLs

- primary app deep link scheme: `vibe:///`
- web/documentation branding: `vibe`
- server base URL variables and docs must use `vibe` naming

## Source-Level Transition Rule

During the imported app phase, some source identifiers may still contain `happy` strings.
That is acceptable only inside `packages/vibe-app` until
`modules/vibe-app/branding-and-naming-adaptation.md` is complete.

External behavior must prefer Vibe naming from day one:

- docs
- binaries
- env vars
- home directories
- package and crate names
- user-visible command output
