# Module Plan: vibe-cli/ui-terminal

## Purpose

Implement terminal-facing UI helpers and interactive displays for the CLI.

## Happy Source Of Truth

- `packages/happy-cli/src/ui/*`
- `packages/happy-cli/src/ui/ink/*`

## Target Rust/Vibe Location

- crate: `crates/vibe-cli`
- files:
  - `src/ui/mod.rs`
  - `src/ui/display.rs`
  - `src/ui/qrcode.rs`

## Responsibilities

- render runtime output in the terminal
- show QR codes and auth prompts
- format messages and statuses for operators

## Non-Goals

- command parsing
- runtime logic

## Public Types And Interfaces

- terminal renderer helpers
- QR rendering helper
- message formatter

## Data Flow

- runtime or auth command produces typed events
- UI layer formats and renders them for terminal users

## Dependencies

- `crossterm`
- `ratatui` for interactive views where needed
- `qrcode`

## Implementation Steps

1. Port non-interactive formatting first.
2. Add interactive views only where Happy CLI behavior depends on them.
3. Implement QR rendering for auth/connect flows.
4. Add snapshot tests for formatting output.

## Edge Cases And Failure Modes

- terminal width handling
- color/TTY assumptions in non-interactive environments

## Tests

- formatter snapshot tests
- QR rendering test
- non-TTY fallback test

## Acceptance Criteria

- CLI output is usable in interactive and scripted environments

## Open Questions

- None.

## Locked Decisions

- use `ratatui` plus `crossterm` when interactive UI is needed
- non-interactive output must remain plain-text friendly
