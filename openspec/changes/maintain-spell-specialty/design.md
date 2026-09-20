# Design

## Context

See proposal.md - Why. Native character creation intentionally begins with no
spells and an empty `bestspell` value. The official client does not set
Specialty directly in its spell-reward function. Instead, each `Brag()` first
saves state, and saving recomputes Specialty across all learned spells. Existing
report construction already reads `bestspell` for the leaderboard Specialty
parameter, so no request-shape change is required.

## Goals / Non-Goals

**Goals:**

- Recompute `bestspell` using the official client's selection rule before
  persisted state and every reportable snapshot.
- Preserve the browser-facing display form of a spell name followed by its
  resulting Roman-numeral rank.
- Prove the canonical state and report construction expose the updated value.

**Non-Goals:**

- Change leaderboard endpoints, request field order, signing, or enrollment.
- Alter spell-selection randomness or rank progression.

## Decisions

### Derive Specialty from the complete learned-spell collection

Select the spell maximizing `(zero-based collection index + 1) * rank`, where
rank is the Arabic value of the stored Roman numeral; use strict comparison so
the earlier spell wins a tie. Store its name-and-rank display value, or an empty
value if no spells are learned.

This is the exact `HotOrNot()` rule the official client applies during
`SaveGame()`. Selecting the most recently rewarded spell or only the
highest-ranked spell was rejected because neither matches the official
selection rule.

### Refresh Specialty at save and report boundaries

Apply the derivation immediately before native state is persisted and before
every explicit or automatic report snapshot is constructed. This mirrors
`Brag()` calling `SaveGame()` before building the leaderboard request and
repairs existing persisted characters with learned spells without requiring
their reward history.

The derivation is a pure function of canonical spells, so this neither changes
random-state progression nor requires a database migration.

### Test state maintenance and protocol use separately

Use deterministic tests for ranking, ties, and empty spells; use trace and
manual-report tests to assert `z` receives the derived value at every report
boundary. This distinguishes rank-selection, snapshot, and serialization
defects.

## Risks / Trade-offs

- [Report paths bypass the derivation] -> Cover automatic transition snapshots,
  explicit actions, and manual submissions with regression tests.
- [Ranking or tie behavior drifts] -> Centralize the pure derivation and cover
  rank products, ties, and empty spells with exact tests.

## Migration Plan

1. Release the save/report-boundary derivation with regression tests.
2. Existing characters receive a valid Specialty the next time their state is
   saved or a report is prepared; no data migration is performed.
3. Roll back by reverting the derivation change; persisted Specialty values
   remain valid display data and reporting behavior is otherwise unchanged.
