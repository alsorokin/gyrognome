# Proposal

## Why

Characters currently lose all potential progression while their worker is stopped or the computer is asleep. A rested-time bank lets them recover up to 12 hours of missed progression through subsequent active play, without granting instant offline rewards.

## What Changes

- Automatically enable rested progression for every managed browser and desktop character, including online characters; no per-character opt-in.
- Accumulate one second of rest per second stopped or asleep/hibernating, capped at 12 hours. Ordinary scheduler stalls and report-delivery delays do not earn rest.
- Spend one banked second per real second of active boosted play. Accelerate all progression, not just experience: a full bank gives 12 real hours at 2x normal speed.
- Persist the bank and its dedicated timing metadata with runtime state, preserve unused rest across interruptions, and return to normal speed exactly when rest expires.
- Preserve browser simulation semantics by servicing complete 100 ms virtual ticks and retaining fractional accelerated time between updates.
- Show remaining rest and the current progression multiplier in managed inspection and both dashboard layouts; make task prediction follow the boost.
- **BREAKING**: replace the runtime's original-speed-only pacing with automatic rested acceleration. Keep simulation rules, callback ordering, request construction, and existing eligibility restrictions intact.
- Explicitly allow rested runtime pacing without new leaderboard evidence. Official acceptance/classification remains unverified; the user will test their own characters after implementation.

## Capabilities

### New Capabilities

- `rested-progression`: capped rest accumulation, active-time spending, persistence, migration, and safe inspection.

### Modified Capabilities

- `local-character-runtime`: distinguish real active time from accelerated simulation time and pace desktop callbacks through a rested virtual clock.
- `terminal-dashboard`: display rested status and predict task progress at the applicable rate, including bank exhaustion.
- `leaderboard-conformance`: narrowly exempt production rested pacing from new live-evidence requirements without claiming accelerated timelines are verified.

## Impact

The main integration points are `src/runtime.rs` (SQLite schema, registration, worker clocks, scheduling, and commits), `src/dashboard.rs` (safe snapshots and prediction), and the CLI's managed inspection formatting. A small Linux clock adapter may require a direct clock-access dependency. Update README runtime/timing guidance and the changelog.

Focused synthetic tests cover bank arithmetic, stop/resume and sleep, both timing profiles, persistence, presentation, and existing report ordering. No live leaderboard requests, new evidence campaign, or player-save fixtures are included.

## Non-goals

Instant offline advancement, XP-only reward multipliers, configurable caps/rates, opt-in controls, rested-state interchange through official saves, and automated leaderboard acceptance verification.
