## Why

The current simulator and local worker have baseline conformance coverage, but
they have not yet been reassessed for elapsed-time accounting across long runs,
scheduler delay, and worker restarts. That proof is required before any future
online reporting could safely rely on locally advanced state.

## What Changes

- Define bounded, monotonic worker-time accounting and explicit behavior for
  scheduler delay, sub-millisecond elapsed time, and restart boundaries.
- Expand deterministic simulation conformance checkpoints to cover elapsed-time
  chunking, long-running progression, and task transitions that cross worker
  persistence updates.
- Make the runtime surface a clear failure without persisting partial state
  when an elapsed duration cannot be represented safely.
- Preserve offline-only operation: this change does not add HTTP transport,
  leaderboard reporting, or browser credential use.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `deterministic-simulation`: Define stronger elapsed-time conformance and
  checkpoint coverage for equivalent advancement sequences.
- `local-character-runtime`: Define monotonic worker timing, restart, and
  persistence behavior under scheduler delay and elapsed-duration limits.

## Impact

- Affects `src/simulation.rs`, `src/runtime.rs`, and their unit and synthetic
  checkpoint tests.
- May add bounded timing constants and test fixtures, but does not require
  schema migration, new runtime services, network dependencies, or changes to
  the dashboard lifecycle interface.
