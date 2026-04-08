# Module Plan: vibe-app-tauri/session-rendering-and-composer

## Purpose

Port the Happy session detail experience, message timeline, composer, and tool rendering into
`packages/vibe-app-tauri`.

## Source Of Truth

- `projects/vibe-app-tauri.md`
- `docs/plans/rebuild/shared/ui-visual-parity.md`
- `docs/plans/rebuild/vibe-app-tauri-wave9-route-and-capability-matrix.md`
- `/root/happy/packages/happy-app/sources/-session/SessionView.tsx`
- `/root/happy/packages/happy-app/sources/components/MessageView.tsx`
- `/root/happy/packages/happy-app/sources/components/AgentInput.tsx`
- `/root/happy/packages/happy-app/sources/components/markdown/**`
- `/root/happy/packages/happy-app/sources/components/diff/**`
- `/root/happy/packages/happy-app/sources/components/tools/**`
- `/root/happy/packages/happy-app/sources/components/autocomplete/**`

## Target Location

- mobile and desktop session UI surfaces inside `packages/vibe-app-tauri`

## Responsibilities

- session detail shell
- message timeline rendering
- composer behavior
- autocomplete and mode-selection behavior
- markdown, diff, tool, and file rendering semantics

## Non-Goals

- full secondary-route parity
- release migration

## Dependencies

- `session-runtime-and-storage`
- `mobile-shell-and-navigation`
- existing Wave 8 desktop session parity work where still relevant

## Implementation Steps

1. Port session-shell ownership from Happy's session view.
2. Port message rendering by message kind.
3. Port composer behavior, send/abort flows, and autocomplete semantics.
4. Port markdown, diff, file, and tool renderers with parity-focused validation.
5. Validate one end-to-end session chain on desktop and mobile.

## Edge Cases And Failure Modes

- message kind handling drifting from Happy behavior
- composer keyboard behavior diverging on mobile
- markdown/tool rendering semantics drifting during desktop/mobile split
- file surfaces working on one platform family only

## Tests

- message rendering tests
- composer interaction tests
- tool/diff/markdown/file rendering tests
- end-to-end session chain smoke test

## Acceptance Criteria

- users can open a real session, read the timeline, compose input, and see tool output on desktop
  and mobile
- session rendering semantics remain Happy-aligned first

## Locked Decisions

- session rendering parity is more important than UI abstraction elegance
- desktop and mobile may have different host components but must preserve the same message and
  action semantics
- session visuals, timeline density, and composer presentation must remain governed by
  `docs/plans/rebuild/shared/ui-visual-parity.md` unless a narrower exception is recorded first
