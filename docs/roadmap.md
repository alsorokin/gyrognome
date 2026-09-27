# Gyrognome Roadmap

Gyrognome 1.0.0 is feature-complete as a Linux-native Progress Quest client.
Its compatibility, inspection, deterministic simulation, local-runtime,
credential-safe online actions, and terminal dashboard are complete.

## Completed: Compatibility Core

- Browser `.pqw` import and unmodified export support.
- Bounded original Windows `.pq` and same-format `.bak` import under an explicit
  `desktop-6.4.4` continuation profile.
- Safe, redacted character identity inspection.
- Browser-compatible Alea continuation, URL encoding, URL normalization, and LFSR
  request construction.
- Sanitized browser-reference fixtures and an offline-only transport boundary.

## Completed: Character Inspection

The `pq-compatibility-core` capability now provides a full read-only character
sheet:

- Parse and validate PRNG state, stats, elapsed time, active task, bars,
  equipment, inventory, spells, plots, quests, and online metadata.
- Add structured terminal output and a `--json` format for browser/native state
  comparison.
- Separate canonical state from cached browser display fields while preserving
  unrecognized data for safe round-tripping.
- Test with synthetic fixtures and disposable browser-save checkpoints.
- Keeps passkeys and unrecognized raw save data out of inspection output.

## Completed: Deterministic Simulation

The `deterministic-simulation` capability advances canonical state in
browser-compatible order from explicit elapsed durations, bundled versioned rule
data, and synthetic browser-derived conformance checkpoints. It remains pure:
no clock, filesystem, database, or network access is reachable from simulation.

The desktop profile uses separately pinned 6.4.4 rules, integer/random
primitives, 100 ms callback semantics, and source-derived checkpoints. Because
desktop saves omit process random state and reliable lifetime counters,
registration initializes a new persisted desktop continuation and labels
historical values unavailable rather than fabricating them.

## Completed: Local Character Runtime

The `local-character-runtime` capability adds XDG-scoped SQLite persistence,
per-character process locking, and `systemd --user` lifecycle control. Managed
characters progress only while their local runtime is active; stopped time does
not catch up. Eligible online workers deliver each persisted level-up and
act-completion event once to the official leaderboard endpoint; delivery is
best effort and never queues or retries.

Desktop workers preserve full-bar pending completion across restart and discard
delayed callback time above 100 ms. Online-originated desktop imports become
permanently local-only on first advancement while progress reporting remains
gated; future online use requires a fresh official-client import.

## Completed: Managed Character Administration

The `managed-character-administration` capability lets users remove unwanted
local registrations without editing the database:

- `gyro delete <character-id>` displays only the safe identity and requires
  explicit confirmation before removal.
- Removal is atomic and refuses a character currently owned by a local worker;
  users must stop it before retrying.
- Administration remains local-only and never exposes browser passkeys, raw
  saves, or unrecognized source fields.

## Completed: Terminal Dashboard

The `terminal-dashboard` capability provides a credential-safe terminal view
of a managed character:

- Displays live identity, human-readable activity, progress bars, equipped
  items, inventory, spells, plots, quests, and runtime/service status.
- Lets users run `gyro dashboard` without an identifier and select a
  registered character, while preserving direct identifier-based startup.
- Refreshes persisted state without owning character locks or advancing
  simulation.
- Provides confirmed lifecycle actions through the existing runtime controls.
- Restores the terminal on quit, terminal failure, or interrupt.
- Keeps browser passkeys, raw save documents, and raw request data out of the
  presentation layer.
- Displays persisted browser or `desktop-6.4.4` identity, overall online
  eligibility, since-import counters, and actionable local-only recovery
  guidance without exposing account passwords, passkeys, or raw endpoints.

## Completed: Simulation Timing Conformance

The deterministic simulation and local runtime now have stronger timing
conformance guarantees:

- Worker elapsed time is monotonic and capped at one configured interval, so
  scheduler delays and process suspension do not grant catch-up progression.
- Equivalent nonzero elapsed-duration partitions preserve canonical state and
  Alea continuation, including across task-completion boundaries.
- Simulation and elapsed-duration failures preserve the last persisted
  canonical state without partial writes.
- Sanitized paired browser-derived checkpoints cover partitioning and task
  completion behavior.

## Completed: Leaderboard Conformance and Anti-Cheat Safety

The `leaderboard-conformance` capability proves that local state advancement
is browser-equivalent and that controlled characters are not classified as
cheaters:

- Credential-free, ordered report-transition traces (`s`/`l`/`a`/`b`/`m`) from
  deterministic simulation, matching the browser's exact report call sites.
- Trigger-specific report construction and synthetic/browser-derived trace
  conformance fixtures, with no signed request or passkey ever committed.
