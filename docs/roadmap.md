# Gyrognome Roadmap

Gyrognome is a Linux-native Progress Quest client. The initial compatibility-core
change is complete and intentionally offline-only.

## Completed: Compatibility Core

- Browser `.pqw` import and unmodified export support.
- Safe, redacted character identity inspection.
- Browser-compatible Alea continuation, URL encoding, URL normalization, and LFSR
  request construction.
- Sanitized browser-reference fixtures and an offline-only transport boundary.

## Next: Expand Character Inspection

Extend the existing `pq-compatibility-core` capability with a full read-only
character sheet:

- Parse and validate PRNG state, stats, elapsed time, active task, bars,
  equipment, inventory, spells, plots, quests, and online metadata.
- Add structured terminal output and a `--json` format for browser/native state
  comparison.
- Separate canonical state from cached browser display fields while preserving
  unrecognized data for safe round-tripping.
- Test with synthetic fixtures and disposable browser-save checkpoints.
- Keep the client read-only: no progression, local persistence, service, or
  leaderboard submission.

## Planned Sequence

1. Expand character inspection.
2. Implement deterministic simulation that advances canonical state exactly as
   the browser client does.
3. Add local character runtime: SQLite persistence, process locking, and service
   lifecycle.
4. Add the terminal dashboard.
5. Reassess and, only when timing and simulation conformance are proven, add
   leaderboard reporting.

## Repository Housekeeping

The completed compatibility-core change was archived and its delta specification
was synced to `openspec/specs/pq-compatibility-core/spec.md`. Commit those archive
and specification moves before beginning the next change.
