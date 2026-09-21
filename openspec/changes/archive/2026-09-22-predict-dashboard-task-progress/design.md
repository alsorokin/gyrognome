# Design

## Context

See proposal.md - Why for motivation.

The dashboard's run loop is a single `event::poll(refresh_interval)` wait. A
timeout means "refresh everything", and a key event means "handle input".
Refresh and redraw are therefore the same event: the display can only change
when the persisted snapshot is re-read.

`DashboardProvider::refresh` bundles two reads of very different cost. Reading
persisted canonical state is a local SQLite query plus an advisory-lock probe;
reading service status spawns a `systemctl --user is-active` subprocess. Any
increase in refresh frequency therefore increases subprocess spawns at the same
rate.

The task progress bar is the only bar whose persisted representation already
encodes its own rate of travel. `ProgressBar.position` for the task bar is
elapsed task milliseconds and `max` is the task duration in milliseconds,
because `simulation::advance` increments the task bar by exactly the elapsed
millisecond count and `set_task` resets `max` to the task's duration. Every
other bar advances only in discrete steps at task completion, by an amount that
depends on RNG, loot, level-ups, and queue contents.

The worker's contributed time is deliberately not wall-clock time.
`select_worker_elapsed` caps each callback's contribution at one interval and
discards scheduler delay and suspension beyond it, so the persisted state is
not a function of how much real time has passed since it was written.

Two independent grids govern progression, and neither is the task boundary. The
worker persists on its own interval, while `advance_with_trace` internally
divides whatever duration it is given into ticks of at most `MAX_TICK_MS`
(100 ms), mirroring the browser's timer. Ticks are capped, not padded, so an
advancement is exactly reproduced by the tick sequence it decomposes into. Task
durations are computed with integer division and are rarely multiples of either
grid, and `ProgressBar::reposition` clamps to `max`, silently discarding the
overshoot of the tick that crosses the boundary. A task therefore consumes
`ceil(max / 100) * 100` milliseconds of simulated time, completes on a tick in
the middle of some worker interval, and is persisted only when that interval
ends. The dashboard consequently learns of a completion up to a full worker
interval late, and never observes the next task at position zero.

This yields the invariant the runtime change rests on: any advancement duration
that is a whole multiple of 100 ms decomposes into the same tick sequence as
any partition of it into other 100 ms multiples. Today's worker always feeds
exactly its interval; feeding a shorter 100 ms multiple instead produces a
bit-identical tick stream, canonical state, and Alea continuation. The
equivalence does not extend to partitions that are not 100 ms multiples,
because a misaligned split changes which tick crosses the boundary and thus how
much overshoot the clamp discards.

## Goals / Non-Goals

**Goals:**

- Make the task bar advance one percent at a time for tasks of any duration,
  without increasing the rate of persisted-state or service-status reads beyond
  what the change explicitly bounds.
- Persist a completed task at the moment it completes, so the dashboard has a
  truthful next-task snapshot to re-anchor to instead of waiting out the
  remainder of a worker interval.
- Keep the worker's advancement stream, and therefore canonical state and the
  Alea continuation, bit-identical to today's.
- Keep the prediction arithmetic a pure, directly testable function of an
  anchor and an elapsed duration, so it can be verified without a terminal or a
  running worker.
- Preserve the existing observer boundary exactly: no lock acquisition, no
  simulation call, no state write, no additional network request.

**Non-Goals:**

- Predicting any value other than the current task bar's position, including
  the other four bars, the completed-task count, and the activity description.
- Reproducing simulation behavior in the dashboard. The dashboard must not
  determine what the next task is, what loot dropped, or whether a level-up
  occurred; only persisted state answers those.
- Changing the worker's persistence interval bounds or default, the
  `--refresh-ms` default, or any command-line surface. Only *when within an
  interval* the worker wakes changes.
- Increasing the worker's advancement rate, tick resolution, or total work per
  unit of real time. The worker still advances by at most one interval per
  wake.

## Decisions

### Align the worker's wake to the current task's completion

The worker computes the simulated time remaining in the current task, rounds it
up to the 100 ms tick grid, floors it at one tick, and sleeps for the lesser of
that and its configured interval. The duration it then advances by is that same
sleep duration, still capped by `select_worker_elapsed`.

This removes the dashboard's boundary stall at the source rather than masking
it. The completed task is persisted at the instant it completes, and the very
next snapshot the dashboard reads shows the following task near its starting
position, which is the clean re-anchor frame the prediction wants.

