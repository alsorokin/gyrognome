## Why

Gyrognome currently advances the six live prime-stat values but can retain an
obsolete `beststat` display string from character creation. Because leaderboard
reports send that string as their Prime Stat field, an online character can
appear with a much lower, stale value than its current attributes.

The official browser refreshes Prime Stat when it saves immediately before
reporting. Gyrognome needs the same boundary behavior to keep persisted state
and leaderboard reports browser-compatible.

## What Changes

- Derive canonical Prime Stat from the current `STR`, `CON`, `DEX`, `INT`,
  `WIS`, and `CHA` values using the official browser's ordering and display
  format.
- Refresh `beststat` and the corresponding `Stats.best` marker before a
  simulated state is persisted and before automatic or explicit report
  snapshots are constructed.
- Ensure leaderboard field `k` uses the refreshed Prime Stat while preserving
  existing report endpoints, field order, signing, and eligibility rules.
- Add regression coverage for changed winning values, changed winning
  attributes, stable browser-order tie handling, and report serialization.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `deterministic-simulation`: Browser-compatible state save and report
  boundaries must maintain the current Prime Stat display value and marker.

## Impact

- Affects deterministic state derivation and report-transition snapshots in
  `src/simulation.rs`.
- Affects manual, motto, and worker-generated reporting paths in
  `src/reporting.rs` through their use of the refreshed canonical state.
- Adds focused simulation and report-construction regression tests; no
  endpoint, protocol-field-order, persistence-format, or dependency changes
  are expected.
