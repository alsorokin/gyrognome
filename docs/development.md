# Development and conformance

For installation and normal use, see the [README](../README.md).
Gyrognome is Linux-native; Rust builds and runtime tests require Linux.

## Build and test

Build or install the CLI from a source checkout:

```sh
cargo build --release
cargo install --path .
```

Both `gyro` and the older `gyrognome` executable are provided. To use background
services, configure `systemd/user/gyrognome@.service` with the absolute installed
binary path, copy it to `~/.config/systemd/user/`, and run
`systemctl --user daemon-reload`. Installation does not enable any characters.

Run automated checks without live submissions:

```sh
cargo test
cargo test --features enrollment-test-transport --test profile_actions
npm ci
npm test
```

Node.js and Playwright support the conformance harness, not the installed client.
`enrollment-test-transport` is test-only: CLI profile actions use categorized
fake outcomes. Do not enable it in an installed client.

For a synthetic local example:

```sh
mkdir -p target
base64 -w0 tests/fixtures/reference-save.json > target/synthetic-character.pqw
XDG_DATA_HOME="$PWD/target/gyrognome-example-data" \
  cargo run -- register target/synthetic-character.pqw
```

Never put real saves, passkeys, browser profiles, or signed leaderboard requests
in the checkout or test artifacts.

## Simulation and fixtures

`simulation::advance(&character, &ruleset::BUNDLED, elapsed_ms)` returns a new
browser canonical state without reading a clock or performing filesystem,
database, or HTTP operations. Callers select the ruleset explicitly.
The bundled browser rules come from
`https://progressquest.com/play/config.js`, revision 6, captured on 2026-09-15;
`ruleset::SOURCE_CONTENT_SHA256` exposes the source hash.
Progress bars use fractional task durations; elapsed-task counters floor each
completed task duration to whole seconds.

`tests/fixtures/checkpoint-*.json` contains synthetic browser observations with
initial state, selected ruleset, advancement sequences, and expected state,
including random continuation. Timing fixtures also verify equal-total
partitions. `tests/fixtures/browser-reference.json` records the observed browser
revision, public request-field ordering, and synthetic deterministic values.

```sh
cargo test --test simulation_checkpoints
cargo test --test fixture_safety
```

Regenerate browser observations only with disposable characters through the
repository's Playwright configuration. Fixtures must remain credential-free.

Browser runtime workers use 100 ms virtual ticks and persist fractional
remainders. Desktop workers use separate callback semantics: normal cadence is
109.375 ms, rested cadence is 54.6875 ms, each callback credits at most 100 game
milliseconds, and task completion follows a full bar on a later callback.
Rested timing is checkpointed at least once per serviced awake second and at
completion, reporting, provenance, and graceful-stop boundaries.
Linux suspend-inclusive and suspend-exclusive clocks distinguish actual sleep
from scheduler or transport delays; delays never become catch-up progression.
Development conformance runners remain unaccelerated.

## Browser leaderboard conformance

Normal builds use bundled, sanitized evidence to gate supported online actions.
State equivalence or HTTP success alone does not establish normal leaderboard
classification. The development-only harness is
`scripts/leaderboard-conformance.mjs`.

The harness requires `--confirm-disposable` and refuses existing or managed
characters. The official browser creates the disposable identity and holds its
passkey only in an ephemeral context. The hidden, feature-gated
`conformance-bridge` receives canonical state without credentials.
Browser access is restricted to the observed official pages and Alpaquil endpoint.

It exercises nine report scenarios: initial load, pause, restart, delayed
callback, task completion, level-up, act completion, manual bragging, and motto
change. A deterministic browser clock drives the official client's functions.
Enrollment checks cover create-before-initial-report ordering, duplicate-name
rejection, and an interrupted creation response.

By default, create/report requests are intercepted. Server-dependent
observations are marked not run; a dry run cannot establish live acceptance:

```sh
node scripts/leaderboard-conformance.mjs \
  --confirm-disposable --guild-designation "Existing test guild" \
  --evidence /path/to/evidence.json
```

**Live execution requires explicit approval for disposable server mutations,**
plus both `--submit` and `--confirm-live-submission`:

```sh
node scripts/leaderboard-conformance.mjs \
  --confirm-disposable --submit --confirm-live-submission \
  --guild-designation "Existing test guild" --evidence /path/to/evidence.json
```

Live classification checks use the public leaderboard, every five seconds for
up to 60 seconds per scenario. Cheater classification and still-unindexed rows
fail the gate; inconclusive is never a pass. Interrupted enrollment remains
unconfirmed rather than claiming whether the server reserved the name.

Guild checks resolve an existing public designation, test join, invalid
designation, and leave, and verify membership publicly rather than trusting the
browser's local value. Cleanup must confirm no guild and normal classification.
The guild gate also requires distinct acceptance/rejection fingerprints,
cancellation, and all nine passing report scenarios.

Output contains only safe outcome categories, unsigned request descriptors,
ordering, and classification observations. Response designations are normalized
using `guild-designations-sha256/v1`; only SHA-256 fingerprints are retained,
not raw or normalized responses. Logs and evidence must exclude credentials,
raw saves, browser profiles, signed URLs, and response bodies.

Only reviewed, sanitized passing observations belong in
`tests/fixtures/enrollment-conformance-evidence.json`. Do not invent acceptance
fingerprints or use real managed characters.

## Desktop conformance

Desktop continuation and online eligibility are separate from browser evidence.
Supported user behavior is described in
[classic desktop compatibility](classic-desktop-compatibility.md); implementation
contracts are in [the desktop specification](../openspec/specs/desktop-save-compatibility/spec.md).
Development runners require the `desktop-live-conformance` feature and are not
part of ordinary use.

Historical scope, safety constraints, and task records are retained in the
[archived OpenSpec changes](../openspec/changes/archive/).
Do not treat a past experiment, a planning artifact, or a command-line
confirmation flag as approval for a new live run. Use newly approved disposable
identities, keep all competing reporters stopped, preserve original saves,
bound requests and observations, and obtain approval for exact private cleanup
paths. Never replay an uncertain mutation.

Pemptus support uses existing observations and narrowly approved equivalence,
not exhaustive Delphi-runtime or anti-cheat certification. Its production guild
confirmation strategy does not supply missing historical response fingerprints.
Leaderboard acceptance of rested timelines remains unverified; normal tests
do not perform live validation.
