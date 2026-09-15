# Gyrognome Roadmap

Gyrognome is a Linux-native Progress Quest client. Its compatibility,
inspection, deterministic simulation, and local-runtime foundations are complete
and intentionally offline-only.

## Completed: Compatibility Core

- Browser `.pqw` import and unmodified export support.
- Safe, redacted character identity inspection.
- Browser-compatible Alea continuation, URL encoding, URL normalization, and LFSR
  request construction.
- Sanitized browser-reference fixtures and an offline-only transport boundary.

## Completed: Character Inspection

The `pq-compatibility-core` capability now provides a full read-only character
sheet:

- Parse and validate PRNG state, stats, elapsed time, active task, bars,
  equipment, inventory, spells, plots, quests, and online metadata.
- Add structured terminal output and a `--json` format for browser/native state
  comparison.
- Separate canonical state from cached browser display fields while preserving
  unrecognized data for safe round-tripping.
- Test with synthetic fixtures and disposable browser-save checkpoints.
- Keeps passkeys and unrecognized raw save data out of inspection output.

## Completed: Deterministic Simulation

The `deterministic-simulation` capability advances canonical state in
browser-compatible order from explicit elapsed durations, bundled versioned rule
data, and synthetic browser-derived conformance checkpoints. It remains pure:
no clock, filesystem, database, or network access is reachable from simulation.

## Completed: Local Character Runtime

The `local-character-runtime` capability adds XDG-scoped SQLite persistence,
per-character process locking, and `systemd --user` lifecycle control. Managed
characters progress only while their local runtime is active; stopped time does
not catch up. The runtime remains local-only and does not report to leaderboards.

## Completed: Terminal Dashboard

The `terminal-dashboard` capability provides a credential-safe terminal view
of one managed character:

- Displays live identity, human-readable activity, progress bars, equipped
  items, inventory, spells, plots, quests, and runtime/service status.
- Refreshes persisted state without owning character locks or advancing
  simulation.
- Provides confirmed lifecycle actions through the existing runtime controls.
- Restores the terminal on quit, terminal failure, or interrupt.
- Keeps browser passkeys, raw save documents, HTTP transport, and leaderboard
  reporting out of scope.

## Planned Sequence

1. Reassess timing and simulation conformance.
2. Only when that reassessment is proven, add leaderboard reporting.
