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

## Completed: Managed Character Administration

The `managed-character-administration` capability lets users remove unwanted
local registrations without editing the database:

- `gyrognome delete <character-id>` displays only the safe identity and requires
  explicit confirmation before removal.
- Removal is atomic and refuses a character currently owned by a local worker;
  users must stop it before retrying.
- Administration remains local-only and never exposes browser passkeys, raw
  saves, or unrecognized source fields.

## Completed: Terminal Dashboard

The `terminal-dashboard` capability provides a credential-safe terminal view
of a managed character:

- Displays live identity, human-readable activity, progress bars, equipped
  items, inventory, spells, plots, quests, and runtime/service status.
- Lets users run `gyrognome dashboard` without an identifier and select a
  registered character, while preserving direct identifier-based startup.
- Refreshes persisted state without owning character locks or advancing
  simulation.
- Provides confirmed lifecycle actions through the existing runtime controls.
- Restores the terminal on quit, terminal failure, or interrupt.
- Keeps browser passkeys, raw save documents, HTTP transport, and leaderboard
  reporting out of scope.

## Completed: Simulation Timing Conformance

The deterministic simulation and local runtime now have stronger timing
conformance guarantees:

- Worker elapsed time is monotonic and capped at one configured interval, so
  scheduler delays and process suspension do not grant catch-up progression.
- Equivalent nonzero elapsed-duration partitions preserve canonical state and
  Alea continuation, including across task-completion boundaries.
- Simulation and elapsed-duration failures preserve the last persisted
  canonical state without partial writes.
- Sanitized paired browser-derived checkpoints cover partitioning and task
  completion behavior.

## Completed: Leaderboard Conformance and Anti-Cheat Safety

The `leaderboard-conformance` capability proves that local state advancement
is browser-equivalent and that controlled characters are not classified as
cheaters:

- Credential-free, ordered report-transition traces (`s`/`l`/`a`/`b`/`m`) from
  deterministic simulation, matching the browser's exact report call sites.
- Trigger-specific report construction and synthetic/browser-derived trace
  conformance fixtures, with no signed request or passkey ever committed.
- A disposable-character, explicitly confirmed Playwright harness that lets
  the official browser create and hold the online credential, so Gyrognome's
  reported traces are compared against it without touching a real save.
- Recorded evidence, gated behind explicit operator confirmation: paired
  browser/Gyrognome scenario traces match, and the disposable character
  remains in the normal leaderboard population rather than the cheater one.

General leaderboard reporting for managed characters remains unsupported
until a dedicated change adds it.

## Planned Sequence

1. Add general leaderboard reporting for managed characters, building on the
   passing leaderboard conformance and anti-cheat safety evidence.
