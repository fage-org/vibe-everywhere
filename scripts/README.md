# scripts

This directory is reserved for validation, migration, packaging, and release helpers introduced
during the rebuild.

Do not add ad hoc one-off scripts here without documenting their owner project and validation role
in the relevant plan files.

Current validation helpers:

- `validate-vibe-wire-fixtures.mjs`
  - owner: `vibe-wire`
  - role: validate published `crates/vibe-wire/fixtures/*.json` against Happy source-of-truth
    schemas
  - prerequisites: a local Happy checkout at `HAPPY_ROOT` or the default `/root/happy`