- A disposable-character, explicitly confirmed Playwright harness that lets
  the official browser create and hold the online credential, so Gyrognome's
  reported traces are compared against it without touching a real save.
- Recorded evidence, gated behind explicit operator confirmation: paired
  browser/Gyrognome scenario traces match, and the disposable character
  remains in the normal leaderboard population rather than the cheater one.

## Completed: Opt-in Imported-character Reporting

Eligible browser-imported managed characters can send one browser-compatible
manual-brag report after an explicit per-submission confirmation, or from the
dashboard's immediate `b` Brag action. Active online workers also deliver each
persisted level-up and act-completion trace event once. The reporting path
rejects offline, malformed, missing-credential, and non-official-endpoint
targets before delivery, and validates bundled credential-free
enrollment-conformance evidence first.

Delivery results are limited to delivered, endpoint-rejected, and
delivery-failed. They do not expose passkeys, raw saves, signed URLs, response
bodies, or leaderboard classification, and no automated retries are attempted.
Automatic worker failures preserve progression and are not queued for replay.

## Completed: Online Profile Management

Managed online characters now retain credential-safe motto and guild metadata
independently from deterministic simulation state:

- Browser-save imports preserve optional motto and guild values, defaulting
  omitted fields to empty values.
- `gyro motto <character-id> <text>` and `gyro guild <character-id>
  <designation>` provide explicit set, clear, join, change, and leave actions.
- Profile actions remain available while a local runtime is active and are
  serialized with automatic reports and manual bragging.
- Motto changes persist before their one-shot report; guild changes persist only
  after a conformance-recognized accepted response.
- Offline characters, invalid credentials, unofficial endpoints, incomplete
  evidence, control characters, and unsafe response data are rejected without
  exposing credentials or raw endpoint content.

## Completed: Dashboard Profile Editing and Presentation

The full dashboard now exposes the completed online client surface without
duplicating simulation or lifecycle ownership:

- `m` edits or clears the persisted motto and `g` submits a guild designation;
  an empty guild value leaves the current guild.
- Editors support printable Unicode input, Backspace, Enter to submit, and
  Escape to cancel, with credential-safe categorized outcomes.
- Details displays formatted elapsed time, Motto, and Guild independently while
  omitting the Quest target.
- Expanded Details fills the remaining left-column height, Activity is capped
  at four rows, Equipment at eleven inner rows, and compact layouts remain
  available for narrow terminals.
- Keyboard help, refresh behavior, and snapshots reflect the completed profile
  and presentation model.

## Completed: Online Enrollment Conformance

The `online-enrollment-conformance` capability establishes browser-derived,
credential-free evidence for a future native enrollment feature:

- Explicitly confirmed disposable browser runs record redacted `cmd=create`
  metadata, successful enrollment, and create-before-initial-`s` ordering.
- A separate ephemeral context confirms duplicate-name rejection without
  creating a second online identity.
- An interrupted creation response is recorded as unconfirmed, including
  observed retry behavior, without claiming whether the server reserved the
  name.
- Evidence validation rejects incomplete observations and sensitive data;
  normal CLI paths remain transport-free.

## Completed: Online New Guy Enrollment

The interactive `gyro new-guy` wizard now provides Offline and Online
modes. Online Sold! validates the bundled enrollment evidence, makes one
official `cmd=create` request, and delivers the browser-compatible initial
`s` report before registering an online managed character.

- Duplicate-name rejection preserves the editable draft for correction without
  sending an initial report or registering a character.
- Unusable creation results and initial-report delivery failures stop without
  retrying, persisting a partial identity, or claiming whether the server
  reserved the name.
- Realm and endpoint are present in credential-safe canonical state; the
  passkey is retained only in private managed storage.
- Enrollment is restricted to the foreground Online wizard; import, offline
  New Guy, workers, lifecycle commands, dashboard, administration, and manual
  reporting cannot invoke enrollment.
- Automated coverage uses a feature-gated synthetic transport, while the
  explicitly confirmed disposable-character conformance workflow keeps live
  credentials and raw responses ephemeral.

## Release Status

Version 1.0.0 is the browser-compatible baseline. Local desktop import and
continuation are implemented under the bounded `desktop-6.4.4` contract.
The September 27, 2026 clean Spoltog experiment established passing,
normal-classification evidence for fresh unadapted imports using the exact
legacy saved endpoint, its verified HTTPS mapping, account/password
authentication, automatic level and act reports, manual brag, motto set/clear,
and accepted/rejected/empty guild operations. Those exact scopes are enabled.
Source-derived differential evidence also permits imports recording only the
deterministic `load-spelling-patch` normalization to use the same operation
evidence while retaining that provenance. Legacy prologue, legacy quest
placeholder, unknown, and combined adaptations, other realms/endpoints or
credential modes, non-ASCII requests, and locally advanced forks remain closed
and must not be described as general classic-server compatibility.
