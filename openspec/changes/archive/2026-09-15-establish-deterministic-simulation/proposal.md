## Why

Gyrognome can now inspect complete browser character state, but cannot advance it.
A deterministic simulation core is needed before persistence, a terminal UI, or
leaderboard reporting can safely exist.

## What Changes

- Add a pure deterministic simulation engine that advances imported character
  state through Progress Quest task completion without using wall-clock time.
- Unify browser-compatible Alea state between parsed character state and random
  number generation.
- Include a versioned, provenance-recorded browser ruleset required for initial
  task selection, combat resolution, rewards, quests, plots, and level-ups.
- Add browser-derived, synthetic conformance checkpoints that assert resulting
  state and RNG continuation after fixed simulation steps.
- Keep scheduling, save mutation, SQLite persistence, terminal UI, HTTP
  transport, and leaderboard reporting out of scope.

## Capabilities

### New Capabilities

- `deterministic-simulation`: Advance Progress Quest character state through
  browser-compatible task completion and verify it against fixed checkpoints.

### Modified Capabilities

- None.

## Impact

- Adds a simulation module, browser ruleset data, and deterministic conformance
  fixtures/tests.
- Refactors the existing typed character state and Alea representation into a
  shared canonical form.
- Does not add network or runtime dependencies.