It is safe precisely because of the 100 ms-multiple equivalence established in
Context. Every duration the worker feeds remains a whole multiple of the tick
grid, so the tick sequence, the clamped overshoot at each task boundary, the
resulting canonical state, and the Alea continuation are unchanged from today.

Rounding *up* to the tick grid rather than sleeping the exact remaining
milliseconds is essential. Sleeping the exact remainder would feed a misaligned
duration, which is the one case where the equivalence fails and progression
would genuinely diverge. The floor of one tick is equally essential: a task can
be created already complete or with a sub-tick remainder, and without the floor
the worker would spin without sleeping.

Making the dashboard watch the database file for changes was considered and
rejected as insufficient on its own. It would tell the dashboard about a write
the instant it lands, but the write itself would still be up to a worker
interval late, so the stall this change exists to remove would remain.

### Anchor the prediction to monotonic observation time, not the stored timestamp

Each successful persisted-state read records the observed task position, the
task duration, a task identity, and an `Instant` captured at the moment of
observation. The predicted position is `anchor_position + elapsed_since_anchor`,
clamped to the task duration.

`ManagedCharacter` already carries `updated_at_unix_ms`, so backdating the
prediction to the store's write timestamp was considered and rejected for two
reasons. First, it is a `SystemTime` value and is not monotonic, so a clock
adjustment could make the bar jump or rewind. Second, and more importantly, it
would reintroduce wall-clock advancement that the runtime deliberately
discards: after a suspension the worker advances by at most one interval while
the stored timestamp reflects the full elapsed wall time, so the prediction
would overshoot the character's real progression.

Anchoring at observation time makes the prediction structurally conservative.
The dashboard observes a snapshot strictly after the worker wrote it, so the
predicted position always trails the true position by at most the observation
lag. The bar can therefore reach 100% only after the real task has already
completed, which is the desired direction for a saturation trigger.

### Re-anchor on every successful state read, and never rewind within a task

Every successful persisted-state read replaces the anchor. This continually
corrects accumulated drift rather than letting it compound, and it satisfies
the spec's re-anchoring requirement without needing to detect whether the state
"changed".

Because the prediction trails reality, a re-anchor normally moves the bar
forward. To make a backward step impossible in the remaining edge cases, the
displayed position is held non-decreasing while the task identity is unchanged;
when the task identity changes, the new persisted position is displayed
directly, including the reset to the start of the next task. Task identity
reuses the completed-task count that `RecentTaskUpdates` already relies on,
combined with the task duration.

### Derive the wait timeout from the next percent boundary

The run loop keeps its single `event::poll` wait but computes the timeout
instead of always passing the refresh interval. The timeout is the minimum of
the time until the predicted bar's next whole-percent boundary, the time until
the next scheduled full refresh, and the time until a permitted saturation
read. Because the task bar's position is in milliseconds, the time until the
bar shows `p + 1` percent is exactly the millisecond distance to
`ceil((p + 1) * max / 100)`, which requires no calibration or frame counting.

A fixed high-frequency redraw timer was rejected because it decouples redraw
cost from the thing being animated: a twenty-second task would redraw hundreds
of times per visible percent change, while a one-second task would still look
coarse. Deriving the deadline from the data redraws exactly as often as the
display can actually change.

The computed timeout is floored at a small minimum spacing so that very short
tasks cannot request an unbounded redraw rate. The shortest tasks the ruleset
produces are one second, which is one percent every ten milliseconds; a floor
in the low tens of milliseconds keeps that visually smooth while capping the
worst case. When the floor applies, the bar advances by more than one percent
per redraw, which the spec's delayed-redraw clause already covers.

### Split the provider's state read from its service read

`DashboardProvider` gains a state-only read alongside the existing combined
`refresh`. The combined read continues to run on the regular interval and on
explicit user refresh; the state-only read is what the saturation trigger uses.
This is the entire justification for the split: without it, reacting promptly
to a completed task would spawn a `systemctl` subprocess each time.

Replacing `refresh` with two independent reads was rejected because the
existing snapshot assembly maps a service-status failure into the displayed
message, and splitting that error handling would change how service-manager
failures surface for no benefit to this change. Adding a narrower second method
leaves the existing combined path, its error mapping, and its tests intact.

### Delay the saturation-triggered read by a settling interval

When the predicted position reaches the task duration, the dashboard does not
read immediately. It waits a bounded settling interval first, then performs a
state-only read, and separates successive triggered reads by that same
interval.

