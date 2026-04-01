# Validation Matrix

## Planning Phase

Required before implementation begins:

- planning tree exists and is internally consistent
- package-to-module mapping is explicit
- canonical shared protocols are documented
- each project has implementation order and acceptance criteria
- each module plan has locked decisions and test requirements

## Shared Contract Phase

For `vibe-wire`:

- unit tests per schema and serialization variant
- compatibility vectors for session protocol, legacy protocol, metadata, and voice
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

## Milestone Exit Criteria

- a milestone is complete only when its project acceptance criteria and required validation set are
  both satisfied
- no downstream milestone may begin with knowingly unstable upstream shared contracts
