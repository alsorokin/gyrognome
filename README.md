# Gyrognome

Gyrognome is a Linux-native Progress Quest client with a Rust compatibility
core and online leaderboard reporting for eligible characters (Alpaquil and Spoltog realms for now).

## Distribution status

The repository currently prepares **private Linux release candidates**, not
public downloads. There is no published GitHub Release or installable npm,
crates.io, AUR, `.deb`, or `.rpm` package. The Node.js manifest is for
conformance tests only. For a locally prepared candidate archive, see
[Linux install and upgrade instructions](docs/linux-release-install.md); they
describe x86_64 and aarch64 glibc 2.35-or-newer builds, checksum verification,
user-local installation, and the optional `systemd --user` unit. Publication
remains blocked pending a final pre-publication privacy review and explicit
owner authorization.

Gyrognome-authored code is MIT licensed; see
[third-party notices](THIRD_PARTY_NOTICES.md) for the upstream source and the
owner's porting rationale. Existing source-checkout commands below remain
available for developers with Rust installed.

## Current support

- Imports browser `.pqw` exports (Base64-encoded JSON) and supported original
  Windows desktop `.pq` or same-format `.bak` saves for read-only inspection.
- Preserves unmodified imported JSON documents for export by library consumers.
- Implements browser-compatible Alea state continuation, form-style URL encoding,
  URL normalization, and LFSR request validators.
- Constructs request data, provides manual reporting and persisted online
  worker event reporting for eligible browser-imported managed characters, and
  supports foreground online enrollment from the interactive New Guy wizard.
- Persists motto and guild metadata independently of simulation progress, with
  explicit CLI and dashboard editing available while a worker is active.
- Provides separate browser and `desktop-6.4.4` continuation profiles plus an
  opt-in local runtime that schedules and persists explicitly registered
  characters without mutating source saves.
- Enables classic desktop level, act, manual brag, motto, and guild operations
  only for fresh `desktop-6.4.4` Spoltog imports covered by bundled passing
  evidence. Unadapted imports and imports recording only the deterministic
  `load-spelling-patch` normalization are covered; every other realm, endpoint,
  credential mode, adaptation, encoding, and local-only fork remains closed.

See [Classic desktop save compatibility](docs/classic-desktop-compatibility.md)
for supported save layouts, continuation behavior, normalization, online
eligibility, local-only forks, and evidence boundaries.

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
activity, progress, equipment, inventory, spells, plots, quests, online realm
metadata, and the credential-safe motto and guild profile. Desktop output also
identifies its continuation target, recognized adaptations, counters measured
since import, and safe online eligibility. For deterministic comparisons,
request the same credential-free canonical state as JSON:

```sh
cargo run -- inspect /path/to/character.pqw --json
```

Passkeys, desktop account passwords, authenticated/raw endpoints, and
unrecognized raw save fields are never included in either inspection format.
Save files can contain bearer credentials and must not be committed.

### Classic desktop compatibility

Desktop saves use a separate, bounded compatibility contract rather than
browser simulation rules. See
[docs/classic-desktop-compatibility.md](docs/classic-desktop-compatibility.md)
for supported layouts and adaptations, random/history limitations, online
eligibility, local-only behavior, transport restrictions, and conformance
evidence.

## Local managed-character runtime

The local runtime imports an existing browser or supported desktop save into a
per-user SQLite database. Eligible browser workers send one best-effort
official-endpoint report after each persisted level-up or act-completion event;
offline workers and currently gated desktop workers never report. Workers do
not create online characters and never print private authentication or
unrecognized raw save fields. Browser source documents are retained privately
for future export; desktop canonical state and private authentication are stored
separately, and no desktop source document is retained. Schema version 4 stores
typed compatibility, profile-specific random continuation, desktop import
metadata, and private authentication while preserving migrated browser records.
Worker writes cannot overwrite profile edits.

Runtime data is stored at:

- `$XDG_DATA_HOME/gyrognome/` when `XDG_DATA_HOME` is set;
- otherwise `~/.local/share/gyrognome/`.

That directory contains `characters.sqlite3` and per-character advisory lock
files. A separate per-character online-action lock orders profile actions,
manual brags, and worker reports without transferring worker ownership.
Back up the directory to retain managed state. Do not copy it to
untrusted locations because the database includes the original browser
documents.

Before a database schema migration, Gyrognome creates a user-private
`characters.sqlite3.pre-v<schema>.backup` snapshot. To roll back to an older
binary, stop all workers, move the current data directory aside, restore that
backup as `characters.sqlite3` in a private `0700` directory with file mode
`0600`, and then start the older binary. Progress recorded after the backup is
not present. Gyrognome never automatically downgrades or overwrites newer
state.

Register a browser or desktop save, list safe identities with their persisted
profiles and eligibility, and inspect persisted canonical state:

```sh
gyro register /path/to/character.pqw
gyro register /path/to/desktop-character.pq
gyro register /path/to/desktop-character.bak
gyro list
gyro managed-inspect <character-id> --json
```

Delete an inactive managed character with an explicit confirmation:

```sh
gyro delete <character-id>
```

The command prints only the target's safe identity and requires `yes` before
removing it. Stop an active worker or user service first; deletion never stops
it automatically.

### Confirmed leaderboard reporting

An imported browser character may submit exactly one manual-brag report only
when it has an existing browser-issued online credential, no local worker owns
it, its endpoint is the official Progress Quest leaderboard, and the bundled
credential-free enrollment-conformance evidence is complete and passing. The
command builds its report from the current persisted canonical state and motto:

```sh
gyro report <character-id>
```

It displays the safe identity and requires typing `yes` for every submission.
Declining changes nothing and sends no request. A successful HTTP response is
reported only as delivered; it does not establish leaderboard classification.
Rejected and failed deliveries do not retry automatically. A worker attempts
each persisted online level-up and act-completion event once, then continues
regardless of delivery outcome. Registration, inspection, dashboard refresh,
lifecycle commands, and character administration never report.

Do not provide a passkey, raw save contents, or a signed request URL to this
command or to any Gyrognome diagnostic. Offline-created characters and active
managed characters are ineligible for manual bragging. Delivery queues, retries,
and leaderboard polling are intentionally out of scope.

### Motto and guild actions

Eligible online characters can update their profile without stopping a worker:

```sh
gyro motto <character-id> "Onward!"
gyro motto <character-id> --clear
gyro guild <character-id> "Existing guild designation"
gyro guild <character-id> ""
```

Each explicit command submits exactly one action without another confirmation.
Motto text and `--clear` are mutually exclusive; an explicitly empty motto also
clears it. An empty guild designation leaves the current guild; there is no
separate leave command. Printable non-control Unicode is accepted for browser characters. Desktop
profile text must additionally be ASCII. Controls, offline characters, invalid
credentials, unofficial endpoints, unsupported encoding, local-only provenance,
and invalid or absent operation evidence are rejected before mutation or
delivery.

A motto is saved before its one `t=m` report and retained even if delivery
fails or the endpoint rejects it. Later manual and worker reports use the saved
value. Delivery alone does not prove server acceptance.

A guild is saved only after a response matches the verified, operation-specific
acceptance fingerprint. The transport disables redirects, bounds the response
to 16 KiB and the request to 30 seconds, and never exposes response text.
Rejected, unknown, oversized, unsuccessful, or undeliverable responses preserve
the previous guild. Output is limited to accepted, rejected, or indeterminate
categories; there are no automatic retries or production membership lookups.

### Interactive New Guy enrollment

