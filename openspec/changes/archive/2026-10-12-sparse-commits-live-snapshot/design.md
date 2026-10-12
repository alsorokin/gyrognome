# Design

## Context

See proposal.md for the motivation. Current mechanics:

- `rested::CHECKPOINT_INTERVAL` is 1 s. Browser workers call
  `replace_state_with_rested` (or `replace_rested`) every loop. Desktop
  workers call `commit_desktop` on every completion, report, local-only
  flip, sleep, and the 1 s checkpoint.
- The browser worker re-reads its state from SQLite on every loop
  (`scheduled_duration`, `advance_elapsed`). The desktop worker already keeps
  a `DesktopSession` in memory and drops it after each commit, so the next
  callback re-reads authentication, evidence, and provenance.
- The dashboard's `dashboard_character` → `Store::managed_inspection` reads
  SQLite. It is called from `LocalProvider::read_state` (polling and settling
  reads) and `collect_snapshot` (combined refresh).
- The database uses the default rollback journal. `restrict_store_files`
  already handles `-wal` and `-shm` files.
- Canonical state JSON written to the database already excludes the online
  profile and credentials. Credentials live in separate columns.

## Goals / Non-Goals

**Goals:**
- About one durable write per character per minute in steady state.
- The dashboard looks as live as it does today.
- Existing guarantees stay intact: atomic state and rest pairing, commit
  before report, no replay.

**Non-Goals:**
- Push notification (inotify) and any socket or IPC protocol.
- Making `managed-inspect`, manual Brag, or `export` read live state. Export
  already stops the worker first. The other two accept up to about one minute
  of staleness.
- Changing the dashboard's polling, settling, or prediction logic.

## Decisions

### 1. Live state is a file in `$XDG_RUNTIME_DIR`

Path: `$XDG_RUNTIME_DIR/gyrognome/<id>.live.json` (directory 0700, file 0600).
If `XDG_RUNTIME_DIR` is unset, the path falls back to `<data_root>/live/`. Stores opened at an explicit data root (`Store::open_at`, used by tests) always use `<data_root>/live/` so they never touch the real runtime directory.
Systemd user services always set the variable. The worker writes a sibling
temporary file and renames it over the target, with no fsync.

Content (serde JSON, versioned):

```
{ "version": 1, "pid": <u32>, "profile": "browser" | "desktop-6.4.4",
  "state": <same canonical JSON the DB column stores>,
  "rested": <RestedState JSON, as stored in the rested_state column> }
```

Reusing the database column formats means the existing deserialization and
validation (`RestedState::validate`, desktop identity derivation) apply
unchanged. Both formats are already credential-free.

Alternatives considered:
- Unix socket: needs a server thread and a protocol.
- Shared memory: overkill.
- A second SQLite database: still pays journal and locking costs.

The atomic rename also keeps a future inotify watcher simple, because it only
has to watch the directory for `IN_MOVED_TO`.

### 2. Observers trust live state only from the current owner

A reader uses the live file only when `Store::is_owned(id)` is true and
`owner_process_id(id)` equals the file's `pid`. In every other case it falls
back to the database. This handles a crash leaving a stale file behind, and a
new worker that hasn't published yet. Both checks are lock-file operations,
not database reads.

### 3. The overlay lives in one dashboard helper

`dashboard_character` reads the database inspection as today (compatibility,
motto/guild, identity), then overlays `state` and `rested` from valid live
state:
- Browser: replace `ManagedCharacter.state`, keeping the profile from the
  database.
- Desktop: replace `RegisteredDesktopCharacter.state` and re-derive identity.
- Rested: `RestedState::view(now, owned)`.

`LocalProvider::read_state` and `collect_snapshot` both go through this
helper, so every dashboard read path is covered. If the live file can't be
read or parsed, the dashboard silently uses the database value.

### 4. Two cadences: publish every 1 s, commit every 60 s

In `rested.rs`, `CHECKPOINT_INTERVAL` is renamed to `PUBLISH_INTERVAL` (1 s,
keeping its current scheduling role), and `COMMIT_INTERVAL = 60 s` is added.
The worker tracks `last_commit_awake` (exists) and `last_publish_awake` (new).

Commit triggers, the same for both profiles: a pending report, the first
local-only flip, observed sleep, graceful stop, and `awake - last_commit_awake
>= COMMIT_INTERVAL`. Publish triggers: every commit, every task completion,
and `awake - last_publish_awake >= PUBLISH_INTERVAL`.

The desktop worker drops the `|| observation.completion_dispatched` commit
trigger and publishes on it instead. Its run loop sleeps until the earlier of
the callback deadline and the next publish deadline.

### 5. The browser worker keeps its state in memory

`Worker` gains `browser_state: Option<Character>`, loaded at start and
advanced in memory. `scheduled_duration` reads it instead of calling
`store.get`. The internal loop uses a new `browser_step`, which simulates,
publishes, and commits only at commit triggers. Report events from
`advance_with_trace` force a commit before `submit_event`.

The public `advance_elapsed` keeps its current always-commit behavior, so
deterministic embedders and the existing conformance tests are unaffected. If
a step fails, both the rest accounting and the in-memory state are restored,
as `browser_update_at` already does for rest.

The motto and guild columns are not written by state commits, so profile
edits made while the worker runs are never overwritten.

### 6. WAL with `synchronous=NORMAL`

`Store::open_at` sets `PRAGMA journal_mode=WAL` and `PRAGMA
synchronous=NORMAL` before `initialize_schema`. WAL is a persistent property
of the database file. With `NORMAL`, transactions stay atomic, but the most
recent ones may roll back on power loss. That is consistent with the
accepted one-minute loss window. `restrict_store_files` already covers the
new files.

## Risks / Trade-offs

- **Crash loses up to about 60 s of progress.** Accepted. Because rest
  downtime is approximated from the last durable timing checkpoint, the lost
  awake time is largely credited back as rest, which partly compensates.
- **Desktop auth, evidence, and provenance changes reach the worker up to
  60 s later.** The session now reloads only on commit. Nothing changes these
  while a worker owns the character, except actions that go through the
  online-action lock, and reports force a commit and reload anyway.
  Acceptable.
- **The dashboard may show progress that a crash later rolls back.**
  Accepted. The display is advisory.
- **Manual Brag and `managed-inspect` see the last commit, up to about 60 s
  old.** Level and act changes force a commit before their own report, so
  the leaderboard never regresses. Brag may omit some recent experience or
  items.
- **Existing tests assert per-completion and 1 s commits.** For example,
  `desktop_batched_callbacks_commit_only_at_commit_points` and
  `rested_desktop_pacing_preserves_cap_completion_and_periodic_commits`.
  These are updated to the new boundaries rather than kept.

## Migration Plan

No data migration. WAL is enabled the first time a new binary opens the
database. Rollback is installing the previous binary, which opens a WAL
database fine. Restarting running workers after the upgrade is enough.
