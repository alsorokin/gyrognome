## Context

See proposal.md for motivation. The repository currently contains OpenSpec and a
repository-scoped Playwright MCP configuration, but no application code or existing
specifications. The official web client exports Base64-encoded JSON state, persists
an Alea PRNG state, and builds signed leaderboard requests using a server-issued
per-character passkey. The server-side acceptance rules remain unknown.

## Goals / Non-Goals

**Goals:**

- Establish a testable Rust boundary between browser-format character data and native
  domain logic.
- Make compatibility behavior deterministic and fixture-driven.
- Preserve imported data and prevent accidental online actions while the protocol is
  still being characterized.
- Give later persistence, terminal UI, service, and HTTP changes a stable core.

**Non-Goals:**

- Advancing a character, rendering a TUI, creating a native online character, or
  submitting leaderboard reports.
- Treating the exposed client-side passkey validator as sufficient anti-cheat.
- Supporting imported saves from unverified third-party ports.

## Decisions

### Use a single Rust binary crate with pure compatibility modules

The initial crate will separate a pure state/PRNG/protocol library boundary from CLI
presentation. This keeps deterministic behavior testable without filesystem,
terminal, or HTTP effects and avoids premature workspace extraction.

An initial multi-crate workspace was considered. It would make the module boundary
more explicit, but adds release and dependency overhead before a second consumer
exists.

### Retain the raw imported JSON document alongside a typed canonical projection

Import will validate and expose the known browser fields through typed state while
retaining the original JSON representation for unmodified export. This provides
forward compatibility for fields not yet understood and avoids data loss during the
read-only phase.

Fully modeling every field immediately was considered. It would create a fragile
and unnecessarily broad first implementation; the retained document makes unknown
field preservation possible without treating unknown data as simulation input.

### Port browser semantics through reference fixtures, not “equivalent” algorithms

The Alea implementation, bounded random behavior, signed 32-bit LFSR arithmetic,
query ordering, percent encoding, and URL normalization will be asserted against
sanitized values observed from the official web client. Tests will include both
initial-sequence and restored-state cases.

Using a conventional Rust RNG or generic query encoder was rejected because small
semantic differences can change progression or request validators.

### Model request construction as data and omit transport

The leaderboard module will output request data or URLs for test comparison but
will have no HTTP dependency and no operation capable of transmission. The passkey
will be supplied only by a caller and redacted from user-facing diagnostics.

Adding an HTTP client now was rejected because it could affect a real character
before report cadence and server validation behavior are understood.

### Derive redacted fixtures from a disposable reference character

Playwright will observe normal browser interactions with a dedicated disposable
character. Fixtures will replace all credentials and identifying character values
with synthetic equivalents while preserving encoding, field ordering, event type,
and validator relationships. Personal `.pqw` exports remain ignored by Git.

Using an existing personal online save as a fixture was rejected because its
passkey permits leaderboard impersonation.

## Risks / Trade-offs

- [Browser behavior changes after fixtures are recorded] -> Pin each fixture to the
  observed revision and retain the observation procedure so it can be rerun.
- [Incorrect JavaScript numeric or URL behavior changes a validator] -> Use
  explicitly wrapping 32-bit operations and exact end-to-end request fixtures.
- [Raw-document retention conceals unsupported fields] -> Keep typed canonical
  validation strict and report unsupported/malformed required fields at import.
- [A user mistakes request construction for leaderboard support] -> Expose no HTTP
  transport and state this limitation in CLI help and documentation.
- [Reference observations expose credentials] -> Use a disposable character, redact
  before committing, and test fixtures for prohibited secret-bearing fields.

## Migration Plan

1. Add the crate, fixtures, and read-only inspection command without touching any
   existing character data.
2. Validate fixtures and imported copies of browser saves locally.
3. Release the compatibility core as an offline-only tool.
4. If a defect is found, roll back by removing the binary; browser saves remain
   unchanged because this change does not write them or contact the leaderboard.
