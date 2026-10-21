# Proposal

## Why

The dashboard's predicted task bar visibly snaps back a few percent when it
re-anchors to a newly read persisted state, most noticeably while rest is
active. The original prediction design held the bar non-decreasing within a
task, but the rested-progression change gated that hold on unchanged rested
values, and rested balance changes on nearly every worker publication, so the
hold is effectively disabled while rested. Exact synchronization with the
worker was already attempted; this change mitigates the visual effect instead.

## What Changes

- Within a single task, the displayed task bar never decreases, regardless of
  rested state.
- When the display is ahead of the worker's observed position at a re-anchor,
  the bar keeps advancing at half the applicable predicted rate until the
  worker-anchored prediction catches up, then follows the worker-anchored
  prediction again.
- A new task (changed task identity) still displays the newly persisted
  position directly, including the reset to the start of the next task.
- Redraw scheduling and the saturation-triggered settling read follow the
  displayed (possibly slowed) bar rather than the raw prediction.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `terminal-dashboard`: the "Predicted current-task progress" requirement
  gains a within-task non-decreasing guarantee and half-rate catch-up while the
  display leads the worker.

## Impact

- `src/dashboard.rs`: task anchor, prediction, re-anchoring in
  `replace_character`, the read-error path, redraw boundary, and settling
  deadline, plus their unit tests.
- No changes to the worker, persisted state, simulation, or reporting.
