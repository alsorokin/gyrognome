# Gyrognome

Gyrognome is a Linux-native Progress Quest client. This initial release provides
its offline-only Rust compatibility core.

## Current support

- Imports browser `.pqw` exports (Base64-encoded JSON) for read-only inspection.
- Preserves unmodified imported JSON documents for export by library consumers.
- Implements browser-compatible Alea state continuation, form-style URL encoding,
  URL normalization, and LFSR request validators.
- Constructs request data only. It has no HTTP dependency and never creates
  characters or submits leaderboard reports.

## Inspecting a save

```sh
cargo run -- inspect /path/to/character.pqw
```

The default output is a read-only character sheet containing traits, attributes,
activity, progress, equipment, inventory, spells, plots, quests, and online realm
metadata. For deterministic comparisons, request the same credential-free
canonical state as JSON:

```sh
cargo run -- inspect /path/to/character.pqw --json
```

Online character passkeys and unrecognized raw save fields are never included in
either inspection format. `.pqw` files are bearer credentials for leaderboard
reporting and must not be committed.

## Reference fixtures

`tests/fixtures/browser-reference.json` records the observed browser client
revision, public request-field ordering, and synthetic deterministic values.
Fixtures intentionally exclude player saves, passkeys, browser profiles, and
complete signed request URLs. Regenerate observations only with disposable
characters through the repository's Playwright MCP configuration.
