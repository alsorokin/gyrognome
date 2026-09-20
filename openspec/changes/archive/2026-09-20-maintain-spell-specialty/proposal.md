# Proposal

## Why

Locally created online characters can learn and rank spells while their
canonical `bestspell` field remains empty. Leaderboard reports correctly send
that field as Specialty, so those characters appear with a blank Specialty
cell even after substantial spell progression.

## What Changes

- Derive the canonical best-spell display value using the official client's
  spell-index-and-rank ranking rule before state is persisted or reported.
- Preserve browser-compatible spell name-and-rank formatting in the derived
  Specialty value sent by existing leaderboard reporting.
- Add regression coverage for spell progression and the resulting report
  payload.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `deterministic-simulation`: Browser-compatible spell progression must keep
  canonical Specialty state current.

## Impact

- Affects native simulation, persisted canonical state, and report snapshots.
- Affects the already-established leaderboard report value for the `z`
  (Specialty) field without changing endpoint, signing, or enrollment behavior.
- Adds focused simulation and report-construction regression coverage.
