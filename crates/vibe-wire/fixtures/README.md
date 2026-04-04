# vibe-wire compatibility vectors

These JSON files are the published non-Rust compatibility vectors for `vibe-wire`.

- Source of truth: `crates/vibe-wire/src/compat.rs`
- Regenerate: `cargo run --example export-fixtures -p vibe-wire`
- Validation: `cargo test -p vibe-wire`
- Happy schema validation: `HAPPY_ROOT=/path/to/happy node scripts/validate-vibe-wire-fixtures.mjs`

Each `*.json` file is an array of objects shaped like:

```json
[
  {
    "name": "fixture-name",
    "value": {}
  }
]
```
