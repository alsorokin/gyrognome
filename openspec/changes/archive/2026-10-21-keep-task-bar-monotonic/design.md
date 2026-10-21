# Design

## Context

See proposal.md for motivation. The dashboard predicts the task bar from a
`TaskAnchor` (observed position, duration, identity, observation instant,
rate, remaining rest boost) via the pure `predicted_task_position(anchor,
elapsed)`. On each successful state read, `replace_character` builds a new
anchor and, only when the task identity, runtime ownership, *and* rested view
are unchanged, raises the anchor position to the currently displayed position.
Rested `available_ms` shrinks with every worker publication, so while rested
the hold never applies and the bar rewinds to the persisted position. The
worker-to-dashboard observation lag is counted at 2x task time while rested,
which makes the rewind larger.

The rested condition was likely added because a plain `max` re-anchors *from*
the displayed position: any over-prediction (for example rest exhausting
earlier than predicted) would be carried forward on every read and accumulate.

## Goals / Non-Goals

**Goals:**

- No visible backward step within a task, rested or not.
- Any lead over the worker shrinks steadily instead of accumulating.
- Prediction stays a pure function of an anchor and elapsed time.

**Non-Goals:**

- Tighter synchronization with the worker (already attempted).
- Smoothing the transition between tasks.

## Decisions

### Display the max of two lines

The anchor keeps the worker-observed position and gains a `floor_ms`: the
displayed position at re-anchor time (equal to the observed position when not
ahead). The displayed position is:

```
anchored(t) = predicted_task_position(anchor, t)            // unchanged
floor(t)    = floor_ms + 0.5 * (anchored(t) - position_ms)  // half rate
displayed   = max(anchored(t), floor(t))  clamped to duration
```

Using half of the anchored *advance* (rather than a separate rate) reuses the
existing rate, rest-boost split, and exhaustion handling for free. At `t = 0`
the value is `max(position_ms, floor_ms)`, at least the previous display, so no
rewind. Both lines are non-decreasing. `anchored` gains on `floor` at half the
applicable rate, so the lead closes in the time the worker needs to cover twice
the lead, after which the bar follows the worker-anchored prediction again.

Alternatives considered:

- *Keep plain `max` and drop the rested condition*: no rewind, but over-
  prediction accumulates without bound.
- *Freeze until caught up (k = 0)*: simplest, but a visible stall.
- *Ease toward the target with a time constant*: not a simple closed form,
  which makes redraw-boundary scheduling harder.

### Re-anchor rule

In `replace_character`, when the identity is unchanged and the runtime owns the
character, set `floor_ms = displayed_task_position(now)`. Otherwise use
`floor_ms = position_ms`. The rested-equality condition is removed. The
read-error path, which already re-anchors to the displayed position, keeps
doing that. Setting both `position_ms` and `floor_ms` to the displayed value is
already consistent.

### Scheduling follows the displayed curve

`next_task_percent_boundary` and `settling_deadline` currently invert the
anchored prediction. They will invert the displayed curve instead. While
`floor` leads, the remaining distance is covered at half rate, so the virtual
time needed is doubled up to the catch-up point. A simple approach that is good
enough: compute the time for each line separately to reach the target, then
take the smaller time, because the displayed value is their max. Saturation
uses the same rule, so the settling read is not scheduled before the displayed
bar is full.

## Risks / Trade-offs

- [After a large lead the bar moves at half speed for a while] → The lead is
  normally an observation lag (well under a second of task time), so the slow
  period is short. The 1 s poll cap while rested stays.
- [A lead can still reach 100% early and hold] → The existing behavior holds
  the bar full until the next persisted state. The half-rate floor makes this
  less likely, not more.
