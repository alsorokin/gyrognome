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
- Provides a pure deterministic simulation API plus an opt-in local runtime
  that schedules and persists explicitly registered characters. Neither layer
  mutates browser saves, renders a UI, or transports data.

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

## Local managed-character runtime

The local runtime imports an existing browser save into a per-user SQLite
database. It is local-only: it makes no HTTP requests, creates no online
characters, and never prints passkeys or unrecognized raw save fields. The
original browser document is retained privately for a future export feature;
the runtime stores its versioned canonical state separately.

Runtime data is stored at:

- `$XDG_DATA_HOME/gyrognome/` when `XDG_DATA_HOME` is set;
- otherwise `~/.local/share/gyrognome/`.

That directory contains `characters.sqlite3` and per-character advisory lock
files. Back up the directory to retain managed state. Do not copy it to
untrusted locations because the database includes the original browser
documents.

Register a browser save, list safe identities, and inspect persisted canonical
state:

```sh
gyrognome register /path/to/character.pqw
gyrognome list
gyrognome managed-inspect <character-id> --json
```

These commands can be exercised with a synthetic save and isolated data root:

```sh
base64 -w0 tests/fixtures/reference-save.json > target/synthetic-character.pqw
XDG_DATA_HOME="$PWD/target/gyrognome-example-data" \
  cargo run -- register target/synthetic-character.pqw
```

For foreground operation, run a worker with the identifier printed by
`register`. It persists completed interval updates using a monotonic clock and
stops cleanly on `SIGINT` or `SIGTERM`.

```sh
gyrognome worker <character-id> --interval-ms 1000
```

Only intervals spent in an active worker are advanced. Starting a worker later
does **not** apply downtime as catch-up progression. A second worker for the
same character exits with an "already running" error; the advisory lock is
released automatically when its owner exits or crashes.

### systemd user-service lifecycle

The packaged `systemd/user/gyrognome@.service` is a user-service template: it
does not require root. Install it for the invoking user, then reload units:

```sh
mkdir -p ~/.config/systemd/user
cp systemd/user/gyrognome@.service ~/.config/systemd/user/
systemctl --user daemon-reload
```

The service manager must be available for the logged-in user, and
`gyrognome` must resolve from its service environment. If it does not, replace
the unit's `ExecStart=gyrognome` command with the absolute path to the installed
binary, reload the user units, and retry.

Use the CLI lifecycle commands rather than invoking the unit directly; they
first validate the local character identifier and then delegate to
`systemctl --user`:

```sh
gyrognome start <character-id>
gyrognome status <character-id>
gyrognome stop <character-id>
gyrognome recover <character-id>
```

`status` reports the safe persisted identity, service activity, and whether a
local worker currently owns the character lock. If the user service manager is
unavailable or a service fails, the command reports that error without
fabricating progression. `recover` clears systemd's failed state and starts a
fresh worker; it resumes only from the last successfully persisted canonical
state. Correct an unsupported simulation state before recovering, or it will
fail again while preserving that state.

## Reference fixtures

`tests/fixtures/browser-reference.json` records the observed browser client
revision, public request-field ordering, and synthetic deterministic values.
Fixtures intentionally exclude player saves, passkeys, browser profiles, and
complete signed request URLs. Regenerate observations only with disposable
characters through the repository's Playwright MCP configuration.