The delay exists because saturation does not imply the completion is already
visible in the store. Even with the worker aligned to the task boundary, the
dashboard's prediction saturates at the same moment the worker wakes, and the
worker still has to dispatch the completion, select the next task, and commit a
transaction. Reading at the exact saturation instant would race that commit and
return the state the prediction was already anchored to.

With boundary alignment in place, this interval is now sized to cover a single
commit rather than a worker interval. That is the substantive difference from
the earlier design, where the settling interval was absorbing an unbounded
fraction of a worker tick and could not be sized well: the dashboard cannot
observe the worker's interval, so any value was either wasteful or visibly
stalling. Covering a commit is a bounded, local, and much smaller quantity.

The interval also doubles as the spacing between successive triggered reads, so
one bound governs both. They are the same concern viewed from two sides: how
eagerly the dashboard may ask the store whether the next task exists yet. A
separate "delay before first read" and "spacing between reads" pair was
rejected as two tunables with no behavioral difference between them.

Spacing still matters because alignment is not a guarantee. A worker may be
stopped, suspended, or advancing a character the dashboard's prediction has
drifted from, in which case the bar saturates with no commit forthcoming.
Ownership gates triggering, which bounds that case: if the runtime stops,
ownership becomes false at the next combined refresh and both prediction and
triggering cease, so a saturated bar cannot poll indefinitely against a stopped
worker. Reads remain local SQLite queries and never touch the service manager.

### Keep the prediction out of the snapshot type

The predicted percentage is computed per frame and passed into rendering
alongside the snapshot, rather than being written into `DashboardSnapshot`.
When prediction is inactive the persisted percentage is used unchanged.

Storing the predicted value in the snapshot was rejected because the snapshot
is the dashboard's record of persisted state and is also what the
recent-task-update comparison and the lifecycle and brag paths read. Keeping
prediction in a separate value makes it structurally impossible for a predicted
number to reach those paths, which is exactly what the spec forbids.

Following the precedent of `select_worker_elapsed` in the runtime, the
prediction and boundary arithmetic are pure free functions over an anchor and
an elapsed `Duration`, so saturation, clamping, percent stepping, and the
zero-duration task case are unit-testable without a terminal, a store, or a
worker.

## Risks / Trade-offs

- [A task whose duration is zero divides by zero when computing a percent
  boundary] → Treat a zero duration as immediately saturated, matching
  `ProgressBar::done`, and return no boundary deadline rather than computing
  one.
- [A worker holds the lock but stops advancing, leaving the bar saturated] →
  Triggered reads remain separated by the settling interval and read only local
  SQLite, never the service manager; the displayed state stays correct and
  merely stops changing.
- [The bar rests at 100% at each task boundary] → Reduced from up to a worker
  interval to roughly one settling interval, because the worker now persists
  the completion as the prediction saturates rather than at the end of its
  enclosing tick. The residual pause is truthful: the task is genuinely
  complete and only its outcome is still unknown.
- [A task whose remaining time rounds to zero makes the worker spin] → The
  aligned sleep is floored at one 100 ms tick, so the worker always sleeps and
  always advances by a whole tick multiple.
- [Boundary alignment changes progression and breaks timing conformance] → It
  cannot, because every advancement duration remains a whole multiple of the
  100 ms tick grid and ticks are capped rather than padded, so the tick
  sequence is identical to today's. The full suite, including the timing
  checkpoint fixtures, is run to confirm.
- [Waking more often per unit of real time increases worker overhead] → At most
  one extra wake per task boundary, and each wake still advances by at most one
  interval. Tasks are on the order of seconds, so the added rate is far below
  the tick grid.
- [Redrawing far more often than before increases CPU use on an idle desktop] →
  Prediction only runs while a worker is actively advancing the character, the
  redraw rate is floored, and Ratatui's buffer diffing limits output to cells
  that actually changed.
- [Prediction could be mistaken for authoritative progress] → It is confined to
  one bar, never increments the task count, never alters rewards, and is
  discarded at every state read, so any divergence self-corrects within one
  refresh interval.

## Migration Plan

No data migration is required. Persisted canonical state, the store schema, and
the service unit are untouched, and the worker's aligned advancement produces
exactly the state its interval-aligned advancement produces today, so running
workers and existing characters are unaffected either way. A rollback that
removes the prediction and restores both the single combined refresh and the
fixed worker sleep leaves every persisted character compatible.
