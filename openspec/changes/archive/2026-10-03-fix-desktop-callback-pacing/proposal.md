# Proposal

## Why

Desktop-profile (Spoltog/Pemptus) workers advance at roughly 61% of wall-clock
speed: each callback sleeps 100 ms *after* about 55 ms of SQLite read, simulate,
and commit work, and the 100 ms per-callback cap discards the overhead. The
dashboard predicts at 1:1, so desktop task bars visibly stick at 100% while the
worker catches up. Browser (Alpaquil) characters are unaffected.

The pinned desktop 6.4.4 source (`Main.dfm`/`Main.pas`) shows a periodic
`TTimer` with `Interval = 100`, whose handler credits `timeGetTime` elapsed
since the end of the previous handler, clamped to 0..100 ms. On a default
Windows timer resolution such a timer fires about every 109.375 ms, so the
original client typically runs at roughly 91% of wall speed. We want to roughly
match that rather than run slower (pointless) or at full speed (risks drifting
from the client and leaderboard anti-cheat classification).

## What Changes

- Desktop workers fire callbacks on a fixed-rate 109.375 ms schedule
  independent of per-callback work time, instead of sleeping a fixed 100 ms
  after the work. Callback semantics stay unchanged: credit monotonic time
  since the end of the previous callback, clamped to 0..100 ms, with the
  full-bar-then-complete boundary and no catch-up for missed callbacks.
- Desktop workers keep their checkpoint in memory and commit it only when a
  callback dispatches task completion, before delivering any level or act
  report, when local-only provenance is first recorded, and on graceful stop.
  Desktop rules cap task length at about 12 s of game time, so completion
  commits happen at least every ~13 s of wall time. A crash may lose the
  current task's partial progress; it cannot duplicate rewards or reports.
- The dashboard predicts desktop task progress at the desktop callback rate
  (100 ms credited per 109.375 ms) instead of 1:1.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `local-character-runtime`: desktop callback cadence becomes a fixed-rate
  109.375 ms schedule, and desktop durability changes from every callback to
  completion, report, provenance, and stop commit points.
- `terminal-dashboard`: desktop task-bar prediction runs at the desktop
  callback credit rate instead of wall-clock rate.

## Impact

- `src/runtime.rs`: `Worker` desktop scheduling, in-memory checkpoint, and
  flush points; `run_until` stop flush.
- `src/desktop_callback.rs`: callback period constant beside
  `MAX_CALLBACK_ELAPSED_MS`.
- `src/dashboard.rs`: per-profile prediction rate in task-anchor prediction
  and whole-percent redraw boundaries.
- `docs/classic-desktop-compatibility.md`: desktop timing section.
- No storage schema, report shape, or browser-profile behavior changes.
