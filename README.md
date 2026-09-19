# Gyrognome

Gyrognome is a Linux-native Progress Quest client. This initial release provides
its offline-only Rust compatibility core.

## Current support

- Imports browser `.pqw` exports (Base64-encoded JSON) for read-only inspection.
- Preserves unmodified imported JSON documents for export by library consumers.
- Implements browser-compatible Alea state continuation, form-style URL encoding,
  URL normalization, and LFSR request validators.
- Constructs request data, provides an explicitly confirmed reporting path for
  eligible browser-imported managed characters, and supports foreground online
  enrollment from the interactive New Guy wizard. It never reports
  automatically.
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
`ruleset::SOURCE_CONTENT_SHA256`. Completed task durations are credited as exact
fractional seconds to progress bars; the browser-compatible elapsed-task counter
floors each completed task duration to a whole second.

The simulation fixtures in `tests/fixtures/checkpoint-*.json` contain only
synthetic disposable-browser observations. Each records its selected ruleset,
initial canonical state, advancement sequence, and expected canonical state
including Alea continuation. Timing checkpoints also record a nonzero,
equal-total partition, which must replay to that same expected state. They must
not contain player saves, passkeys, browser profiles, or signed leaderboard
requests.

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
database. Workers and lifecycle operations are local-only: they make no HTTP
requests, create no online characters, and never print passkeys or
unrecognized raw save fields. The original browser document is retained
privately for a future export feature; the runtime stores its versioned
canonical state separately.

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

Delete an inactive managed character with an explicit confirmation:

```sh
gyrognome delete <character-id>
```

The command prints only the target's safe identity and requires `yes` before
removing it. Stop an active worker or user service first; deletion never stops
it automatically.

### Confirmed leaderboard reporting

An imported browser character may submit exactly one manual-brag report only
when it has an existing browser-issued online credential, no local worker owns
it, its endpoint is the official Progress Quest leaderboard, and the bundled
credential-free enrollment-conformance evidence is complete and passing. The
command builds its report from the current persisted canonical state:

```sh
gyrognome report <character-id>
```

It displays the safe identity and requires typing `yes` for every submission.
Declining changes nothing and sends no request. A successful HTTP response is
reported only as delivered; it does not establish leaderboard classification.
Rejected and failed deliveries do not retry automatically. Reporting is never
performed by registration, inspection, dashboard refresh, workers, lifecycle
commands, or character administration.

Do not provide a passkey, raw save contents, or a signed request URL to this
command or to any Gyrognome diagnostic. Offline-created characters and active
managed characters are ineligible. Automatic reporting, queues, retries, and
leaderboard polling are intentionally out of scope.

### Interactive New Guy enrollment

Run `gyrognome new-guy` without explicit traits to open the terminal creator.
Its Mode row starts at Offline; select Online to make Sold! perform one
foreground enrollment through the official endpoint. The wizard validates
bundled enrollment evidence before creating the character, sends the required
initial `s` report, and registers it only after both succeed. A duplicate name
returns to the same editable draft. Any other create or report failure is
reported as incomplete enrollment without retrying, registering a local
character, or revealing credentials.

Supplying `--name`, `--race`, and `--class` together always remains the
offline-only scripted creation path.

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
does **not** apply downtime as catch-up progression. Each callback contributes
at most one configured interval; scheduler delay and suspension time beyond
that interval are discarded, not carried into a later callback. A second worker
for the same character exits with an "already running" error; the advisory lock
is released automatically when its owner exits or crashes.

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

### Terminal dashboard

Observe a registered character in a local, full-screen dashboard. With no
identifier, choose one from the safe registered-character list:

```sh
gyrognome dashboard
gyrognome dashboard <character-id>
```

The dashboard reads the persisted canonical state and `systemctl --user`
status every second by default; change that bounded interval with
`--refresh-ms` (100 through 60000). It never advances simulation, acquires
the worker lock, writes state, makes HTTP requests, or sends leaderboard data.
It displays only the credential-safe canonical fields, never browser passkeys,
the retained original save, or unrecognized source fields.

Press `q` to quit, `r` to refresh, `s` to start, `x` to stop, or `c` to
recover the selected service. Start, stop, and recover require `Enter`
confirmation; press `Esc` to cancel. Ctrl-C and SIGTERM quit through the same
terminal-restoration path. The same logged-in-user systemd prerequisites
described above apply to service status and lifecycle actions. If the user
service manager is unavailable or an action fails, the dashboard preserves the
last successfully displayed character state and shows the actionable error.
The selection flow accepts Up/Down or `j`/`k`, `Enter` to open a character, and
`Esc` or `q` to cancel. It reports an error without entering a dashboard when
no characters are registered.

