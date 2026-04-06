# Validation Matrix

## Planning Phase

Required before implementation begins:

- planning tree exists and is internally consistent
- package-to-module mapping is explicit
- every major Happy source tree plus critical build/bootstrap inputs and transport entrypoints has an explicit owner in `shared/source-crosswalk.md`
- target module paths in `shared/source-crosswalk.md`, project plans, and module plans agree with the current implementation layout, or the future seam is explicitly marked as deferred
- canonical shared protocols are documented
- each project has implementation order and acceptance criteria
- each module plan has locked decisions and test requirements
- completion claims in `PLAN.md`, `master-summary.md`, `execution-plan.md`, and `execution-batches.md` are consistent with any open remediation sections in the owning project plans
- when `vibe-app-tauri` is active, the extraction inventory, route inventory, capability matrix,
  and coexistence matrix exist as explicit planning artifacts before module implementation begins
- when `vibe-app-tauri` uses localhost loopback auth, bind address, state validation, listener
  lifecycle, and per-instance ownership rules are explicit in planning before implementation begins
- when `vibe-app-tauri` route parity is frozen, currently desktop-visible routes are classified as
  `P0`, `P1`, `P2`, or `deferred` rather than being left implicit

## Shared Contract Phase

For `vibe-wire`:

- unit tests per schema and serialization variant
- compatibility vectors for session protocol, legacy protocol, metadata, and voice
- cross-language schema validation of published compatibility vectors against Happy source-of-truth schemas
- crypto vector tests for any reusable crypto helpers

## Server Phase

For `vibe-server`:

- route-level tests for auth, sessions, machines, and updates
- storage integration tests for encrypted records
- socket update tests for session and machine updates
- compatibility tests using `vibe-wire` fixtures

## Agent Phase

For `vibe-agent`:

- auth flow tests
- credential storage tests
- REST client tests
- socket session client tests
- end-to-end control flow tests against a real `vibe-server`

## CLI Phase

For `vibe-cli`:

- provider runtime tests per provider
- session protocol mapper tests
- daemon tests
- persistence/resume tests
- sandbox tests
- end-to-end tests against a real `vibe-server`

## App Phase

For `vibe-app`:

- import/build verification
- protocol parser compatibility tests
- endpoint adaptation verification
- at least one real chain test: app -> server -> agent or CLI
- desktop shell verification once Tauri adaptation begins

For `vibe-app-tauri`:

- package bootstrap validation
- route-level desktop navigation smoke tests
- parser/reducer compatibility checks for any reused shared session/message logic
- at least one real desktop chain test: app-tauri -> server -> agent or CLI
- auth/connect callback validation against the locked localhost loopback strategy, including state
  validation, timeout, listener teardown, and per-instance ownership behavior
- desktop package smoke validation for Tauri bundles
- realistic session-load performance and memory review before promotion
- Linux, macOS, and Windows startup/package validation before promotion
- side-by-side parity checklist review against current `vibe-app` desktop behavior before promotion

## Milestone Exit Criteria

- a milestone is complete only when its project acceptance criteria and required validation set are
  both satisfied
- implementation-order `[done]` markers do not authorize “validated complete” status claims while
  blocking remediation remains open in the owning project plan
- no downstream milestone may begin with knowingly unstable upstream shared contracts
