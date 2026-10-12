# Proposal

## Why

Each running worker writes its full state to SQLite about once per second (the
1-second checkpoint plus every task completion). The database uses the default
rollback journal, so with several characters running in parallel, the shared
database is constantly busy committing. The database serves two jobs: crash
durability and live dashboard updates. Splitting them lets durable writes drop
to roughly one per minute per character without making the dashboard less
live.

## What Changes

- Workers keep their live state in memory and publish a credential-safe live
  state file in the per-user runtime directory (RAM-backed tmpfs) at each task
  completion and at the existing one-second checkpoint. Publishing is an
  atomic file replacement with no fsync and no database write.
- Durable database commits happen about once per minute, plus at the mandatory
  points that stay unchanged: before any report is delivered, when local-only
  provenance is first recorded, on suspend/wake, and on graceful stop.
  Browser and desktop characters follow the same commit rules.
- While a live runtime owns a character, the dashboard reads that runtime's
  live state instead of the database. It keeps its current polling and
  settling behavior and falls back to the database otherwise.
- The SQLite store switches to WAL journaling with `synchronous=NORMAL`, so
  commits are cheaper and readers are never blocked by a writer.
- **BREAKING (durability)**: an abnormal exit (crash, kill, power loss) can
  lose up to about one minute of progress and rest accounting, instead of
  about one second. Reports are still never replayed. Graceful stops lose
  nothing.
- Out of scope: inotify or other push notification for the dashboard (it can
  be added later on top of the atomic live-state file), and any socket-based
  inter-process communication (IPC).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-character-runtime`: browser results are published live instead of
  being durably committed one by one. Profile-specific desktop commit
  boundaries are replaced by shared runtime commit boundaries with a
  one-minute checkpoint. A new requirement covers live runtime state
  publication.
- `terminal-dashboard`: live refresh reads the owning runtime's published live
  state, and falls back to persisted state when no runtime owns the character.

## Impact

- `src/runtime.rs`: the browser `Worker` path keeps state in memory instead of
  re-reading the database each loop. Desktop and browser commit triggers
  change. A live-state publish step is added. `Store::open_at` sets the WAL
  pragmas.
- `src/rested.rs`: a durable commit interval (60 s) is separated from the
  one-second publish cadence.
- `src/dashboard.rs`: `LocalProvider::read_state` and `collect_snapshot`
  prefer the live state while the character is owned.
- Commands that read the database while a worker runs (`inspect`, manual
  Brag) may see state up to about one minute old. Level and act changes still
  force a commit before reporting, so the leaderboard never regresses. Export
  already stops the worker first.
- No new dependencies.