## Reference fixtures

`tests/fixtures/browser-reference.json` records the observed browser client
revision, public request-field ordering, and synthetic deterministic values.
Fixtures intentionally exclude player saves, passkeys, browser profiles, and
complete signed request URLs. Regenerate observations only with disposable
characters through the repository's Playwright MCP configuration.

## Leaderboard conformance experiment

The leaderboard can classify a character as a cheater separately from state
equivalence, so Gyrognome gates any general leaderboard-reporting feature
behind explicit, disposable, opt-in evidence that its report traces match the
browser's and that the browser-created disposable character is never
classified as a cheater. This is not a product feature; a normal Gyrognome
build never talks to the leaderboard.

`scripts/leaderboard-conformance.mjs` is a Playwright harness that:

- Refuses to run at all without `--confirm-disposable`, and refuses any
  option that names an existing/managed character.
- Creates the disposable online character through the official browser only
  (`https://progressquest.com/play/`); the browser is the only thing that
  ever holds its passkey, and only for the lifetime of the ephemeral browser
  context. Gyrognome's credential-free bridge (`conformance-bridge`, a
  hidden, feature-gated CLI subcommand) receives only canonical state.
- Limits browser network access to the observed official pages and the
  leaderboard endpoint (`https://progressquest.com/alpaquil.php`).
- Drives all nine required scenarios — initial load, pause, restart, delayed
  callback, task completion, level-up, act completion, manual bragging, and
  motto change — by injecting a deterministic clock (`Date.now` and the
  client's timer-scheduling path) and invoking the browser's own functions
  directly, rather than waiting out real in-game hours or reimplementing
  browser behavior.
- By default (no `--submit`), intercepts every leaderboard request so no
  network create or report is ever actually sent. Its dry-run evidence records
  the redacted `cmd=create` descriptor before the initial `s` report; the
  server-dependent duplicate-name and interrupted-response observations are
  explicitly marked not run.
- Only submits real reports and polls the live leaderboard for
  classification when both `--submit` and a second, distinct
  `--confirm-live-submission` flag are given. Classification is read from
  the public, unauthenticated realm page
  (`https://progressquest.com/alpaquil.php?name=<character>`), whose
  heading and matching row indicate "Hall of Fame" (normal) or "Hall of
  Infamy" (cheater); no matching row means not-yet-indexed. Polling is
  bounded (every 5 seconds, up to 60 seconds per scenario); a scenario that
  is still unindexed when that bound is reached is recorded `inconclusive`
  and fails the gate, exactly like a `cheater` result — neither is ever
  treated as a pass.
- With both confirmations, creates a second ephemeral browser context that
  attempts the generated name from the successful enrollment and records only
  its browser-visible rejection, redacted request descriptor, and confirmation
  that no second online identity was created. A third context aborts the
  creation response before it is usable, records the browser's retry count,
  and labels that result `unconfirmed`; it never claims whether the server
  reserved that interrupted name.
- Writes only credential-free evidence (`--evidence <path>`, and always to
  stdout): per-scenario pass/fail, redacted `cmd=create` and `cmd=b`
  descriptors (endpoint, method, operation, trigger, and unsigned field
  names), browser-visible enrollment outcomes, ordering, and (when
  submitting) classification results. The evidence gate requires a successful
  creation, duplicate-name rejection, create-before-initial-report ordering,
  and an interrupted enrollment observation. Errors, logs, and evidence reject
  passkeys, response bodies, raw saves, browser profiles, the retained
  original document, and complete signed leaderboard URLs.

Run the credential-free, no-network-report dry run with:

```sh
node scripts/leaderboard-conformance.mjs --confirm-disposable --evidence /path/to/evidence.json
```

Run the full, real-network experiment (creates one disposable character and
submits its reports to the live leaderboard) with:

```sh
node scripts/leaderboard-conformance.mjs --confirm-disposable --submit --confirm-live-submission --evidence /path/to/evidence.json
```

Neither invocation accepts a real character's identity, its saved document, or
its passkey; the procedure never asks an operator to use a managed character
or to publish its passkey. General leaderboard reporting remains unimplemented
and unsupported until this gate's evidence records every required scenario as
conformant and classified normal.
