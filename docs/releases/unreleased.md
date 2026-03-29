## Highlights

- README now uses a more formal technical-document structure for user onboarding, deployment,
  configuration semantics, authentication, overlay ports, and troubleshooting.
- README deployment sections were streamlined by removing duplicated release-artifact inventory and
  transport-scheme prose that was not required in the top-level operator entry document.
- Relay install scripts now follow a binary-only lifecycle model with explicit `install`,
  `update`, and `uninstall` commands.
- Relay install scripts now install the CLI binary set, including `vibe-relay` and `vibe-agent`,
  with optional component scoping for relay-only or agent-only hosts.
- Relay startup guidance now lives in dedicated Chinese and English startup guides instead of
  being implied by the install scripts.
- Relay deployment docs now document GitHub acceleration behavior for mainland-China network paths,
  including script-internal release resolution and archive downloads.

## Included Iterations And Remediations

- Documentation follow-up after `v0.1.8` auth and deployment clarification work.

## Operator Notes

- `scripts/install-relay.sh` and `scripts/install-relay.ps1` do not create services, Scheduled
  Tasks, or environment files; configure startup separately after installing binaries.
- Default Linux install paths are `/usr/local/bin/vibe-relay` and `/usr/local/bin/vibe-agent`;
  default Windows install paths are under `C:\Program Files\Vibe Everywhere\`.
- Avoid using `0.0.0.0` or `127.0.0.1` as client-facing relay origins outside local development.
- If clients should reach a non-default relay port directly, keep that port in
  `VIBE_PUBLIC_RELAY_BASE_URL`.
- Agent bridge ports `19090` to `19092` are expected only when EasyTier overlay mode is enabled.

## Validation

- `./scripts/render-release-notes.sh v0.0.0 >/dev/null`
