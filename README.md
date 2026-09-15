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
- Provides a library-only deterministic simulation API. It does not schedule
  advancement, persist state, mutate saves, render a UI, or transport data.

## Deterministic simulation

`simulation::advance(&character, &ruleset::BUNDLED, elapsed_ms)` returns a new
canonical state after the supplied elapsed milliseconds. It never reads a
wall clock or performs filesystem, database, or HTTP operations. Callers select
the ruleset explicitly; `ruleset::BUNDLED` is the Progress Quest browser
`config.js` snapshot from `https://progressquest.com/play/config.js`, revision
6, captured on 2026-09-15. Its content hash is exposed as
`ruleset::SOURCE_CONTENT_SHA256`.

The simulation fixtures in `tests/fixtures/checkpoint-*.json` contain only
synthetic disposable-browser observations. Each records its selected ruleset,
initial canonical state, advancement sequence, and expected canonical state
including Alea continuation. They must not contain player saves, passkeys,
browser profiles, or signed leaderboard requests.

Run conformance and fixture-safety checks with:

```sh
cargo test --test simulation_checkpoints
cargo test --test fixture_safety
```

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
