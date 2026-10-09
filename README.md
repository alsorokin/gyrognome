# Gyrognome

<img width="425" height="425" alt="image" src="https://github.com/user-attachments/assets/e86c6dae-f233-4df9-9d9f-903c0f838417" />

Gyrognome is a Linux-native Progress Quest client with a Rust compatibility
core and online leaderboard reporting for eligible characters (Alpaquil,
Spoltog, and limited Pemptus operations).

<img width="1920" height="1054" alt="image" src="https://github.com/user-attachments/assets/78c91f56-bc8d-48e8-8dd2-3e48f42a645d" />


## Distribution status

Gyrognome provides versioned Linux archives on the
[GitHub Releases page](https://github.com/alsorokin/gyrognome/releases).
The manual GitHub Actions workflow also produces temporary release candidates
that are accessible to signed-in users with read access and expire after seven
days. There is no installable npm, crates.io, AUR, `.deb`, or `.rpm` package;
the Node.js manifest is for conformance tests only. For archive installation, see
[Linux install and upgrade instructions](docs/linux-release-install.md); they
describe x86_64 and aarch64 glibc 2.35-or-newer builds, checksum verification,
user-local installation, and the optional `systemd --user` unit.

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
- Enables all five classic desktop online operations (level, act, manual brag,
  motto, and guild) for fresh `desktop-6.4.4` Spoltog imports covered by bundled
  passing evidence. Unadapted imports and imports recording only the deterministic
  `load-spelling-patch` normalization are covered; unsupported contracts,
  adaptations, encodings, and local-only forks remain closed.
- Supports all five online operations for otherwise eligible `desktop-6.4.4`
  Pemptus imports: unadapted, spelling-only, and exact indexed quest-placeholder-only.
  Existing unadvanced imports gain support without reimport. Reporting and motto
  reuse pinned native observations across this explicitly approved equivalent
  family; guild actions use one native request and one bounded public membership
  confirmation. Unconfirmed guild results preserve previous membership without
  retry. Unsupported/combined paths and permanent local-only forks stay excluded.
- Supports explicit `--allow-quest-placeholder` admission for a separately
  approved Pemptus progression experiment with the exact indexed `fQuest` marker
  as its sole adaptation. New automatic evidence must declare that original
  import path, even though production readiness can share coverage within the
  supported family. Combined adaptations remain excluded; the flag alone does
  not authorize live requests.
- Allows an explicitly approved older disposable Pemptus character in the
  immediate conformance stage, with exact baseline checks, separately counted
  initial guild cleanup, and preservation of original save files. This does
  not authorize progression or bypass production evidence gates. A freshly
  approved 60-second read-only window can reconcile only the sole uncertain
  preparatory leave; it never replays a mutation or supplies operation evidence.
- Supports a separately approved, non-enabling continuation of the specific
  operator-accepted guild join, without repeating completed live checks. Missing
  join response fingerprints still exclude guild production evidence; only
  independently conclusive operation records can be promoted after cleanup.

See [Classic desktop save compatibility](docs/classic-desktop-compatibility.md)
for supported save layouts, continuation behavior, normalization, online
eligibility, local-only forks, and evidence boundaries.

Pemptus readiness uses existing live observations and narrow source/synthetic
equivalence, not exhaustive per-path Delphi or anti-cheat certification. The
missing historical guild fingerprint audit remains non-conclusive; public
confirmation does not manufacture that evidence. Public state can lag, so a guild
action may report uncertainty even if the server applied it. No new live
validation is required or authorized by this readiness policy.

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

Export a managed character back to a save file (`.pq` for desktop characters,
`.pqw` for browser characters):

```sh
gyro export <character-id> [-o PATH] [--force]
```

The default path is `./<save-name>.pq` or `.pqw`. The file contains the
character's credentials, so it is written atomically with mode 0600. An existing
file prompts for confirmation unless `--force` is given. A running user service
is stopped for the export and restarted afterwards.

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
A successful HTTP response is reported only as delivered; it does not establish
leaderboard classification.
After the confirmed report attempt completes, the CLI opens the character's
realm-specific public leaderboard page with its display name selected. This
happens whether delivery succeeds, is rejected, or fails; a browser-launch
failure is reported separately. Declining changes nothing, sends no request,
and opens no page. The dashboard's Brag action also opens the public page after
its manual report attempt. Worker level-up and act-completion reports never open
a page and are attempted once each regardless of delivery outcome.
Registration, inspection, dashboard refresh, lifecycle commands, and character
administration never report.

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

Only time spent in an active worker advances game state. Every managed character
automatically earns **rested time** while its worker is stopped or the computer
sleeps/hibernates, up to a **12-hour bank**. Each banked second buys one real
second of **2x progression**: tasks, experience, loot, quests, and plots all
progress faster through their unchanged game rules. Twelve hours banked gives
twelve real hours boosted, not six. Stopping preserves unused rest and adds
subsequent downtime up to the cap; starting never grants instant offline rewards.

New registrations start with an empty bank. Existing characters also start empty
when their database is first migrated; old update timestamps do not grant
retroactive rest. Motto/guild edits do not reset the bank. Rested metadata stays
in the managed store and is not carried in official save exports.

Browser workers service stable 100 ms virtual ticks and retain fractional
virtual time across updates and restarts. They wake at the earlier of the
configured interval, tick-aligned task completion, rest exhaustion, and a
one-second checkpoint. `--interval-ms` (1000 by default) bounds contributing
**real** time; the boost can earn twice that virtual time, with a previous
fractional remainder available to complete a tick.

Desktop workers retain their original callback rules: each callback credits at
most 100 game milliseconds, and a full bar completes only on a later callback.
Normal cadence is 109.375 real milliseconds; rested cadence is 54.6875 ms.
Partial progress and accounting are checkpointed at least once per awake second
while the loop is serviced, as well as at completion, report, provenance, and
graceful-stop boundaries.

Awake scheduler stalls, persistence, and report delivery do not earn rest.
They still spend available rested time, and discarded delays never become
catch-up progression. Linux suspend-inclusive and suspend-exclusive clocks
distinguish actual computer sleep from these delays. Stopped-time accrual uses
the local wall clock: forward adjustments can add capped rest, while backward
adjustments do not credit already-accounted time again. After a crash, downtime
is estimated from the last durable timing checkpoint; a crash during a long
blocked operation can leave a larger uncertain interval.

This applies to otherwise eligible online characters too, without a new
rested-evidence gate. Existing reporting restrictions, local-only history,
request construction, and no-retry behavior remain unchanged. **Official
leaderboard acceptance and classification of rested timelines are unverified.**
No live verification is required or automatically performed. Development
conformance runners retain their original, unaccelerated timing.

A second worker for the same character exits with an "already running" error;
the advisory lock is released automatically when its owner exits or crashes.

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

`status` reports the safe persisted identity, service activity, whether a
local worker currently owns the character lock, and persistent autostart
(`On`, `Off`, or `Unavailable` with a diagnostic). JSON output includes
`autostart` as `enabled`, `disabled`, or `unavailable`. If the user service manager is
unavailable or a service fails, the command reports that error without
fabricating progression. `recover` clears systemd's failed state and starts a
fresh worker; it resumes only from the last successfully persisted canonical
state. Correct an unsupported simulation state before recovering, or it will
fail again while preserving that state.

### Automatic startup after restart

Autostart is opt-in for each registered character:

```sh
gyro autostart <character-id> on
gyro autostart <character-id> off
gyro status <character-id>
```

Enabling autostart schedules the existing service when your user service manager
next starts, normally at login. It does **not** start the character now; use
`gyro start` for that. Disabling autostart does not stop an active worker.
Start, Stop, Recover, and export's temporary stop/restart never change this
preference. A manually stopped character with autostart On will start again
at the next user-manager startup. External systemd enablement changes are
reflected by the CLI and dashboard; temporary `--runtime` enablement does not
count as persistent autostart.

To run enabled characters after boot **before login**, and keep them running
after logout, separately enable account-wide lingering:

```sh
loginctl enable-linger "$USER"
```

Your host may require administrator authorization. Gyrognome never changes
lingering or elevates privileges automatically. The installed service template
and reliable absolute executable path described above are still required.
Preference changes work on unit files even without an active user manager.
Registration and installation do not automatically enable characters.

Automatic workers resume the last persisted state with the existing rest,
reporting-eligibility, and local-only provenance rules; there is no immediate
offline-time catch-up. Enabling an online-originated desktop character may
warn that later advancement will make it permanently local-only if reporting
is gated, just as manual startup does. Confirmed deletion cleans up startup
registration before removing character data; cleanup failures retain the data.

Autostart remains configured if you roll back to an older binary. To remove it
without the new CLI command, use
`systemctl --user disable gyrognome@<character-id>.service` (without `--now`).

### Terminal dashboard

Observe a registered character in a local, full-screen dashboard. With no
identifier, choose one from the safe registered-character list:

```sh
gyro dashboard
gyro dashboard <character-id>
```

The character-selection list shows each character's available rested time as
`Rested: <time>`, without a progression multiplier,
alongside its identity, last access, and service activity.

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

Full Details shows remaining rested time as `Rested: <time>`. Compact Character
content also shows the active 1x/2x multiplier. A stopped character can have a bank without being
actively boosted. Task-bar prediction follows the profile's boosted rate only
for the estimated remaining rest, then returns to normal rate; it still cannot
complete tasks or award rewards. Managed inspection retains the unverified
rested-leaderboard notice without cluttering the dashboard.
`gyro managed-inspect <character-id> --json` exposes
`rested.availableMs` and `rested.activeMultiplier` without credentials; inspecting
a stopped character projects pending rest without writing or resetting it.

For desktop records, Details shows one overall online eligibility result
and counters measured since import, without technical identifier or
compatibility-profile rows. Unadvanced provenance
is hidden; a local-only record instead shows the actionable fresh
official-client import requirement. Start and recover confirmations repeat the
permanent local-only warning for an online-originated import that would advance
while progress reporting is gated.
Ineligible Brag, motto, and guild controls stop at the shared typed gate without
calling transport. A desktop Task bar that is full but awaiting its next actual
callback remains at 100%; display prediction never invents completion rewards,
a new activity, or a report.

The dashboard refreshes automatically. Press `q` to quit, `b` to immediately
submit one eligible manual Brag report, `m` to edit the motto, `g` to edit the
guild, or `s` for the contextual lifecycle action: start an inactive service,
stop an active one, or recover a failed one by clearing its failed state and
starting it. Brag has no confirmation overlay and shows only delivered,
endpoint-rejected, or delivery-failed outcomes. Start, stop, and recover
require `Enter` confirmation; press `Esc` to cancel. Press `a` (`auto` in key
help) to toggle the selected character's persistent autostart, also with
Enter/Escape confirmation. Both full and compact views show
`Autostart: On`, `Off`, or `Unavailable` independently of service activity.
The toggle changes future startup only, without starting or stopping the worker.
Ctrl-C and SIGTERM quit
through the same terminal-restoration path. The same logged-in-user systemd
prerequisites described above apply to service status and lifecycle actions.
If the user service manager is unavailable or an action fails, the dashboard
preserves the last successfully displayed character state and shows the
actionable error. The selection flow accepts Up/Down or `j`/`k`, `Enter` to
open a character, and `Esc` or `q` to cancel. It reports an error without
entering a dashboard when no characters are registered.

Profile editors start with the persisted value, accept printable Unicode and
Backspace, submit with Enter, and cancel with Escape without sending a request.
While editing, action keys such as `q`, `b`, or `s` insert text instead of
triggering actions. Empty input clears the motto or leaves the guild. After
submission the dashboard refreshes persisted profile values and shows the
same safe outcome categories as the CLI.

In the full layout, F1 through F6 toggle Activity, Progress, Equipment, Details,
Adventure, and Journal. The non-collapsible Status pane shares the bottom row
with Keys, with Keys on the left at one-third width and Status on the right at
two-thirds width. They use one content row when the shortcut line fits and
expand to two only when it wraps. The Journal title includes the current plot caption (for example,
`Journal - Act VIII`), while Adventure omits a separate plot row. Expanded Activity has four total rows,
Progress seven, and Equipment at most thirteen (eleven content rows). Expanded
Details fills the remaining vertical space, with a six-row minimum when space
permits; collapsing it retains only its two border rows. Details shows the
identifier and compact elapsed time (for example,
`1d 1h 1m 1s` or `0s`), with Motto and Guild independently shown only when
nonempty; it no longer shows Quest target. Narrow terminals retain the compact
character view, showing only the current quest rather than completed quest
history. Its XP, encumbrance, plot, and quest percentages share a summary, with
task progress on its own bar immediately above, using the same appearance and
prediction as full mode. Empty lines separate equipment from the preceding
percentage summary and following inventory, and separate the spellbook from plot
information. All of this content scrolls together. The compact Keys pane
likewise uses one content row when the shortcut line fits and two only when it
wraps.

Content that overflows a pane's visible area can be scrolled. In the full
layout, `Tab` and `Shift+Tab` move keyboard scroll focus forward and backward
among the expanded panes, with a "none selected" position between the last and
first panes. No pane is selected initially; `Tab` selects the first expanded
pane, and `Shift+Tab` selects the last. The focused pane has a cyan
double-line border. `Up`/`Down` scroll the focused pane by one row, and
`PageUp`/`PageDown` scroll it by one visible page. Collapsing the focused
pane moves focus to the next expanded pane. With no pane selected, these
keyboard scrolling keys do nothing in the full layout, and toggling panes
leaves focus unselected. The mouse wheel scrolls whichever
expanded pane is under the pointer, independent of keyboard focus. In the
compact layout, `Up`/`Down`, `PageUp`/`PageDown`, and the mouse wheel all
scroll the combined Character pane. Scroll offsets are independent per pane,
stay within the available content as the terminal is resized, and never
change character state or pane collapse state.

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