Run `gyro new-guy` without explicit traits to open the terminal creator.
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
gyro worker <character-id> --interval-ms 1000
```

Only time spent in an active worker is advanced. Starting a worker later does
**not** apply downtime as catch-up progression. The worker wakes at the earlier
of its configured interval and the current task's completion, so
`--interval-ms` (1000 by default) remains an upper bound on how long it sleeps
and how much time one update contributes. Scheduler delay and suspension time
beyond that bound are discarded, not carried into a later callback. A second
worker for the same character exits with an "already running" error; the
advisory lock is released automatically when its owner exits or crashes.

### systemd user-service lifecycle

The packaged `systemd/user/gyrognome@.service` is a user-service template: it
does not require root. Install it for the invoking user, then reload units:

```sh
mkdir -p ~/.config/systemd/user
cp systemd/user/gyrognome@.service ~/.config/systemd/user/
systemctl --user daemon-reload
```

The service manager must be available for the logged-in user, and
`gyro` must resolve from its service environment. If it does not, replace
the unit's `ExecStart=gyro` command with the absolute path to the installed
binary, reload the user units, and retry.
The installed `gyrognome` compatibility executable remains available for older
service files and drop-in overrides; use `gyro` for interactive commands.

Use the CLI lifecycle commands rather than invoking the unit directly; they
first validate the local character identifier and then delegate to
`systemctl --user`:

```sh
gyro start <character-id>
gyro status <character-id>
gyro stop <character-id>
gyro recover <character-id>
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
gyro dashboard
gyro dashboard <character-id>
```

The dashboard reads the persisted canonical state and `systemctl --user`
status every second by default; change that interval with `--refresh-ms`
(100 through 60000). While a local runtime owns the character, the Task bar
moves between reads using a display-only prediction and promptly re-reads
persisted state after the task finishes. Prediction is confined to the Task
bar; all other progress, activity, rewards, and statistics change only when
persisted state is read. The dashboard never advances simulation, acquires the
worker lock, or writes canonical simulation state. Explicit profile actions
write only profile metadata. It displays only credential-safe canonical fields
and profile values, never browser passkeys, the retained original save, raw
endpoint responses, or unrecognized source fields.

For desktop records, Details shows `desktop-6.4.4`, one overall online
eligibility result, and counters measured since import. Unadvanced provenance
is hidden; a local-only record instead shows the actionable fresh
official-client import requirement. Start and recover confirmations repeat the
permanent local-only warning for an online-originated import that would advance
while progress reporting is gated.
Ineligible Brag, motto, and guild controls stop at the shared typed gate without
calling transport. A desktop Task bar that is full but awaiting its next actual
callback remains at 100%; display prediction never invents completion rewards,
a new activity, or a report.

Press `q` to quit, `r` to refresh, `b` to immediately submit one eligible
manual Brag report, `m` to edit the motto, `g` to edit the guild, `s` to start
an inactive service or stop an active one, or `c` to recover the selected
service. Brag has no confirmation overlay and shows only delivered,
endpoint-rejected, or delivery-failed outcomes. Start, stop, and recover
require `Enter` confirmation; press `Esc` to cancel. Ctrl-C and SIGTERM quit
through the same terminal-restoration path. The same logged-in-user systemd
prerequisites described above apply to service status and lifecycle actions.
If the user service manager is unavailable or an action fails, the dashboard
preserves the last successfully displayed character state and shows the
actionable error. The selection flow accepts Up/Down or `j`/`k`, `Enter` to
open a character, and `Esc` or `q` to cancel. It reports an error without entering a dashboard when
no characters are registered.

Profile editors start with the persisted value, accept printable Unicode and
Backspace, submit with Enter, and cancel with Escape without sending a request.
While editing, action keys such as `q`, `b`, or `s` insert text instead of
triggering actions. Empty input clears the motto or leaves the guild. After
submission the dashboard refreshes persisted profile values and shows the
same safe outcome categories as the CLI.

In the full layout, F1 through F7 toggle Activity, Progress, Equipment, Details,
Status, Adventure, and Journal. Expanded Activity has four total rows,
Progress seven, and Equipment at most thirteen (eleven content rows). Expanded
Details fills the remaining vertical space, with a six-row minimum when space
permits; collapsing it retains only its two border rows. Details shows the
identifier and compact elapsed time (for example,
`1d 1h 1m 1s` or `0s`), with Motto and Guild independently shown only when
nonempty; it no longer shows Quest target. Narrow terminals retain the compact
character view and display profile-editor shortcuts in the footer.

## Reference fixtures

`tests/fixtures/browser-reference.json` records the observed browser client
revision, public request-field ordering, and synthetic deterministic values.
Fixtures intentionally exclude player saves, passkeys, browser profiles, and
complete signed request URLs. Regenerate observations only with disposable
characters through the repository's Playwright MCP configuration.

## Classic desktop conformance

Classic online support is bounded by separately approved, disposable
conformance evidence. The development-only procedure, safety rules, historical
results, and currently enabled scope are documented in
[Classic desktop save compatibility](docs/classic-desktop-compatibility.md).

## Leaderboard conformance experiment

The leaderboard can classify a character as a cheater separately from state
equivalence, so Gyrognome gates any general leaderboard-reporting feature
behind explicit, disposable, opt-in evidence that its report traces match the
browser's and that the browser-created disposable character is never
classified as a cheater. The experiment is development-only; normal builds use
the sanitized bundled evidence to gate the explicit online features and
eligible online worker reports described above.

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
- With `--guild-designation`, resolves an existing guild's canonical public
  identifier before creating a character, then exercises joining, a deliberately
  invalid designation, and leaving. Public server membership verifies acceptance
  or rejection; the browser's locally assigned guild is not sufficient evidence.
  Cleanup always attempts leaving and must verify empty membership and normal
  leaderboard classification.
- Normalizes exact prior/submitted designations in ephemeral response text
  using `guild-designations-sha256/v1`, storing only SHA-256 fingerprints and
  safe outcome categories. Neither raw nor normalized response text is retained.
- Writes only credential-free evidence (`--evidence <path>`, and always to
  stdout): per-scenario pass/fail, redacted `cmd=create` and `cmd=b`
  descriptors (endpoint, method, operation, trigger, and unsigned field
  names), browser-visible enrollment outcomes, ordering, and (when
  submitting) classification results. The evidence gate requires a successful
  creation, duplicate-name rejection, create-before-initial-report ordering,
  and an interrupted enrollment observation. Errors, logs, and evidence reject
  passkeys, response bodies, raw saves, browser profiles, the retained
  original document, and complete signed leaderboard URLs. The guild gate also
  requires distinct accepted-join, rejected-invalid, and accepted-leave
  fingerprints, cancellation, server verification, successful cleanup, and all
  nine passing report scenarios.

Run the credential-free, no-network-report dry run with:

```sh
node scripts/leaderboard-conformance.mjs --confirm-disposable --guild-designation "Existing test guild" --evidence /path/to/evidence.json
```

Run the full, real-network experiment (creates one disposable character and
submits its reports to the live leaderboard) with:

```sh
node scripts/leaderboard-conformance.mjs --confirm-disposable --submit --confirm-live-submission --guild-designation "Existing test guild" --evidence /path/to/evidence.json
```

Neither invocation accepts a real character's identity, its saved document, or
its passkey; the procedure never asks an operator to use a managed character
or to publish its passkey. Only sanitized passing evidence belongs in
`tests/fixtures/enrollment-conformance-evidence.json`; do not substitute
synthetic acceptance fingerprints for actual disposable observations.

Run automated checks without live submissions:

```sh
cargo test
cargo test --features enrollment-test-transport --test profile_actions
npm test
```

`enrollment-test-transport` is test-only: its CLI profile actions use categorized
fake outcomes, never live requests. Do not enable it in an installed client.
