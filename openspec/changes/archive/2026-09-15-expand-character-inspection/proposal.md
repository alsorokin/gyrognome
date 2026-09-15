## Why

The compatibility core retains complete browser save documents but currently exposes
only name, race, class, level, and online realm. Detailed read-only inspection is
needed to compare native state with browser checkpoints and diagnose future
simulation differences.

## What Changes

- Parse the browser save's simulation-relevant state into a typed canonical model,
  including PRNG state, attributes, task/progress state, equipment, inventory,
  spells, plots, quests, and online metadata.
- Expand `gyrognome inspect` into a complete, credential-safe character-sheet view.
- Add a machine-readable JSON inspection format for deterministic state comparison.
- Validate nested required state with actionable field-level errors while retaining
  unrecognized data for unmodified export.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `pq-compatibility-core`: Expand browser-save interchange and safe inspection from identity-only summary output to a complete typed and machine-readable character state view.

## Impact

- Extends the Rust character-state and save-import modules and the `inspect` CLI.
- Adds synthetic and disposable-save-derived fixtures for full state validation and
  output testing.
- Does not add simulation advancement, local persistence, an interactive UI, or
  leaderboard transport.
