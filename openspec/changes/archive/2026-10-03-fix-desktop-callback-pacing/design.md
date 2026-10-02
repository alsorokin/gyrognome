# Design

## Context

See proposal.md for the measured slowdown and the original-client timer
evidence. Today `Worker::run_until` sleeps the scheduled duration, then calls
`advance_elapsed`. For desktop characters, each call reloads the managed record,
authentication, and random continuation from SQLite, runs one callback, and
commits a full-state `IMMEDIATE` transaction (rollback journal, several
fsyncs). That work (about 55 ms) adds to the 100 ms sleep, and
`apply_callback` clamps the larger elapsed value back to 100 ms. While running,
the worker is the only writer of desktop canonical state. Explicit motto and
guild actions write profile metadata only, and reports read the profile at
delivery time.

## Goals / Non-Goals

**Goals:**

- Desktop game time advances at about 100/109.375 of wall time, regardless of
  per-callback I/O.
- The dashboard's prediction matches that rate, so the full bar shows only
  briefly before the next task appears.

**Non-Goals:**

- Browser-profile scheduling or persistence changes.
- Changing the SQLite journal mode or adding worker-to-dashboard IPC.
- Exact emulation of Windows timer jitter.

## Decisions

### Fixed-rate deadline scheduler for desktop

Keep a `next_deadline: Instant` and advance it by
`DESKTOP_CALLBACK_PERIOD = Duration::from_micros(109_375)` after every
callback. If the deadline has already passed, advance it to the next future
multiple rather than firing back to back. Sleep until the deadline, then check
the stop flag as today. Put the constant beside `MAX_CALLBACK_ELAPSED_MS` in
`desktop_callback.rs`.

Alternative: keep sleep-after-work but sleep `period - work_time`. That is
equivalent in the common case but drifts and handles overruns less clearly.

### Baseline after the simulation step, before I/O

Set `last_tick = Instant::now()` right after `apply_progression_callback`
returns, before any commit or report delivery. Elapsed for the next callback is
`now - last_tick`. The existing 100 ms clamp in `apply_callback` stays the
authority. In normal running, each callback then credits about 100 ms, and the
completion-commit callback does not shorten the following one. This mirrors
the original handler, whose per-tick work is in memory, with `Timer1.Tag`
reset at the end of the handler.

### In-memory desktop checkpoint with explicit commit points

On the first desktop advance, load the `DesktopCallbackCheckpoint`,
authentication, and evidence availability once and cache them in the `Worker`.
Also keep pending counters: credited milliseconds, completed tasks, and whether
the character has been marked local-only. Commit by generalizing
`replace_desktop_checkpoint` to take the accumulated counters instead of a
single observation, then reset them after a successful commit.

Commit when any of these holds:

- the callback dispatched completion;
- `hooks.reports()` is non-empty (commit before delivery, as today);
- `should_mark_desktop_local_only` is newly true;
- the loop exits on a stop request.

On error, return without flushing, which behaves like a crash. Flushing after
a failed completion commit would record the completion while its report was
never delivered.

Keep the public `advance_elapsed` committing every call, since deterministic
embedders and existing tests rely on it. Only `run_until` uses the batched
commit policy, through an internal step that reports whether a commit is
required.

Alternative: a periodic (for example 30 s) flush. Dropped because desktop task
length is bounded: kill tasks take `6 s × monster level ÷ character level`,
where the randomized monster level is at most twice the character level, so at
most 12 s. Market, travel, sell, and queued cinematic or loading tasks take
1–5 s. Completion commits therefore already occur at least every ~13 s of wall
time. The only exception is an imported save's in-flight task, which can have
an arbitrary maximum; that is a one-time case covered by the stop flush.

### Dashboard prediction rate per profile

Add `rate: f64` to `TaskAnchor`, set from
`character.compatibility.profile`: `1.0` for browser and `100.0 / 109.375`
for desktop. Multiply elapsed time by the rate in `predicted_task_position`.
Divide by the rate when computing wall-time deadlines in
`next_task_percent_boundary` and `settling_deadline`. The existing settling
logic (one 100 ms settle, then retries spaced by the settling interval) already
absorbs the extra ~109 ms completion callback.

## Risks / Trade-offs

- [A crash loses the current task's partial progress (normally at most ~12 s;
  unbounded only for an imported save's first, in-flight task)] → Accepted by
  the user. Completion and report commits keep rewards and reports from being
  duplicated.
- [Dashboard opened mid-task anchors to the task's start position] → It
  self-corrects at the next completion commit. `replace_character` already keeps
  the larger of the predicted and persisted positions for the same task. Desktop
  tasks are usually short.
- [Manual Brag or motto actions read a canonical task position from the last
  commit] → Reports do not include the task position. Level, act, and other
  report fields change only at completion, which is committed immediately.
- [Cached authentication or evidence availability goes stale while running] →
  Re-read both on each commit. A change in evidence still takes effect by the
  next task completion.
- [Real Windows clients vary: ~100 ms with high-resolution timers, ~109 ms by
  default] → 109.375 ms stays within the client's observed range and never
  runs faster than its 100 ms ceiling.

## Migration Plan

No schema or data migration. Running workers pick up the change after a
restart (`gyrognome stop`, then `start`). Roll back by reinstalling the
previous binary.
