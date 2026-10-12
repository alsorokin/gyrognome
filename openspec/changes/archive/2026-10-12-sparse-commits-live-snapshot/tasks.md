# Tasks

## 1. Storage

- [x] 1.1 In `Store::open_at`, set `journal_mode=WAL` and `synchronous=NORMAL`; verify `sqlite3 <db> "pragma journal_mode"` reports `wal` after a test opens the store, and existing store tests pass
- [x] 1.2 Add a live-state module (path resolution from `XDG_RUNTIME_DIR` with `<data_root>/live/` fallback, versioned JSON with pid/profile/state/rested, atomic temp-file rename publish, remove, and an owner-checked read); verify unit tests for round-trip, private permissions, and rejection of a mismatched pid

## 2. Worker cadences

- [x] 2.1 Split `rested::CHECKPOINT_INTERVAL` into `PUBLISH_INTERVAL` (1 s) and `COMMIT_INTERVAL` (60 s), track the last publish time, and publish after every commit; verify the crate builds
- [x] 2.2 Desktop worker: publish (do not commit) on task completion; commit on report, local-only flip, sleep, stop, and `COMMIT_INTERVAL`; verify updated desktop commit-boundary tests (per-completion commit replaced, report-before-delivery still enforced)
- [x] 2.3 Browser worker: hold state in memory, add an internal step that publishes each result and commits only at commit triggers (forced before `submit_event`), keep `advance_elapsed` always-commit; verify the browser rested and fractional-remainder tests and the report-ordering tests pass
- [x] 2.4 On graceful stop, commit and then remove the live file; verify a test that a stopped worker leaves the database current and no live file

## 3. Dashboard

- [x] 3.1 In `dashboard_character`, overlay state and rested from owner-valid live state (browser keeps the database profile; desktop re-derives identity), falling back to the database on any live read error; verify tests for owned overlay, stale-pid fallback, and unowned fallback

## 4. Wrap-up

- [x] 4.1 Run the full `cargo test`; then, with two or more characters running, confirm the dashboard updates on task completion while the database file's modification time advances only about once per minute
