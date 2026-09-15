## Context

The deterministic simulator accepts explicit durations and internally applies
bounded browser-style ticks. The managed worker currently derives those
durations from `Instant` after sleeping at its configured cadence. See
proposal.md and the modified capability specifications for the required timing
and persistence behavior.

## Goals / Non-Goals

**Goals:**

- Make scheduler-delay accounting deterministic, testable, and bounded by the
  selected worker interval.
- Prove that equivalent explicit-duration partitions preserve canonical state
  and Alea continuation across task-completion boundaries.
- Keep persistence atomic when elapsed-duration conversion or simulation fails.

**Non-Goals:**

- Change Progress Quest rules, task-selection behavior, or the configured
  default worker cadence.
- Treat stopped time, process suspension time beyond one interval, or deferred
  failures as catch-up progression.
- Add network transport, leaderboard reporting, or browser credential access.

## Decisions

### Cap each scheduled update at one configured interval

Measure worker time with `Instant`, then advance by the lesser of measured
elapsed time and the configured interval. Reset the timing baseline after every
callback attempt so excess delayed time is deliberately discarded rather than
accumulating into a later update.

This preserves normal cadence while preventing a delayed callback or system
suspension from granting a large progression jump. Counting the entire gap was
rejected because the user selected capped behavior and it would make local
progress depend on scheduler stalls.

### Separate elapsed-time selection from advancement and persistence

Extract the worker's monotonic elapsed-time calculation into a small
clock-independent unit that accepts a prior tick, current tick, and interval.
The worker will use that unit before calling the existing explicit
`advance_elapsed` path.

This permits exact tests for on-time, early, delayed, and restart timing without
sleeping in tests. Testing only `run_until` with real sleeps was rejected as
slow and scheduler-dependent.

### Add paired conformance fixtures at task boundaries

Add sanitized checkpoint cases that advance a state once by a total duration and
in nonzero partitions totaling the same duration. Include at least one sequence
that crosses a task completion and exercises Alea continuation.

This validates a stronger invariant than isolated output snapshots. Property
testing without browser-derived expected states was rejected because it could
confirm internal consistency while missing browser incompatibility.

## Risks / Trade-offs

- [A cap drops real elapsed time during heavy scheduling delay] → Document the
  policy and update the baseline on every callback attempt so behavior is
  predictable and never backfills.
- [Refactoring timing changes ordinary worker cadence] → Test on-time and
  shorter-than-interval updates against the current explicit-duration path.
- [New fixtures accidentally include sensitive source data] → Reuse the
  fixture-safety validator and synthetic disposable-browser inputs.

## Migration Plan

1. Add the elapsed-time selection unit and tests before changing the worker
   loop.
2. Apply the cap in the worker and validate no failed update writes partial
   state.
3. Add paired conformance checkpoints and run the complete test suite.
4. Roll back by restoring the prior worker timing implementation; persisted
   canonical state and database schema remain compatible.
