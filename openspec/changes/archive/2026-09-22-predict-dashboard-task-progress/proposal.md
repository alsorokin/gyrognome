# Proposal

## Why

The dashboard renders the task progress bar exactly as it was in the last
persisted snapshot, and it polls for that snapshot on a fixed interval that is
independent of the worker's persistence interval. Because both run at roughly
one second and neither is aligned to the other, the bar advances in coarse
jumps and periodically appears to skip an update when the two cadences drift
past each other.

Polling faster does not fix this. The underlying persisted value still only
changes once per worker tick, so a faster poll multiplies database reads and
`systemctl` subprocess spawns while producing the same jumpy motion.

The jumpiness at task boundaries has a second, independent cause: the worker's
persistence schedule is unrelated to when tasks actually end. The worker sleeps
for a fixed interval and then advances by that whole interval, while the
simulation internally divides that time into hundred-millisecond ticks and
completes the task on one of them. The worker then keeps simulating past the
completion and persists only once, at the end of its interval. A task therefore
finishes well before the dashboard is told, and the dashboard never observes the
next task at its starting position. Prediction alone cannot remove that stall,
because the information simply does not exist in the store yet.

## What Changes

- The dashboard derives a **display-only predicted position** for the Task
  progress bar between persisted snapshots. The task bar is uniquely suited to
  this because its persisted units are already milliseconds of task time, so
  its rate of travel is fully determined by the snapshot itself.
- The dashboard schedules its redraws at the next whole-percent boundary of the
  predicted Task bar instead of at a fixed polling interval, so the bar
  advances one percent at a time regardless of how long the current task is.
- Prediction is strictly bounded to the current task. It never exceeds 100%,
  never increments the completed-task count, and never predicts experience,
  quests, plots, encumbrance, loot, stats, or the next task. Those remain
  step-changes driven only by persisted state.
- Prediction runs only while a local runtime actually owns the character. When
  the character is not owned, or ownership cannot be determined, the displayed
  task bar freezes at its persisted value.
- The worker **aligns its persistence to task boundaries**. Instead of always
  sleeping its full interval, it sleeps until the earlier of its interval and
  the completion of the current task, so a completed task is persisted as soon
  as it completes rather than at the end of the enclosing interval. The
  boundary is rounded up to the simulation's hundred-millisecond tick grid and
  floored at one tick, which keeps the advancement stream identical to today's
  and therefore leaves progression rate and determinism unchanged.
- Persisted character state and runtime service status become **separately
  scheduled reads**. This lets the dashboard re-read the inexpensive persisted
  state promptly when the prediction reaches the end of the current task,
  without spawning service-status subprocesses at that same rate.
- Existing refresh, lifecycle, brag, and recent-task-update behavior is
  preserved. The `--refresh-ms` and `--interval-ms` bounds and defaults are
  unchanged, and no command-line surface is added or removed. This change is
  not breaking.

## Capabilities

### New Capabilities

None. This extends how an existing capability presents state it already reads.

### Modified Capabilities

- `terminal-dashboard`: The "Live persisted-state refresh" requirement changes.
  It currently permits the dashboard to display only the values present in the
  last persisted snapshot. It must additionally permit a bounded, display-only
  predicted position for the current task bar, define when that prediction is
  active and when it freezes, prohibit predicting anything beyond the current
  task, and allow persisted state and service status to refresh on separate
  bounded schedules.
- `local-character-runtime`: The worker's persistence scheduling changes. It
  currently advances and persists on a fixed interval regardless of where task
  boundaries fall. It must instead wake at the earlier of its interval and the
  current task's completion, so that a completed task is persisted promptly,
  while advancing by durations that remain whole multiples of the simulation's
  tick grid so canonical state and the Alea continuation are unaffected.

## Impact

- `src/dashboard.rs`: snapshot observation timing, the interactive run loop's
  redraw scheduling, the state-refresh path, and task-bar rendering in both the
  full Progress pane and the compact progress summary.
- `src/runtime.rs`: the worker's sleep scheduling and the selection of each
  advancement duration.
- `README.md`: the terminal dashboard section, which currently describes the
  displayed state as coming solely from a once-per-second persisted read.
- Unchanged: the deterministic simulation itself, the worker's persistence
  interval bounds and default, the character store, the lifecycle interface,
  leaderboard reporting, and every credential-safety boundary. The dashboard
  continues to acquire no character lock, advance no simulation, and write no
  state.
