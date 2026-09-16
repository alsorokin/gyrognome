## Why

Gyrognome currently proves browser-compatible canonical state advancement, but
the Progress Quest leaderboard can classify nonconforming characters separately.
The browser reports signed snapshots at particular transitions, so state
equivalence alone cannot establish that locally progressed characters have a
safe, browser-conformant leaderboard history.

## What Changes

- Add a leaderboard-conformance capability that models the browser's
  leaderboard report events and records credential-free expected report traces.
- Extend deterministic simulation to expose the ordered, point-in-time
  transitions that generate browser report events without compromising its
  purity.
- Extend the compatibility core's report construction coverage to verify
  trigger-specific payload snapshots, field order, normalization, and signing
  against disposable browser observations.
- Define an explicitly opt-in Playwright-harness external conformance procedure
  in which the official browser creates the disposable online character and
  retains its credential only for the ephemeral experiment. Cover starts,
  pauses, restarts, delayed callbacks, task completion, level-up, act
  completion, manual bragging, and motto changes.
- Require evidence that controlled Gyrognome characters remain in the normal
  leaderboard population and are not classified as cheaters before a later
  change can add general leaderboard transport.

## Capabilities

### New Capabilities

- `leaderboard-conformance`: Produces and verifies browser-equivalent
  leaderboard report traces and defines the controlled evidence required to
  establish anti-cheat safety.

### Modified Capabilities

- `deterministic-simulation`: Expose ordered browser-report transition
  snapshots while retaining deterministic, I/O-free state advancement.
- `pq-compatibility-core`: Construct and verify exact trigger-specific
  leaderboard report payloads from canonical browser-transition snapshots
  without transmitting them.

## Impact

- Affects simulation outcome APIs, protocol request construction, synthetic
  browser-derived fixtures, and conformance tests.
- Adds an opt-in Playwright test harness and documented procedure for
  disposable online characters; it must never persist or commit credentials,
  or silently transmit a user's save-derived passkey.
- Native browser character-generation (`newguy`) porting remains a future
  capability and is not required for this conformance gate.
- Does not add production HTTP transport, automatic reporting, online
  character creation, or submission for existing managed characters.
