## Context

See proposal.md - Why. The official client derives both Specialty and Prime
Stat in `HotOrNot()` during `SaveGame()`, which runs before `Brag()`. Gyrognome
already refreshes `bestspell` at advancement, snapshot, and explicit-report
boundaries, but it preserves `beststat` and `Stats.best` from the prior state.
Report construction then sends stale `beststat` in field `k`.

## Goals / Non-Goals

**Goals:**

- Make saved canonical state and every report snapshot use the official
  browser's current Prime Stat selection.
- Repair stale Prime Stat metadata for existing managed characters at their
  next persistence or report boundary without a migration.
- Preserve deterministic simulation, random-state continuation, and report
  request compatibility.

**Non-Goals:**

- Change the six-stat advancement algorithm, reward distribution, or
  encumbrance behavior.
- Change leaderboard endpoint selection, field ordering, signatures, or
  server-classification behavior.
- Retroactively rewrite every stored character before it is next advanced or
  reported.

## Decisions

### Derive Prime Stat from current prime-stat values

Add a pure counterpart to Specialty derivation that examines prime stats in the
browser's fixed order: `STR`, `CON`, `DEX`, `INT`, `WIS`, then `CHA`. Select the
largest integer value using strict comparison, so the first stat remains the
winner on ties. Set both `Stats.best` and `beststat` to browser-compatible
display values.

This mirrors the browser's `HotOrNot()` loop exactly. Reusing the ruleset's
prime-stat order is preferred over maintaining a second independently ordered
list. Selecting the most recently increased stat is rejected because it does
not match the browser and would produce incorrect reports after earlier stats
remain larger.

### Refresh at existing save and report boundaries

Invoke Prime Stat derivation wherever Specialty is currently refreshed:
before a successor simulation state is returned for persistence, when a
credential-free transition snapshot is captured, and when explicit reporting
constructs a working state. This makes automatic level-up/act snapshots,
manual brags, and motto reports consistent without changing protocol
construction or adding report-only state.

Updating only report construction is rejected because persisted character data
and dashboard state would remain stale. Updating only during level-up is
rejected because existing characters and manual reports can cross a reporting
boundary without a new stat reward.

### Test state maintenance separately from request construction

Use focused deterministic tests to prove winner changes, output values, and
tie ordering. Add report-path tests that begin with intentionally stale metadata
and assert field `k` receives the derived display value for automatic snapshots
and explicit manual actions. Keep synthetic fixtures credential-free and update
only fixtures whose expected canonical snapshots legitimately change.

## Risks / Trade-offs

- [A refresh path is missed] -> Cover successor-state persistence, automatic
  snapshots, manual brags, and motto changes with stale-metadata regressions.
- [Tie behavior diverges from the browser] -> Centralize derivation and test
  strict first-in-order selection.
- [Derived values alter signed request output] -> Assert the changed `k` value
  alongside unchanged endpoint, field order, and signature construction rules.

## Migration Plan

1. Release the pure boundary derivation with regression coverage.
2. Existing managed characters receive correct Prime Stat metadata on their
   next advance, manual report, motto action, or worker-generated report; no
   database migration is required.
3. Roll back by reverting the derivation change. Existing refreshed values
   remain valid display data, and no protocol migration is needed.
